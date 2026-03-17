use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    Oidc,
    Local,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_mode_serde_oidc() {
        let mode = AuthMode::Oidc;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"oidc\"");
        let back: AuthMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, AuthMode::Oidc);
    }

    #[test]
    fn auth_mode_serde_local() {
        let mode = AuthMode::Local;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"local\"");
        let back: AuthMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, AuthMode::Local);
    }

    #[test]
    fn auth_mode_eq() {
        assert_eq!(AuthMode::Oidc, AuthMode::Oidc);
        assert_eq!(AuthMode::Local, AuthMode::Local);
        assert_ne!(AuthMode::Oidc, AuthMode::Local);
    }

    #[test]
    fn auth_mode_clone_copy() {
        let mode = AuthMode::Local;
        let cloned = mode;
        let copied = mode;
        assert_eq!(cloned, copied);
    }

    #[test]
    fn auth_mode_debug() {
        let s = format!("{:?}", AuthMode::Oidc);
        assert_eq!(s, "Oidc");
    }

    #[test]
    fn auth_mode_deserialize_invalid() {
        let result = serde_json::from_str::<AuthMode>("\"unknown\"");
        assert!(result.is_err());
    }
}
