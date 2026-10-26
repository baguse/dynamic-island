use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let toggle_item = MenuItem::with_id(app, "toggle", "Toggle Dynamic Island", true, None::<&str>)?;
    let cycle_item = MenuItem::with_id(app, "cycle_monitor", "Cycle Screen ⮂", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings & Preferences", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Dynamic Island", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &toggle_item,
            &cycle_item,
            &settings_item,
            &quit_item,
        ],
    )?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Dynamic Island")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => {
                let _ = app.emit("tray-toggle-island", ());
            }
            "cycle_monitor" => {
                let _ = app.emit("tray-cycle-monitor", ());
            }
            "settings" => {
                let _ = app.emit("tray-open-settings", ());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = app.emit("tray-toggle-island", ());
            }
        });

    let icon = match app.default_window_icon() {
        Some(i) => i.clone(),
        None => tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))?,
    };
    builder = builder.icon(icon);

    builder.build(app)?;

    Ok(())
}
