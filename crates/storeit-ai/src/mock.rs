use async_trait::async_trait;
use storeit_domain::entities::IdentificationResult;
use storeit_domain::errors::Result;
use storeit_domain::storage::ItemIdentifier;

/// Mock identifier for testing. Returns a hardcoded result.
#[cfg(feature = "test-support")]
pub struct MockItemIdentifier;

#[cfg(feature = "test-support")]
impl MockItemIdentifier {
    pub fn hardcoded_result() -> IdentificationResult {
        IdentificationResult {
            name: "Red Stapler".into(),
            category: Some("office supplies".into()),
            description: Some("A classic red Swingline stapler".into()),
            aliases: vec!["stapler".into(), "Swingline".into()],
            keywords: vec!["office".into(), "stapler".into(), "red".into()],
            color: Some("red".into()),
            material: Some("plastic and metal".into()),
            condition_notes: Some("good condition".into()),
        }
    }
}

#[cfg(feature = "test-support")]
#[async_trait]
impl ItemIdentifier for MockItemIdentifier {
    async fn identify(&self, _image_data: &[u8], _mime_type: &str) -> Result<IdentificationResult> {
        Ok(Self::hardcoded_result())
    }

    async fn identify_with_correction(
        &self,
        _image_data: &[u8],
        _mime_type: &str,
        correction: &str,
    ) -> Result<IdentificationResult> {
        let mut result = Self::hardcoded_result();
        result.name = format!("Corrected: {correction}");
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardcoded_result_has_expected_fields() {
        let result = MockItemIdentifier::hardcoded_result();
        assert_eq!(result.name, "Red Stapler");
        assert_eq!(result.category.as_deref(), Some("office supplies"));
        assert!(!result.aliases.is_empty());
        assert!(!result.keywords.is_empty());
    }

    #[tokio::test]
    async fn identify_returns_hardcoded() {
        let mock = MockItemIdentifier;
        let result = mock.identify(b"fake-image", "image/jpeg").await.unwrap();
        assert_eq!(result.name, "Red Stapler");
    }

    #[tokio::test]
    async fn identify_with_correction_includes_correction() {
        let mock = MockItemIdentifier;
        let result = mock
            .identify_with_correction(b"fake-image", "image/jpeg", "blue pen")
            .await
            .unwrap();
        assert_eq!(result.name, "Corrected: blue pen");
    }
}
