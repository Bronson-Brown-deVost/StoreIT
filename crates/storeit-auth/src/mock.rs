#![cfg(any(test, feature = "test-support"))]

use async_trait::async_trait;

use crate::error::AuthError;
use crate::provider::AuthProvider;
use crate::types::{AuthStartResult, OidcUserInfo};

/// Mock auth provider for integration tests.
/// Returns pre-configured user info regardless of the auth code.
pub struct MockAuthProvider {
    pub user_info: OidcUserInfo,
    pub group_prefix: String,
}

impl MockAuthProvider {
    pub fn new(user_info: OidcUserInfo, group_prefix: impl Into<String>) -> Self {
        Self {
            user_info,
            group_prefix: group_prefix.into(),
        }
    }
}

#[async_trait]
impl AuthProvider for MockAuthProvider {
    fn start_auth(&self) -> Result<AuthStartResult, AuthError> {
        Ok(AuthStartResult {
            authorize_url: "http://mock-auth/authorize?mock=true".into(),
            csrf_state: "mock-csrf-state".into(),
            pkce_verifier: "mock-pkce-verifier".into(),
        })
    }

    async fn exchange_code(
        &self,
        _code: &str,
        _pkce_verifier: &str,
    ) -> Result<OidcUserInfo, AuthError> {
        Ok(self.user_info.clone())
    }

    fn group_prefix(&self) -> &str {
        &self.group_prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_mock() -> MockAuthProvider {
        MockAuthProvider::new(
            OidcUserInfo {
                external_id: "test-user-123".into(),
                email: "test@example.com".into(),
                display_name: "Test User".into(),
                groups: vec!["storeit:family".into(), "storeit:work".into()],
            },
            "storeit:",
        )
    }

    #[test]
    fn mock_start_auth() {
        let mock = test_mock();
        let result = mock.start_auth().unwrap();
        assert!(result.authorize_url.contains("mock"));
        assert_eq!(result.csrf_state, "mock-csrf-state");
        assert_eq!(result.pkce_verifier, "mock-pkce-verifier");
    }

    #[tokio::test]
    async fn mock_exchange_code() {
        let mock = test_mock();
        let info = mock
            .exchange_code("any-code", "any-verifier")
            .await
            .unwrap();
        assert_eq!(info.external_id, "test-user-123");
        assert_eq!(info.email, "test@example.com");
        assert_eq!(info.groups.len(), 2);
    }

    #[test]
    fn mock_group_prefix() {
        let mock = test_mock();
        assert_eq!(mock.group_prefix(), "storeit:");
    }

    #[test]
    fn mock_filter_groups() {
        let mock = test_mock();
        let filtered = mock.filter_groups(&mock.user_info.groups);
        assert_eq!(filtered, vec!["family", "work"]);
    }
}
