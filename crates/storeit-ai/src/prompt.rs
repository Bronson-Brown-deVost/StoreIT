pub const SYSTEM_PROMPT: &str = r#"You are an AI assistant that identifies household objects from photos. Analyze the image and return a JSON object with the following fields:

- "name": string (required) — a short, descriptive name for the object
- "category": string or null — a general category (e.g. "electronics", "kitchen", "tools", "clothing", "furniture", "office supplies")
- "description": string or null — a brief description of the object
- "aliases": array of strings — alternative names or common synonyms
- "keywords": array of strings — searchable tags relevant to the object
- "color": string or null — the primary color or color description
- "material": string or null — the primary material (e.g. "plastic", "metal", "wood", "fabric")
- "condition_notes": string or null — any visible condition notes (e.g. "good condition", "minor scratches", "worn")

Return ONLY valid JSON with no additional text, no markdown fences, no explanation."#;

pub fn build_identify_prompt() -> String {
    format!("{SYSTEM_PROMPT}\n\nIdentify the object in this image and return the JSON response.")
}

pub fn build_correction_prompt(correction: &str) -> String {
    format!(
        "{SYSTEM_PROMPT}\n\nA previous identification was incorrect. The user provided this correction: \"{correction}\"\n\nPlease re-identify the object in this image, taking the user's correction into account, and return the JSON response."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identify_prompt_contains_system_prompt() {
        let prompt = build_identify_prompt();
        assert!(prompt.contains("household objects"));
        assert!(prompt.contains("Identify the object"));
    }

    #[test]
    fn correction_prompt_includes_user_correction() {
        let prompt = build_correction_prompt("This is actually a stapler");
        assert!(prompt.contains("household objects"));
        assert!(prompt.contains("This is actually a stapler"));
        assert!(prompt.contains("correction"));
    }
}
