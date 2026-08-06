#![forbid(unsafe_code)]

pub mod history;
pub mod paths;
pub mod secrets;
pub mod settings;

pub use history::{FixRecord, History, HistoryError, NewFix, Stats, UsageSummary};
pub use paths::{Paths, PathsError};
pub use secrets::{SecretBackend, SecretError, SecretStore};
pub use settings::{
    AppSettings, AppearanceSettings, BehaviorSettings, SettingsError, SystemSettings, Theme,
    CURRENT_SCHEMA_VERSION, DEFAULT_HOTKEY,
};
