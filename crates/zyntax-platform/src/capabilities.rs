use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub enum DisplayServer {
    Windows,
    MacOs,
    X11,
    Wayland,

    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub enum HotkeyBackend {
    Os,

    Portal,

    ExternalCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub enum InjectionBackend {
    Native,

    Wtype,

    Ydotool,

    None,
}

impl InjectionBackend {
    pub fn is_available(self) -> bool {
        !matches!(self, InjectionBackend::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub enum NoteSeverity {
    Info,

    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub struct CapabilityNote {
    pub severity: NoteSeverity,
    pub title: String,

    pub detail: String,

    pub remedy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub struct Capabilities {
    pub display_server: DisplayServer,

    pub can_capture_selection: bool,
    pub injection: InjectionBackend,
    pub hotkey: HotkeyBackend,
    pub notes: Vec<CapabilityNote>,
}

impl Capabilities {
    pub fn can_replace_in_place(&self) -> bool {
        self.injection.is_available()
    }

    pub fn is_degraded(&self) -> bool {
        self.notes
            .iter()
            .any(|note| note.severity == NoteSeverity::Degraded)
    }

    pub fn detect() -> Self {
        Self::from_environment(&Environment::probe())
    }

    pub fn from_environment(env: &Environment) -> Self {
        match env.display_server {
            DisplayServer::Windows => Self {
                display_server: DisplayServer::Windows,
                can_capture_selection: true,
                injection: InjectionBackend::Native,
                hotkey: HotkeyBackend::Os,
                notes: Vec::new(),
            },

            DisplayServer::MacOs => {
                let mut notes = Vec::new();
                if !env.macos_accessibility_granted {
                    notes.push(CapabilityNote {
                        severity: NoteSeverity::Degraded,
                        title: "Accessibility permission not granted".to_owned(),
                        detail: "macOS blocks applications from reading a selection or typing \
                                 into other apps until you allow it. Until then ZyntaxAI can \
                                 only work with text you copy yourself."
                            .to_owned(),
                        remedy: Some(
                            "Open System Settings → Privacy & Security → Accessibility and \
                             enable ZyntaxAI."
                                .to_owned(),
                        ),
                    });
                }
                Self {
                    display_server: DisplayServer::MacOs,
                    can_capture_selection: env.macos_accessibility_granted,
                    injection: if env.macos_accessibility_granted {
                        InjectionBackend::Native
                    } else {
                        InjectionBackend::None
                    },
                    hotkey: HotkeyBackend::Os,
                    notes,
                }
            }

            DisplayServer::X11 => Self {
                display_server: DisplayServer::X11,
                can_capture_selection: true,
                injection: InjectionBackend::Native,
                hotkey: HotkeyBackend::Os,
                notes: Vec::new(),
            },

            DisplayServer::Wayland => {
                let mut notes = Vec::new();

                let injection = if env.has_wtype {
                    InjectionBackend::Wtype
                } else if env.has_ydotool {
                    InjectionBackend::Ydotool
                } else {
                    InjectionBackend::None
                };

                if injection == InjectionBackend::None {
                    notes.push(CapabilityNote {
                        severity: NoteSeverity::Degraded,
                        title: "Cannot type into other applications".to_owned(),
                        detail: "Wayland does not let an application send keystrokes to another \
                                 window. Replace, Append and Prepend are unavailable, so \
                                 corrections are put on your clipboard instead."
                            .to_owned(),
                        remedy: Some(
                            "Install `wtype` to enable them. If your compositor does not \
                             support it, `ydotool` works everywhere but needs its daemon \
                             running."
                                .to_owned(),
                        ),
                    });
                }

                let hotkey = if env.has_global_shortcuts_portal {
                    HotkeyBackend::Portal
                } else {
                    notes.push(CapabilityNote {
                        severity: NoteSeverity::Degraded,
                        title: "Global hotkey unavailable".to_owned(),
                        detail: "This compositor does not implement the desktop portal for \
                                 global shortcuts, so ZyntaxAI cannot register a hotkey itself."
                            .to_owned(),
                        remedy: Some(
                            "Bind a key in your compositor's own settings to the command \
                             `zyntax fix`."
                                .to_owned(),
                        ),
                    });
                    HotkeyBackend::ExternalCommand
                };

                Self {
                    display_server: DisplayServer::Wayland,

                    can_capture_selection: env.has_wlr_data_control,
                    injection,
                    hotkey,
                    notes,
                }
            }

            DisplayServer::Unknown => Self {
                display_server: DisplayServer::Unknown,
                can_capture_selection: false,
                injection: InjectionBackend::None,
                hotkey: HotkeyBackend::ExternalCommand,
                notes: vec![CapabilityNote {
                    severity: NoteSeverity::Degraded,
                    title: "No graphical session detected".to_owned(),
                    detail: "Neither X11 nor Wayland could be found, so hotkeys and text \
                             replacement are unavailable."
                        .to_owned(),
                    remedy: None,
                }],
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub display_server: DisplayServer,
    pub has_wtype: bool,
    pub has_ydotool: bool,
    pub has_global_shortcuts_portal: bool,
    pub has_wlr_data_control: bool,
    pub macos_accessibility_granted: bool,
}

impl Environment {
    pub fn probe() -> Self {
        let display_server = detect_display_server();
        Self {
            has_wtype: binary_exists("wtype"),
            has_ydotool: binary_exists("ydotool"),
            has_global_shortcuts_portal: global_shortcuts_portal_present(),
            has_wlr_data_control: primary_selection_is_readable(display_server),
            macos_accessibility_granted: macos_accessibility_granted(),
            display_server,
        }
    }

    #[cfg(any(test, feature = "test-util"))]
    pub fn for_test(display_server: DisplayServer) -> Self {
        Self {
            display_server,
            has_wtype: false,
            has_ydotool: false,
            has_global_shortcuts_portal: false,
            has_wlr_data_control: false,
            macos_accessibility_granted: true,
        }
    }
}

fn detect_display_server() -> DisplayServer {
    if cfg!(target_os = "windows") {
        return DisplayServer::Windows;
    }
    if cfg!(target_os = "macos") {
        return DisplayServer::MacOs;
    }

    match std::env::var("XDG_SESSION_TYPE").as_deref() {
        Ok("wayland") => return DisplayServer::Wayland,
        Ok("x11") => return DisplayServer::X11,
        _ => {}
    }
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return DisplayServer::Wayland;
    }
    if std::env::var_os("DISPLAY").is_some() {
        return DisplayServer::X11;
    }
    DisplayServer::Unknown
}

fn binary_exists(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(name).is_file())
}

fn global_shortcuts_portal_present() -> bool {
    if !cfg!(target_os = "linux") {
        return false;
    }

    let dirs = [
        "/usr/share/xdg-desktop-portal/portals",
        "/usr/local/share/xdg-desktop-portal/portals",
    ];

    dirs.iter().any(|dir| {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        entries.filter_map(Result::ok).any(|entry| {
            std::fs::read_to_string(entry.path())
                .map(|contents| contents.contains("GlobalShortcuts"))
                .unwrap_or(false)
        })
    })
}

fn primary_selection_is_readable(display_server: DisplayServer) -> bool {
    match display_server {
        DisplayServer::X11 => true,

        DisplayServer::Windows | DisplayServer::MacOs | DisplayServer::Unknown => false,
        DisplayServer::Wayland => {
            let Ok(mut clipboard) = crate::clipboard::Clipboard::new() else {
                return false;
            };
            !matches!(
                clipboard.primary_text(),
                Err(crate::clipboard::ClipboardError::Unavailable(_))
            )
        }
    }
}

fn macos_accessibility_granted() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos_accessibility_client::accessibility::application_is_trusted()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_is_fully_capable() {
        let caps = Capabilities::from_environment(&Environment::for_test(DisplayServer::Windows));
        assert!(caps.can_capture_selection);
        assert!(caps.can_replace_in_place());
        assert_eq!(caps.hotkey, HotkeyBackend::Os);
        assert!(!caps.is_degraded());
    }

    #[test]
    fn x11_is_fully_capable() {
        let caps = Capabilities::from_environment(&Environment::for_test(DisplayServer::X11));
        assert!(caps.can_capture_selection);
        assert_eq!(caps.injection, InjectionBackend::Native);
        assert!(!caps.is_degraded());
    }

    #[test]
    fn bare_wayland_degrades_to_clipboard_only_and_says_so() {
        let caps = Capabilities::from_environment(&Environment::for_test(DisplayServer::Wayland));

        assert!(!caps.can_replace_in_place());
        assert_eq!(caps.injection, InjectionBackend::None);
        assert!(caps.is_degraded());

        let note = caps
            .notes
            .iter()
            .find(|n| n.title.contains("type into"))
            .expect("the user must be told why replace is unavailable");

        assert!(note.remedy.as_ref().is_some_and(|r| r.contains("wtype")));
    }

    #[test]
    fn wayland_with_wtype_can_inject() {
        let env = Environment {
            has_wtype: true,
            has_global_shortcuts_portal: true,
            has_wlr_data_control: true,
            ..Environment::for_test(DisplayServer::Wayland)
        };
        let caps = Capabilities::from_environment(&env);

        assert_eq!(caps.injection, InjectionBackend::Wtype);
        assert!(caps.can_replace_in_place());
        assert_eq!(caps.hotkey, HotkeyBackend::Portal);
        assert!(!caps.is_degraded());
    }

    #[test]
    fn wtype_is_preferred_over_ydotool() {
        let env = Environment {
            has_wtype: true,
            has_ydotool: true,
            ..Environment::for_test(DisplayServer::Wayland)
        };

        assert_eq!(
            Capabilities::from_environment(&env).injection,
            InjectionBackend::Wtype
        );
    }

    #[test]
    fn ydotool_is_used_when_wtype_is_missing() {
        let env = Environment {
            has_ydotool: true,
            ..Environment::for_test(DisplayServer::Wayland)
        };
        assert_eq!(
            Capabilities::from_environment(&env).injection,
            InjectionBackend::Ydotool
        );
    }

    #[test]
    fn wayland_without_a_portal_falls_back_to_an_external_binding() {
        let env = Environment {
            has_wtype: true,
            has_global_shortcuts_portal: false,
            ..Environment::for_test(DisplayServer::Wayland)
        };
        let caps = Capabilities::from_environment(&env);

        assert_eq!(caps.hotkey, HotkeyBackend::ExternalCommand);
        let note = caps
            .notes
            .iter()
            .find(|n| n.title.contains("hotkey"))
            .expect("must explain the missing hotkey");
        assert!(note
            .remedy
            .as_ref()
            .is_some_and(|r| r.contains("zyntax fix")));
    }

    #[test]
    fn wayland_without_data_control_cannot_read_the_selection() {
        let env = Environment {
            has_wlr_data_control: false,
            ..Environment::for_test(DisplayServer::Wayland)
        };
        assert!(!Capabilities::from_environment(&env).can_capture_selection);
    }

    #[test]
    fn macos_without_permission_is_degraded_with_the_exact_steps() {
        let env = Environment {
            macos_accessibility_granted: false,
            ..Environment::for_test(DisplayServer::MacOs)
        };
        let caps = Capabilities::from_environment(&env);

        assert!(!caps.can_replace_in_place());
        assert!(caps.is_degraded());
        assert!(caps.notes[0]
            .remedy
            .as_ref()
            .is_some_and(|r| r.contains("Accessibility")));
    }

    #[test]
    fn macos_with_permission_is_fully_capable() {
        let caps = Capabilities::from_environment(&Environment::for_test(DisplayServer::MacOs));
        assert!(caps.can_replace_in_place());
        assert!(!caps.is_degraded());
    }

    #[test]
    fn a_headless_session_is_degraded_rather_than_pretending_to_work() {
        let caps = Capabilities::from_environment(&Environment::for_test(DisplayServer::Unknown));
        assert!(!caps.can_capture_selection);
        assert!(!caps.can_replace_in_place());
        assert!(caps.is_degraded());
    }

    #[test]
    fn every_degraded_note_has_a_title_and_detail() {
        for server in [
            DisplayServer::Windows,
            DisplayServer::MacOs,
            DisplayServer::X11,
            DisplayServer::Wayland,
            DisplayServer::Unknown,
        ] {
            let caps = Capabilities::from_environment(&Environment::for_test(server));
            for note in &caps.notes {
                assert!(!note.title.is_empty(), "{server:?}");
                assert!(!note.detail.is_empty(), "{server:?}");
            }
        }
    }

    #[test]
    fn primary_selection_availability_is_decided_per_platform() {
        assert!(primary_selection_is_readable(DisplayServer::X11));

        assert!(!primary_selection_is_readable(DisplayServer::Windows));
        assert!(!primary_selection_is_readable(DisplayServer::MacOs));
        assert!(!primary_selection_is_readable(DisplayServer::Unknown));
    }

    #[test]
    fn probing_the_real_session_does_not_panic() {
        let caps = Capabilities::detect();

        assert!(caps.notes.iter().all(|n| !n.title.is_empty()));
    }
}
