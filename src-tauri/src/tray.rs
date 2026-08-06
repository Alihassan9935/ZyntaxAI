use crate::state::AppState;
use crate::{events, windows};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, Emitter, Manager};

const ID_ENABLED: &str = "enabled";
const ID_FIX: &str = "fix-clipboard";
const ID_SETTINGS: &str = "settings";
const ID_QUIT: &str = "quit";

pub fn install(app: &App) -> tauri::Result<()> {
    let enabled_now = app.state::<AppState>().settings().enabled;

    let enabled = CheckMenuItem::with_id(
        app,
        ID_ENABLED,
        "Corrections enabled",
        true,
        enabled_now,
        None::<&str>,
    )?;
    let fix = MenuItem::with_id(app, ID_FIX, "Correct clipboard text", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, ID_SETTINGS, "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, ID_QUIT, "Quit ZyntaxAI", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &enabled,
            &PredefinedMenuItem::separator(app)?,
            &fix,
            &PredefinedMenuItem::separator(app)?,
            &settings,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    *app.state::<AppState>()
        .tray_enabled_item
        .lock()
        .expect("tray-item lock poisoned") = Some(enabled);

    let mut builder = TrayIconBuilder::with_id("zyntax-tray")
        .tooltip("ZyntaxAI")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            ID_ENABLED => toggle_enabled(app),
            ID_FIX => crate::fix::trigger_from_clipboard(app.clone()),
            ID_SETTINGS => windows::show_settings(app),
            ID_QUIT => app.exit(0),
            other => tracing::warn!(id = other, "unhandled tray menu item"),
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                windows::show_settings(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

fn toggle_enabled(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    let mut settings = state.settings();
    settings.enabled = !settings.enabled;

    match state.save_settings(settings) {
        Ok(saved) => {
            tracing::info!(
                enabled = saved.enabled,
                "master switch toggled from the tray"
            );
            state.sync_tray_enabled();
            let _ = app.emit(events::SETTINGS_CHANGED, &saved);
        }
        Err(err) => {
            tracing::error!(%err, "could not persist the master switch");

            state.sync_tray_enabled();
        }
    }
}
