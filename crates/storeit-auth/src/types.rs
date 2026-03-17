use serde::{Deserialize, Serialize};

/// Configuration for the auth provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub group_prefix: String,
}

/// User info returned from OIDC provider after successful authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcUserInfo {
    /// Subject identifier from the OIDC provider (unique per user).
    pub external_id: String,
    pub email: String,
    pub display_name: String,
    /// Raw group names from the OIDC provider (before prefix filtering).
    pub groups: Vec<String>,
}

/// Result of initiating an auth flow.
#[derive(Debug, Clone)]
pub struct AuthStartResult {
    /// URL to redirect the user to for authentication.
    pub authorize_url: String,
    /// CSRF token to validate in the callback.
    pub csrf_state: String,
    /// PKCE code verifier to use when exchanging the authorization code.
    pub pkce_verifier: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- AuthConfig serde --

    #[test]
    fn auth_config_serde_roundtrip() {
        let config = AuthConfig {
            issuer_url: "https://auth.example.com".into(),
            client_id: "my-client".into(),
            client_secret: "secret".into(),
            redirect_uri: "http://localhost/callback".into(),
            group_prefix: "storeit:".into(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let back: AuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.issuer_url, "https://auth.example.com");
        assert_eq!(back.client_id, "my-client");
        assert_eq!(back.client_secret, "secret");
        assert_eq!(back.redirect_uri, "http://localhost/callback");
        assert_eq!(back.group_prefix, "storeit:");
    }

    #[test]
    fn auth_config_clone() {
        let config = AuthConfig {
            issuer_url: "https://auth.example.com".into(),
            client_id: "cid".into(),
            client_secret: "sec".into(),
            redirect_uri: "http://localhost/cb".into(),
            group_prefix: "p:".into(),
        };
        let cloned = config.clone();
        assert_eq!(cloned.issuer_url, config.issuer_url);
        assert_eq!(cloned.client_id, config.client_id);
    }

    // -- OidcUserInfo serde --

    #[test]
    fn oidc_user_info_serde_roundtrip() {
        let info = OidcUserInfo {
            external_id: "sub-123".into(),
            email: "user@example.com".into(),
            display_name: "Test User".into(),
            groups: vec!["storeit:family".into(), "other".into()],
        };
        let json = serde_json::to_string(&info).unwrap();
        let back: OidcUserInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.external_id, "sub-123");
        assert_eq!(back.email, "user@example.com");
        assert_eq!(back.display_name, "Test User");
        assert_eq!(back.groups.len(), 2);
    }

    #[test]
    fn oidc_user_info_clone() {
        let info = OidcUserInfo {
            external_id: "sub-456".into(),
            email: "u@e.com".into(),
            display_name: "U".into(),
            groups: vec![],
        };
        let cloned = info.clone();
        assert_eq!(cloned.external_id, info.external_id);
        assert!(cloned.groups.is_empty());
    }

    // -- AuthStartResult --

    #[test]
    fn auth_start_result_clone() {
        let result = AuthStartResult {
            authorize_url: "https://auth.example.com/authorize?client_id=x".into(),
            csrf_state: "csrf-token-abc".into(),
            pkce_verifier: "verifier-xyz".into(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.authorize_url, result.authorize_url);
        assert_eq!(cloned.csrf_state, result.csrf_state);
        assert_eq!(cloned.pkce_verifier, result.pkce_verifier);
    }

    #[test]
    fn auth_start_result_debug() {
        let result = AuthStartResult {
            authorize_url: "https://example.com".into(),
            csrf_state: "state".into(),
            pkce_verifier: "verifier".into(),
        };
        let dbg = format!("{:?}", result);
        assert!(dbg.contains("AuthStartResult"));
        assert!(dbg.contains("https://example.com"));
    }
}
