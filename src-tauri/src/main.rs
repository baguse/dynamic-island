// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  #[cfg(target_os = "linux")]
  unsafe {
    // Disable WebKitGTK DMABUF renderer to eliminate transparency noise artifacts on Linux X11
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    std::env::set_var("NO_AT_BRIDGE", "1");
    std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
  }

  dynamic_island_lib::run();
}
