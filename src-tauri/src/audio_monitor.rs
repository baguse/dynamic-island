use serde::{Deserialize, Serialize};
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub const SAMPLE_RATE: u32 = 22050;
pub const CHUNK_SIZE: usize = 256;
const TARGET_FREQS: [f32; 5] = [65.0, 190.0, 650.0, 2400.0, 7200.0];
const FREQ_WEIGHTS: [f32; 5] = [1.2, 1.6, 2.4, 3.6, 5.2];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioSpectrumData {
    pub bands: [f32; 5],
    pub energy: f32,
}

/// Compute Goertzel magnitude for a specific frequency
pub fn goertzel_mag(samples: &[f32], freq: f32, sample_rate: f32) -> f32 {
    let n = samples.len() as f32;
    let k = (n * freq / sample_rate).round();
    let omega = 2.0 * std::f32::consts::PI * k / n;
    let coeff = 2.0 * omega.cos();

    let mut s_prev = 0.0f32;
    let mut s_prev2 = 0.0f32;

    for &s in samples {
        let s_curr = s + coeff * s_prev - s_prev2;
        s_prev2 = s_prev;
        s_prev = s_curr;
    }

    let power = s_prev * s_prev + s_prev2 * s_prev2 - coeff * s_prev * s_prev2;
    (power.max(0.0)).sqrt() / (n / 2.0)
}

/// Process a chunk of PCM samples and return 5 normalized spectrum bands and energy
pub fn process_audio_chunk(
    samples: &[f32],
    peak: &mut f32,
    prev_levels: &mut [f32; 5],
) -> AudioSpectrumData {
    let n = samples.len();
    if n == 0 {
        return AudioSpectrumData {
            bands: [0.0; 5],
            energy: 0.0,
        };
    }

    // 1. RMS Energy
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    let rms = (sum_sq / n as f32).sqrt();
    let energy = (rms * 12.0).clamp(0.0, 1.0);

    // 2. Frequency bands via Goertzel filter
    let mut raw_bands = [0.0f32; 5];
    let mut current_max = 0.001f32;

    for i in 0..5 {
        let mag = goertzel_mag(samples, TARGET_FREQS[i], SAMPLE_RATE as f32);
        let weighted = mag * FREQ_WEIGHTS[i];
        raw_bands[i] = weighted;
        if weighted > current_max {
            current_max = weighted;
        }
    }

    // Adaptive Automatic Gain Control (AGC) peak tracker
    *peak = (*peak * 0.985).max(current_max).max(0.015);

    // 3. Normalize with responsive attack and snappy decay
    let mut normalized_bands = [0.0f32; 5];
    for i in 0..5 {
        let norm = (raw_bands[i] / *peak).sqrt().clamp(0.0, 1.0);
        let smoothed = if norm > prev_levels[i] {
            // Instant attack
            norm
        } else {
            // Snappy gravity decay
            norm.max(prev_levels[i] * 0.72)
        };
        prev_levels[i] = smoothed;
        normalized_bands[i] = (smoothed * 100.0).round() / 100.0;
    }

    AudioSpectrumData {
        bands: normalized_bands,
        energy: (energy * 100.0).round() / 100.0,
    }
}

/// Spawns the background PipeWire / PulseAudio monitor thread
pub fn start_audio_monitor(app_handle: AppHandle, is_running: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let sample_bytes = CHUNK_SIZE * 2; // 16-bit mono = 2 bytes per sample
        let mut byte_buffer = vec![0u8; sample_bytes];
        let mut samples = vec![0.0f32; CHUNK_SIZE];
        let mut peak = 0.03f32;
        let mut prev_levels = [0.0f32; 5];

        while is_running.load(Ordering::SeqCst) {
            // Attempt to start low-latency parec capture (15ms buffer)
            let mut child: Option<Child> = Command::new("parec")
                .args([
                    "--raw",
                    &format!("--rate={}", SAMPLE_RATE),
                    "--channels=1",
                    "--format=s16le",
                    "--latency-msec=15",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .ok();

            if let Some(ref mut proc) = child {
                if let Some(ref mut stdout) = proc.stdout {
                    let mut last_emit = std::time::Instant::now();

                    while is_running.load(Ordering::SeqCst) {
                        match stdout.read_exact(&mut byte_buffer) {
                            Ok(()) => {
                                // Convert 16-bit little-endian PCM to f32 [-1.0, 1.0]
                                for i in 0..CHUNK_SIZE {
                                    let b0 = byte_buffer[i * 2];
                                    let b1 = byte_buffer[i * 2 + 1];
                                    let sample_i16 = i16::from_le_bytes([b0, b1]);
                                    samples[i] = sample_i16 as f32 / 32768.0;
                                }

                                // Process audio chunk
                                let spectrum = process_audio_chunk(
                                    &samples,
                                    &mut peak,
                                    &mut prev_levels,
                                );

                                // Fast emission (~55 FPS, every ~18ms)
                                if last_emit.elapsed() >= Duration::from_millis(18) {
                                    let _ = app_handle.emit("audio-spectrum", &spectrum);
                                    last_emit = std::time::Instant::now();
                                }
                            }
                            Err(_) => {
                                // Pipe broken or parec terminated
                                break;
                            }
                        }
                    }
                }
                let _ = proc.kill();
                let _ = proc.wait();
            }

            // If parec failed or terminated, pause before retrying
            std::thread::sleep(Duration::from_millis(1500));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goertzel_sine_detection() {
        let n = 256;
        let fs = 22050.0;
        let target_freq = 65.0;

        // Generate pure 60 Hz sine wave
        let mut samples = vec![0.0f32; n];
        for i in 0..n {
            let t = i as f32 / fs;
            samples[i] = (2.0 * std::f32::consts::PI * target_freq * t).sin();
        }

        let mag_60 = goertzel_mag(&samples, 60.0, fs);
        let mag_1000 = goertzel_mag(&samples, 1000.0, fs);

        assert!(mag_60 > 0.8, "60Hz magnitude should be high (found {})", mag_60);
        assert!(mag_1000 < 0.1, "1000Hz magnitude should be low (found {})", mag_1000);
    }

    #[test]
    fn test_process_audio_chunk() {
        let mut peak = 0.05f32;
        let mut prev = [0.0f32; 5];
        let samples = vec![0.0f32; CHUNK_SIZE];

        let res = process_audio_chunk(&samples, &mut peak, &mut prev);
        assert_eq!(res.energy, 0.0);
        for &b in &res.bands {
            assert!(b >= 0.0 && b <= 1.0);
        }
    }
}
