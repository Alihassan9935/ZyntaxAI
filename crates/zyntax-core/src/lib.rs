#![forbid(unsafe_code)]

pub mod diff;
pub mod fix;
pub mod language;
pub mod persona;
pub mod postprocess;
pub mod prompt;
pub mod provider;
pub mod speed;

pub use diff::{word_diff, DiffKind, DiffSegment};
pub use fix::{FixOutcome, InputSource, OutputMode, TokenUsage};
pub use language::{auto_language, builtin_languages, Language, AUTO_TAG};
pub use persona::{builtin_persona, builtin_personas, Persona, DEFAULT_PERSONA_ID};
pub use postprocess::clean_model_output;
pub use prompt::{Prompt, PromptError, PromptSpec, MAX_INPUT_CHARS};
pub use provider::{ModelPricing, ProviderId, ProviderProfile};
pub use speed::{GenerationParams, Speed, MAX_OUTPUT_TOKENS};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
