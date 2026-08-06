use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub struct Language {
    pub tag: String,

    pub label: String,
    pub builtin: bool,
}

impl Language {
    fn builtin(tag: &str, label: &str) -> Self {
        Self {
            tag: tag.to_owned(),
            label: label.to_owned(),
            builtin: true,
        }
    }

    pub fn is_auto(&self) -> bool {
        self.tag == AUTO_TAG
    }
}

pub const AUTO_TAG: &str = "auto";

pub fn auto_language() -> Language {
    Language::builtin(AUTO_TAG, "Detect automatically")
}

pub fn builtin_languages() -> Vec<Language> {
    vec![
        auto_language(),
        Language::builtin("en", "English"),
        Language::builtin("de", "German"),
        Language::builtin("fr", "French"),
        Language::builtin("es", "Spanish"),
        Language::builtin("it", "Italian"),
        Language::builtin("pt", "Portuguese"),
        Language::builtin("nl", "Dutch"),
        Language::builtin("pl", "Polish"),
        Language::builtin("ru", "Russian"),
        Language::builtin("tr", "Turkish"),
        Language::builtin("ja", "Japanese"),
        Language::builtin("ko", "Korean"),
        Language::builtin("zh", "Chinese (Simplified)"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_is_first_and_detected() {
        let languages = builtin_languages();
        assert!(languages[0].is_auto());
        assert_eq!(languages.iter().filter(|l| l.is_auto()).count(), 1);
    }

    #[test]
    fn tags_are_unique() {
        let languages = builtin_languages();
        let mut tags: Vec<&str> = languages.iter().map(|l| l.tag.as_str()).collect();
        tags.sort_unstable();
        let count = tags.len();
        tags.dedup();
        assert_eq!(tags.len(), count);
    }
}
