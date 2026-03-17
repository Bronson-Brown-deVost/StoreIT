use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("OIDC discovery failed: {0}")]
    Discovery(String),

    #[error("token exchange failed: {0}")]
    Exchange(String),

    #[error("failed to fetch user info: {0}")]
    UserInfo(String),

    #[error("invalid CSRF state")]
    InvalidState,

    #[error("auth configuration error: {0}")]
    Config(String),

    #[error("invalid credentials")]
    InvalidCredentials,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        assert!(
            AuthError::Discovery("timeout".into())
                .to_string()
                .contains("timeout")
        );
        assert!(
            AuthError::Exchange("bad code".into())
                .to_string()
                .contains("bad code")
        );
        assert!(
            AuthError::UserInfo("unreachable".into())
                .to_string()
                .contains("unreachable")
        );
        assert!(AuthError::InvalidState.to_string().contains("CSRF"));
        assert!(
            AuthError::Config("missing".into())
                .to_string()
                .contains("missing")
        );
    }
}
