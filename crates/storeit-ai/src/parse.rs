use storeit_domain::entities::IdentificationResult;
use storeit_domain::errors::{DomainError, Result};

/// Parse an `IdentificationResult` from raw LLM output.
///
/// Handles both raw JSON and JSON wrapped in markdown fences (```json ... ```).
pub fn parse_identification_json(raw: &str) -> Result<IdentificationResult> {
    let trimmed = raw.trim();

    // Try direct parse first
    if let Ok(result) = serde_json::from_str::<IdentificationResult>(trimmed) {
        return Ok(result);
    }

    // Try extracting JSON from markdown fences
    if let Some(json_str) = extract_fenced_json(trimmed)
        && let Ok(result) = serde_json::from_str::<IdentificationResult>(json_str)
    {
        return Ok(result);
    }

    Err(DomainError::Internal(format!(
        "failed to parse identification JSON from AI response: {trimmed}"
    )))
}

fn extract_fenced_json(s: &str) -> Option<&str> {
    // Match ```json\n...\n``` or ```\n...\n```
    let start = if let Some(pos) = s.find("```json") {
        pos + "```json".len()
    } else if let Some(pos) = s.find("```") {
        pos + "```".len()
    } else {
        return None;
    };

    let rest = &s[start..];
    let end = rest.find("```")?;
    Some(rest[..end].trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_JSON: &str = r#"{
        "name": "Red Stapler",
        "category": "office supplies",
        "description": "A classic red Swingline stapler",
        "aliases": ["stapler", "Swingline"],
        "keywords": ["office", "stapler", "red", "desk"],
        "color": "red",
        "material": "plastic and metal",
        "condition_notes": "good condition"
    }"#;

    #[test]
    fn parse_raw_json() {
        let result = parse_identification_json(VALID_JSON).unwrap();
        assert_eq!(result.name, "Red Stapler");
        assert_eq!(result.category.as_deref(), Some("office supplies"));
        assert_eq!(result.color.as_deref(), Some("red"));
        assert_eq!(result.aliases, vec!["stapler", "Swingline"]);
    }

    #[test]
    fn parse_fenced_json() {
        let input = format!("Here is the result:\n```json\n{VALID_JSON}\n```\n");
        let result = parse_identification_json(&input).unwrap();
        assert_eq!(result.name, "Red Stapler");
    }

    #[test]
    fn parse_fenced_no_language_tag() {
        let input = format!("```\n{VALID_JSON}\n```");
        let result = parse_identification_json(&input).unwrap();
        assert_eq!(result.name, "Red Stapler");
    }

    #[test]
    fn parse_invalid_json_returns_error() {
        let result = parse_identification_json("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn parse_with_whitespace() {
        let input = format!("  \n  {VALID_JSON}  \n  ");
        let result = parse_identification_json(&input).unwrap();
        assert_eq!(result.name, "Red Stapler");
    }

    #[test]
    fn parse_minimal_json() {
        let input = r#"{"name": "Mug", "aliases": [], "keywords": []}"#;
        let result = parse_identification_json(input).unwrap();
        assert_eq!(result.name, "Mug");
        assert!(result.category.is_none());
    }
}
