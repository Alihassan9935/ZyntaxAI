use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Speed {
    Fast,

    #[default]
    Normal,

    Detailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GenerationParams {
    pub temperature: f32,
    pub max_output_tokens: u32,
}

impl Speed {
    pub const fn temperature(self) -> f32 {
        match self {
            Speed::Fast => 0.1,
            Speed::Normal => 0.3,
            Speed::Detailed => 0.5,
        }
    }

    pub const fn base_output_tokens(self) -> u32 {
        match self {
            Speed::Fast => 512,
            Speed::Normal => 1024,
            Speed::Detailed => 2048,
        }
    }

    pub fn params_for(self, input_chars: usize) -> GenerationParams {
        let estimated_input = estimate_tokens(input_chars);

        let needed = estimated_input.saturating_mul(2).saturating_add(256);

        GenerationParams {
            temperature: self.temperature(),
            max_output_tokens: needed.max(self.base_output_tokens()).min(MAX_OUTPUT_TOKENS),
        }
    }
}

pub const MAX_OUTPUT_TOKENS: u32 = 8192;

pub fn estimate_tokens(chars: usize) -> u32 {
    u32::try_from(chars.div_ceil(3)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_input_uses_the_preset_floor() {
        assert_eq!(Speed::Fast.params_for(40).max_output_tokens, 512);
        assert_eq!(Speed::Normal.params_for(40).max_output_tokens, 1024);
        assert_eq!(Speed::Detailed.params_for(40).max_output_tokens, 2048);
    }

    #[test]
    fn long_input_grows_the_budget_past_the_preset() {
        let params = Speed::Fast.params_for(10_000);
        assert!(
            params.max_output_tokens > Speed::Fast.base_output_tokens(),
            "budget must grow with input, got {}",
            params.max_output_tokens
        );
        assert!(params.max_output_tokens >= estimate_tokens(10_000) * 2);
    }

    #[test]
    fn budget_is_capped() {
        assert_eq!(
            Speed::Detailed.params_for(usize::MAX).max_output_tokens,
            MAX_OUTPUT_TOKENS
        );
    }

    #[test]
    fn temperature_increases_with_depth() {
        assert!(Speed::Fast.temperature() < Speed::Normal.temperature());
        assert!(Speed::Normal.temperature() < Speed::Detailed.temperature());
    }
}
