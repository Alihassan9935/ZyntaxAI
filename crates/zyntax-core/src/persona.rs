use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../apps/desktop/src/lib/bindings/")]
pub struct Persona {
    pub id: String,
    pub name: String,

    pub instruction: String,
    pub builtin: bool,
}

impl Persona {
    fn builtin(id: &str, name: &str, instruction: &str) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            instruction: instruction.to_owned(),
            builtin: true,
        }
    }
}

pub const DEFAULT_PERSONA_ID: &str = "standard";

pub fn builtin_personas() -> Vec<Persona> {
    vec![
        Persona::builtin(
            DEFAULT_PERSONA_ID,
            "Standard",
            "Correct grammar, spelling and punctuation only. Preserve the author's voice, \
             register, vocabulary and sentence structure exactly. Do not rephrase, shorten, \
             expand or reorder anything that is already correct.",
        ),
        Persona::builtin(
            "friendly",
            "Friendly",
            "Correct the text and make it read as warm and conversational. Prefer plain, \
             approachable wording over formal constructions. Keep every fact, request and \
             commitment exactly as the author stated it.",
        ),
        Persona::builtin(
            "professional",
            "Professional",
            "Correct the text and raise it to a polished business register. Remove slang, \
             filler and casual contractions. Keep it direct rather than ornate, and do not \
             introduce claims, hedges or pleasantries the author did not make.",
        ),
        Persona::builtin(
            "concise",
            "Concise",
            "Correct the text and tighten it. Remove redundancy, filler and throat-clearing \
             so it says the same thing in fewer words. Never drop a fact, condition, number \
             or qualifier in the process.",
        ),
        Persona::builtin(
            "creative",
            "Creative",
            "Correct the text and make it more vivid and engaging: stronger verbs, more \
             concrete imagery, better rhythm. Stay truthful to the author's intent and do not \
             invent details.",
        ),
    ]
}

pub fn builtin_persona(id: &str) -> Option<Persona> {
    builtin_personas().into_iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_have_unique_stable_ids() {
        let personas = builtin_personas();
        let mut ids: Vec<&str> = personas.iter().map(|p| p.id.as_str()).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count, "built-in persona ids must be unique");
    }

    #[test]
    fn default_persona_exists_and_is_builtin() {
        let persona = builtin_persona(DEFAULT_PERSONA_ID).expect("default persona must exist");
        assert!(persona.builtin);
    }

    #[test]
    fn every_builtin_is_marked_builtin() {
        assert!(builtin_personas().iter().all(|p| p.builtin));
    }
}
