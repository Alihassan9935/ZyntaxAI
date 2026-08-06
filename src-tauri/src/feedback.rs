use crate::fix::FixError;
use tauri::AppHandle;
#[cfg(not(target_os = "linux"))]
use tauri_plugin_notification::NotificationExt;
use zyntax_core::FixOutcome;
use zyntax_store::AppSettings;

pub fn notify_success(app: &AppHandle, settings: &AppSettings, outcome: &FixOutcome) {
    if !settings.behavior.show_notifications {
        return;
    }

    let body = if outcome.changed {
        summarise(&outcome.corrected)
    } else {
        "No changes needed.".to_owned()
    };

    show(app, body, settings.behavior.play_sound);
}

fn show(app: &AppHandle, body: String, sound: bool) {
    #[cfg(target_os = "linux")]
    {
        let _ = app;
        std::thread::spawn(move || {
            let mut notification = notify_rust::Notification::new();

            notification
                .appname("ZyntaxAI")
                .icon("dev.theholyonez.zyntaxai")
                .summary("ZyntaxAI")
                .body(&body);
            if sound {
                notification.sound_name(SOUND);
            }
            if let Err(err) = notification.show() {
                tracing::debug!(%err, "could not show notification");
            }
        });
    }

    #[cfg(not(target_os = "linux"))]
    {
        let app = app.clone();
        std::thread::spawn(move || {
            let mut builder = app.notification().builder().title("ZyntaxAI").body(body);
            if sound {
                builder = builder.sound(SOUND);
            }
            if let Err(err) = builder.show() {
                tracing::debug!(%err, "could not show notification");
            }
        });
    }
}

const SOUND: &str = "default";

pub fn notify_failure(app: &AppHandle, error: &FixError) {
    show(app, format!("{}\n{}", error.message, error.remedy), false);
}

fn summarise(text: &str) -> String {
    const LIMIT: usize = 120;
    let flattened = text.split_whitespace().collect::<Vec<_>>().join(" ");

    if flattened.chars().count() <= LIMIT {
        return flattened;
    }
    let mut out: String = flattened.chars().take(LIMIT).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_is_shown_whole() {
        assert_eq!(
            summarise("I don't think it's correct."),
            "I don't think it's correct."
        );
    }

    #[test]
    fn long_text_is_trimmed_with_an_ellipsis() {
        let summary = summarise(&"word ".repeat(200));
        assert!(summary.chars().count() <= 121);
        assert!(summary.ends_with('…'));
    }

    #[test]
    fn line_breaks_are_flattened_for_a_single_line_banner() {
        assert_eq!(summarise("first\n\n  second   line"), "first second line");
    }
}
