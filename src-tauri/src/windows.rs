use crate::events::{MAIN_WINDOW, OVERLAY_WINDOW};
use tauri::{AppHandle, LogicalPosition, Manager, PhysicalPosition, WebviewWindow};

pub fn show_settings(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        tracing::error!("settings window is missing");
        return;
    };

    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

pub fn hide_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.hide();
    }
}

pub fn overlay(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(OVERLAY_WINDOW)
}

pub fn show_overlay_at_cursor(app: &AppHandle) {
    let Some(window) = overlay(app) else {
        tracing::error!("overlay window is missing");
        return;
    };

    if let Err(err) = position_at_cursor(app, &window) {
        tracing::debug!(%err, "could not position the overlay at the cursor; using its last position");
    }

    let _ = window.show();
    let _ = window.set_focus();
}

pub fn hide_overlay(app: &AppHandle) {
    if let Some(window) = overlay(app) {
        let _ = window.hide();
    }
}

const CURSOR_OFFSET: f64 = 12.0;

const SCREEN_MARGIN: f64 = 8.0;

fn position_at_cursor(app: &AppHandle, window: &WebviewWindow) -> tauri::Result<()> {
    let cursor = app.cursor_position()?;

    let monitor = app
        .monitor_from_point(cursor.x, cursor.y)?
        .or(app.primary_monitor()?);
    let Some(monitor) = monitor else {
        return Ok(());
    };

    let scale = monitor.scale_factor();
    let size = window.outer_size()?.to_logical::<f64>(scale);
    let screen = monitor.size().to_logical::<f64>(scale);
    let origin = monitor.position().to_logical::<f64>(scale);

    let cursor = PhysicalPosition::new(cursor.x, cursor.y).to_logical::<f64>(scale);
    let max_x = origin.x + screen.width - size.width - SCREEN_MARGIN;
    let max_y = origin.y + screen.height - size.height - SCREEN_MARGIN;

    let x = (cursor.x + CURSOR_OFFSET).clamp(origin.x + SCREEN_MARGIN, max_x.max(origin.x));
    let y = (cursor.y + CURSOR_OFFSET).clamp(origin.y + SCREEN_MARGIN, max_y.max(origin.y));

    window.set_position(LogicalPosition::new(x, y))
}
