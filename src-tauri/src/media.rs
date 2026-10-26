use mpris::{PlaybackStatus, PlayerFinder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaMetadata {
    pub player_name: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub art_url: Option<String>,
    pub status: String, // "Playing", "Paused", "Stopped"
    pub position_ms: Option<u64>,
    pub length_ms: Option<u64>,
}

pub fn get_active_player_metadata() -> Option<MediaMetadata> {
    let finder = PlayerFinder::new().ok()?;

    // First try find_active, fallback to the first running player
    let player = finder
        .find_active()
        .ok()
        .or_else(|| finder.find_all().ok()?.into_iter().next())?;

    let identity = player.identity().to_string();
    let status = match player.get_playback_status().unwrap_or(PlaybackStatus::Stopped) {
        PlaybackStatus::Playing => "Playing".to_string(),
        PlaybackStatus::Paused => "Paused".to_string(),
        PlaybackStatus::Stopped => "Stopped".to_string(),
    };

    let metadata = player.get_metadata().ok();
    let title = metadata
        .as_ref()
        .and_then(|m| m.title())
        .unwrap_or("Unknown Track")
        .to_string();

    let artist = metadata
        .as_ref()
        .and_then(|m| m.artists())
        .map(|a| a.join(", "))
        .unwrap_or_else(|| "Unknown Artist".to_string());

    let album = metadata
        .as_ref()
        .and_then(|m| m.album_name())
        .unwrap_or("")
        .to_string();

    let art_url = metadata.as_ref().and_then(|m| m.art_url()).map(String::from);

    let length_ms = metadata
        .as_ref()
        .and_then(|m| m.length())
        .map(|d| d.as_millis() as u64);

    let position_ms = player.get_position().ok().map(|d| d.as_millis() as u64);

    Some(MediaMetadata {
        player_name: identity,
        title,
        artist,
        album,
        art_url,
        status,
        position_ms,
        length_ms,
    })
}

pub fn play_pause() -> Result<(), String> {
    let finder = PlayerFinder::new().map_err(|e| e.to_string())?;
    let player = finder
        .find_active()
        .map_err(|e| e.to_string())
        .or_else(|_| {
            finder
                .find_all()
                .map_err(|e| e.to_string())?
                .into_iter()
                .next()
                .ok_or_else(|| "No media players found".to_string())
        })?;

    player.play_pause().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn next() -> Result<(), String> {
    let finder = PlayerFinder::new().map_err(|e| e.to_string())?;
    let player = finder
        .find_active()
        .map_err(|e| e.to_string())
        .or_else(|_| {
            finder
                .find_all()
                .map_err(|e| e.to_string())?
                .into_iter()
                .next()
                .ok_or_else(|| "No media players found".to_string())
        })?;

    player.next().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn previous() -> Result<(), String> {
    let finder = PlayerFinder::new().map_err(|e| e.to_string())?;
    let player = finder
        .find_active()
        .map_err(|e| e.to_string())
        .or_else(|_| {
            finder
                .find_all()
                .map_err(|e| e.to_string())?
                .into_iter()
                .next()
                .ok_or_else(|| "No media players found".to_string())
        })?;

    player.previous().map_err(|e| e.to_string())?;
    Ok(())
}
