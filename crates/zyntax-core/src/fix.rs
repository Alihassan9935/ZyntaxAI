use crate::diff::{has_changes, word_diff, DiffSegment};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub enum InputSource {
    #[default]
    Selection,

    Clipboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub enum OutputMode {
    #[default]
    Review,

    Replace,

    Clipboard,

    Append,

    Prepend,
}

impl OutputMode {
    pub fn needs_injection(self) -> bool {
        matches!(
            self,
            OutputMode::Replace | OutputMode::Append | OutputMode::Prepend
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl TokenUsage {
    pub fn total(&self) -> u32 {
        self.input_tokens.saturating_add(self.output_tokens)
    }

    pub fn cost(&self, input_per_million: f64, output_per_million: f64) -> f64 {
        (f64::from(self.input_tokens) * input_per_million
            + f64::from(self.output_tokens) * output_per_million)
            / 1_000_000.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub struct FixOutcome {
    pub original: String,
    pub corrected: String,

    pub diff: Vec<DiffSegment>,

    pub changed: bool,
    pub usage: TokenUsage,
    pub provider: String,
    pub model: String,

    #[ts(type = "number")]
    pub elapsed_ms: u64,
}

impl FixOutcome {
    pub fn new(
        original: String,
        corrected: String,
        usage: TokenUsage,
        provider: String,
        model: String,
        elapsed_ms: u64,
    ) -> Self {
        let diff = word_diff(&original, &corrected);
        Self {
            changed: has_changes(&diff),
            diff,
            original,
            corrected,
            usage,
            provider,
            model,
            elapsed_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_text_is_reported_as_unchanged() {
        let outcome = FixOutcome::new(
            "already correct".to_owned(),
            "already correct".to_owned(),
            TokenUsage::default(),
            "gemini".to_owned(),
            "gemini-2.5-flash".to_owned(),
            120,
        );
        assert!(!outcome.changed);
    }

    #[test]
    fn changed_text_carries_a_diff() {
        let outcome = FixOutcome::new(
            "i dont know".to_owned(),
            "I don't know".to_owned(),
            TokenUsage::default(),
            "gemini".to_owned(),
            "gemini-2.5-flash".to_owned(),
            120,
        );
        assert!(outcome.changed);
        assert!(!outcome.diff.is_empty());
    }

    #[test]
    fn cost_prices_input_and_output_separately() {
        let usage = TokenUsage {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
        };

        assert!((usage.cost(0.10, 0.40) - 0.50).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_usage_costs_nothing() {
        assert_eq!(TokenUsage::default().cost(1.0, 2.0), 0.0);
        assert_eq!(TokenUsage::default().total(), 0);
    }

    #[test]
    fn only_in_place_modes_need_injection() {
        assert!(OutputMode::Replace.needs_injection());
        assert!(OutputMode::Append.needs_injection());
        assert!(OutputMode::Prepend.needs_injection());
        assert!(!OutputMode::Review.needs_injection());
        assert!(!OutputMode::Clipboard.needs_injection());
    }
}
