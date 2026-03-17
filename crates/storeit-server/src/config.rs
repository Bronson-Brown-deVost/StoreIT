use std::path::PathBuf;

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("storeit")
}

pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub image_storage_path: String,

    // Auth / OIDC
    /// When `None` the server uses `MockAuthProvider` (dev mode).
    pub auth_issuer_url: Option<String>,
    pub auth_client_id: String,
    pub auth_client_secret: String,
    pub auth_redirect_uri: String,
    /// Prefix used to filter OIDC group claims (e.g. "storeit:").
    pub auth_group_prefix: String,
    /// Secret key used to sign the PKCE cookie during the auth flow.
    pub session_secret: String,
    /// Session time-to-live in hours (default 24).
    pub session_ttl_hours: u64,

    // Local auth admin seeding
    pub admin_username: String,
    pub admin_password: String,
    pub admin_email: String,
    pub admin_display_name: String,

    // AI identification
    /// Anthropic API key. When set, the direct API backend is used.
    pub anthropic_api_key: Option<String>,
    /// AI model to use (default: claude-haiku-4-5-20251001).
    pub ai_model: String,
    /// Path to the `claude` CLI binary (default: "claude").
    pub claude_code_path: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Simple CLI arg parsing for --anthropic-api-key
        let cli_api_key = Self::parse_cli_api_key();

        Ok(Self {
            bind_addr: std::env::var("STOREIT_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                let db_path = default_data_dir().join("storeit.db");
                format!("sqlite:{}?mode=rwc", db_path.display())
            }),
            image_storage_path: std::env::var("STOREIT_IMAGE_PATH").unwrap_or_else(|_| {
                default_data_dir()
                    .join("images")
                    .to_string_lossy()
                    .into_owned()
            }),

            // Auth
            auth_issuer_url: std::env::var("STOREIT_AUTH_ISSUER").ok(),
            auth_client_id: std::env::var("STOREIT_AUTH_CLIENT_ID")
                .unwrap_or_else(|_| "storeit".into()),
            auth_client_secret: std::env::var("STOREIT_AUTH_CLIENT_SECRET")
                .unwrap_or_else(|_| "changeme".into()),
            auth_redirect_uri: std::env::var("STOREIT_AUTH_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:8080/api/v1/auth/callback".into()),
            auth_group_prefix: std::env::var("STOREIT_AUTH_GROUP_PREFIX")
                .unwrap_or_else(|_| "storeit:".into()),
            session_secret: std::env::var("STOREIT_SESSION_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-me-in-production-32ch".into()),
            session_ttl_hours: std::env::var("STOREIT_SESSION_TTL_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),

            // Local auth admin seeding
            admin_username: std::env::var("STOREIT_ADMIN_USERNAME")
                .unwrap_or_else(|_| "admin".into()),
            admin_password: std::env::var("STOREIT_ADMIN_PASSWORD")
                .unwrap_or_else(|_| "changeme".into()),
            admin_email: std::env::var("STOREIT_ADMIN_EMAIL")
                .unwrap_or_else(|_| "admin@localhost".into()),
            admin_display_name: std::env::var("STOREIT_ADMIN_DISPLAY_NAME")
                .unwrap_or_else(|_| "Administrator".into()),

            // AI
            anthropic_api_key: cli_api_key
                .or_else(|| std::env::var("STOREIT_ANTHROPIC_API_KEY").ok()),
            ai_model: std::env::var("STOREIT_AI_MODEL")
                .unwrap_or_else(|_| "claude-haiku-4-5-20251001".into()),
            claude_code_path: std::env::var("STOREIT_CLAUDE_PATH")
                .unwrap_or_else(|_| "claude".into()),
        })
    }

    fn parse_cli_api_key() -> Option<String> {
        let args: Vec<String> = std::env::args().collect();
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            if arg == "--anthropic-api-key" {
                return iter.next().cloned();
            }
            if let Some(val) = arg.strip_prefix("--anthropic-api-key=") {
                return Some(val.to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_env_defaults() {
        // Clear env vars to test defaults
        unsafe {
            std::env::remove_var("STOREIT_BIND");
            std::env::remove_var("DATABASE_URL");
            std::env::remove_var("STOREIT_IMAGE_PATH");
            std::env::remove_var("STOREIT_AUTH_ISSUER");
            std::env::remove_var("STOREIT_AUTH_CLIENT_ID");
            std::env::remove_var("STOREIT_ANTHROPIC_API_KEY");
            std::env::remove_var("STOREIT_AI_MODEL");
            std::env::remove_var("STOREIT_CLAUDE_PATH");
        }
        let config = Config::from_env().unwrap();
        assert_eq!(config.bind_addr, "0.0.0.0:8080");

        // Should use platform data dir, not relative paths
        let data_dir = default_data_dir();
        let expected_db = format!("sqlite:{}?mode=rwc", data_dir.join("storeit.db").display());
        let expected_images = data_dir.join("images").to_string_lossy().into_owned();
        assert_eq!(config.database_url, expected_db);
        assert_eq!(config.image_storage_path, expected_images);

        assert!(config.auth_issuer_url.is_none());
        assert_eq!(config.auth_client_id, "storeit");
        assert_eq!(config.auth_group_prefix, "storeit:");
        assert_eq!(config.session_ttl_hours, 24);
        assert!(config.anthropic_api_key.is_none());
        assert_eq!(config.ai_model, "claude-haiku-4-5-20251001");
        assert_eq!(config.claude_code_path, "claude");
    }
}
