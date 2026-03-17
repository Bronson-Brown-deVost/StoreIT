pub mod error;
pub mod mock;
pub mod mode;
pub mod oidc;
pub mod password;
pub mod provider;
pub mod types;

pub use error::AuthError;
pub use mode::AuthMode;
pub use password::{hash_password, verify_password};
pub use provider::AuthProvider;
pub use types::{AuthConfig, AuthStartResult, OidcUserInfo};

// Re-export mock when test-support feature or test cfg is active
#[cfg(any(test, feature = "test-support"))]
pub use mock::MockAuthProvider;
