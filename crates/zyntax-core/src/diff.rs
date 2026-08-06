use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub enum DiffKind {
    Equal,

    Insert,

    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub struct DiffSegment {
    pub kind: DiffKind,
    pub text: String,
}

pub fn word_diff(before: &str, after: &str) -> Vec<DiffSegment> {
    let diff = TextDiff::from_words(before, after);
    let mut segments: Vec<DiffSegment> = Vec::new();

    for change in diff.iter_all_changes() {
        let kind = match change.tag() {
            ChangeTag::Equal => DiffKind::Equal,
            ChangeTag::Insert => DiffKind::Insert,
            ChangeTag::Delete => DiffKind::Delete,
        };

        match segments.last_mut() {
            Some(last) if last.kind == kind => last.text.push_str(change.value()),
            _ => segments.push(DiffSegment {
                kind,
                text: change.value().to_owned(),
            }),
        }
    }

    segments
}

pub fn has_changes(segments: &[DiffSegment]) -> bool {
    segments.iter().any(|s| s.kind != DiffKind::Equal)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(segments: &[DiffSegment], kind: DiffKind) -> String {
        segments
            .iter()
            .filter(|s| s.kind == kind)
            .map(|s| s.text.as_str())
            .collect()
    }

    #[test]
    fn identical_text_yields_only_equal_segments() {
        let segments = word_diff("all good here", "all good here");
        assert!(segments.iter().all(|s| s.kind == DiffKind::Equal));
        assert!(!has_changes(&segments));
    }

    #[test]
    fn a_single_word_fix_is_isolated() {
        let segments = word_diff("i dont know", "i don't know");
        assert!(has_changes(&segments));
        assert!(texts(&segments, DiffKind::Delete).contains("dont"));
        assert!(texts(&segments, DiffKind::Insert).contains("don't"));

        assert!(texts(&segments, DiffKind::Equal).contains("know"));
    }

    #[test]
    fn adjacent_changes_merge_into_one_run() {
        let segments = word_diff("this is bad wrong text", "this is good text");
        let runs = segments
            .iter()
            .filter(|s| s.kind == DiffKind::Delete)
            .count();
        assert_eq!(runs, 1, "consecutive deletions must merge: {segments:?}");
    }

    #[test]
    fn reconstructs_both_sides_exactly() {
        let before = "the quick brown  fox\njumped";
        let after = "The quick brown fox jumps";
        let segments = word_diff(before, after);

        let rebuilt_before: String = segments
            .iter()
            .filter(|s| s.kind != DiffKind::Insert)
            .map(|s| s.text.as_str())
            .collect();
        let rebuilt_after: String = segments
            .iter()
            .filter(|s| s.kind != DiffKind::Delete)
            .map(|s| s.text.as_str())
            .collect();

        assert_eq!(rebuilt_before, before);
        assert_eq!(rebuilt_after, after);
    }

    #[test]
    fn handles_empty_sides() {
        assert!(!has_changes(&word_diff("", "")));
        assert!(has_changes(&word_diff("", "added")));
        assert!(has_changes(&word_diff("removed", "")));
    }

    #[test]
    fn handles_non_ascii() {
        let segments = word_diff("ich hab kein zeit", "ich habe keine Zeit");
        assert!(has_changes(&segments));
        let rebuilt: String = segments
            .iter()
            .filter(|s| s.kind != DiffKind::Delete)
            .map(|s| s.text.as_str())
            .collect();
        assert_eq!(rebuilt, "ich habe keine Zeit");
    }
}
