use async_trait::async_trait;

use crate::error::AuthError;
use crate::types::{AuthStartResult, OidcUserInfo};

/// Abstract auth provider trait. Implemented by OidcProvider for production
/// and MockAuthProvider for tests.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Start the auth flow. Returns a URL to redirect the user to,
    /// along with CSRF state and PKCE verifier for validation in the callback.
    fn start_auth(&self) -> Result<AuthStartResult, AuthError>;

    /// Exchange an authorization code for user information.
    async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: &str,
    ) -> Result<OidcUserInfo, AuthError>;

    /// The group prefix to filter OIDC groups by (e.g., "storeit:").
    fn group_prefix(&self) -> &str;

    /// Filter raw OIDC group names and strip the prefix.
    /// Default implementation filters by prefix and strips it.
    fn filter_groups(&self, groups: &[String]) -> Vec<String> {
        let prefix = self.group_prefix();
        groups
            .iter()
            .filter(|g| g.starts_with(prefix))
            .map(|g| g[prefix.len()..].to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProvider;

    #[async_trait]
    impl AuthProvider for TestProvider {
        fn start_auth(&self) -> Result<AuthStartResult, AuthError> {
            unimplemented!()
        }

        async fn exchange_code(
            &self,
            _code: &str,
            _pkce_verifier: &str,
        ) -> Result<OidcUserInfo, AuthError> {
            unimplemented!()
        }

        fn group_prefix(&self) -> &str {
            "storeit:"
        }
    }

    #[test]
    fn filter_groups_strips_prefix() {
        let provider = TestProvider;
        let groups = vec![
            "storeit:family".to_string(),
            "other-group".to_string(),
            "storeit:work".to_string(),
        ];
        let filtered = provider.filter_groups(&groups);
        assert_eq!(filtered, vec!["family", "work"]);
    }

    #[test]
    fn filter_groups_empty() {
        let provider = TestProvider;
        let groups = vec!["admin".to_string(), "users".to_string()];
        let filtered = provider.filter_groups(&groups);
        assert!(filtered.is_empty());
    }
}
