#![forbid(unsafe_code)]

pub mod capabilities;
pub mod clipboard;
pub mod hotkey;
pub mod textio;

#[cfg(target_os = "linux")]
pub mod wayland;

#[cfg(target_os = "linux")]
pub mod portal;

pub use capabilities::{
    Capabilities, CapabilityNote, DisplayServer, Environment, HotkeyBackend, InjectionBackend,
    NoteSeverity,
};
pub use clipboard::{Clipboard, ClipboardError};
pub use hotkey::{Hotkey, HotkeyError, Modifier};
pub use textio::{DesktopTextIo, PlatformError, TextIo};
