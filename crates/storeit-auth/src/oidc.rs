use async_trait::async_trait;
use openidconnect::{
    AdditionalClaims, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, IdTokenFields, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RevocationErrorResponseType, Scope, StandardErrorResponse,
    StandardTokenIntrospectionResponse, StandardTokenResponse,
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
        CoreJsonWebKeyType, CoreJsonWebKeyUse, CoreJweContentEncryptionAlgorithm,
        CoreJwsSigningAlgorithm, CoreProviderMetadata, CoreResponseType, CoreRevocableToken,
        CoreTokenType,
    },
    reqwest::async_http_client,
};
use serde::{Deserialize, Serialize};

use crate::error::AuthError;
use crate::provider::AuthProvider;
use crate::types::{AuthConfig, AuthStartResult, OidcUserInfo};

// ---------------------------------------------------------------------------
// Custom additional-claims type so we can extract `groups` from UserInfo / ID token
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GroupClaims {
    #[serde(default)]
    pub groups: Vec<String>,
}

impl AdditionalClaims for GroupClaims {}

// ---------------------------------------------------------------------------
// Client type alias parameterised with GroupClaims instead of EmptyAdditionalClaims
// ---------------------------------------------------------------------------

type StoreITIdTokenFields = IdTokenFields<
    GroupClaims,
    EmptyExtraTokenFields,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
>;

type StoreITTokenResponse = StandardTokenResponse<StoreITIdTokenFields, CoreTokenType>;

type StoreITClient = openidconnect::Client<
    GroupClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
    CoreJsonWebKeyUse,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    StoreITTokenResponse,
    CoreTokenType,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, CoreTokenType>,
    CoreRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
>;

// ---------------------------------------------------------------------------
// OidcProvider
// ---------------------------------------------------------------------------

/// OIDC-based auth provider using openidconnect crate.
pub struct OidcProvider {
    client: StoreITClient,
    group_prefix: String,
}

impl OidcProvider {
    /// Create a new OidcProvider by discovering the OIDC configuration.
    pub async fn new(config: &AuthConfig) -> Result<Self, AuthError> {
        let issuer_url = IssuerUrl::new(config.issuer_url.clone())
            .map_err(|e| AuthError::Config(e.to_string()))?;

        let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
            .await
            .map_err(|e| AuthError::Discovery(e.to_string()))?;

        let redirect_url = RedirectUrl::new(config.redirect_uri.clone())
            .map_err(|e| AuthError::Config(e.to_string()))?;

        let client = StoreITClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
        )
        .set_redirect_uri(redirect_url);

        Ok(Self {
            client,
            group_prefix: config.group_prefix.clone(),
        })
    }
}

#[async_trait]
impl AuthProvider for OidcProvider {
    fn start_auth(&self) -> Result<AuthStartResult, AuthError> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_state, _nonce) = self
            .client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("openid".into()))
            .add_scope(Scope::new("profile".into()))
            .add_scope(Scope::new("email".into()))
            .add_scope(Scope::new("groups".into()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok(AuthStartResult {
            authorize_url: auth_url.to_string(),
            csrf_state: csrf_state.secret().clone(),
            pkce_verifier: pkce_verifier.secret().clone(),
        })
    }

    async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: &str,
    ) -> Result<OidcUserInfo, AuthError> {
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| AuthError::Exchange(e.to_string()))?;

        // Fetch user info from the provider
        let user_info_request = self
            .client
            .user_info(token_response.access_token().clone(), None)
            .map_err(|e| AuthError::UserInfo(e.to_string()))?;

        let user_info: openidconnect::UserInfoClaims<GroupClaims, CoreGenderClaim> =
            user_info_request
                .request_async(async_http_client)
                .await
                .map_err(|e| AuthError::UserInfo(e.to_string()))?;

        let external_id = user_info.subject().to_string();

        let email = user_info.email().map(|e| e.to_string()).unwrap_or_default();

        let display_name = user_info
            .name()
            .and_then(|n| n.get(None))
            .map(|n| n.to_string())
            .unwrap_or_else(|| email.clone());

        // Extract groups from the custom additional claims
        let groups = user_info.additional_claims().groups.clone();

        Ok(OidcUserInfo {
            external_id,
            email,
            display_name,
            groups,
        })
    }

    fn group_prefix(&self) -> &str {
        &self.group_prefix
    }
}

#[cfg(test)]
mod tests {
    use super::GroupClaims;
    use openidconnect::AdditionalClaims;

    #[test]
    fn group_claims_deserialize_with_groups() {
        let json = r#"{"groups":["storeit:family","storeit:work","other"]}"#;
        let claims: GroupClaims = serde_json::from_str(json).unwrap();
        assert_eq!(
            claims.groups,
            vec!["storeit:family", "storeit:work", "other"]
        );
    }

    #[test]
    fn group_claims_deserialize_empty_groups() {
        let json = r#"{"groups":[]}"#;
        let claims: GroupClaims = serde_json::from_str(json).unwrap();
        assert!(claims.groups.is_empty());
    }

    #[test]
    fn group_claims_deserialize_missing_groups_defaults_to_empty() {
        let json = r#"{}"#;
        let claims: GroupClaims = serde_json::from_str(json).unwrap();
        assert!(claims.groups.is_empty());
    }

    #[test]
    fn group_claims_serialize_roundtrip() {
        let claims = GroupClaims {
            groups: vec!["storeit:home".into(), "admin".into()],
        };
        let json = serde_json::to_string(&claims).unwrap();
        let back: GroupClaims = serde_json::from_str(&json).unwrap();
        assert_eq!(back.groups, claims.groups);
    }

    #[test]
    fn group_claims_implements_additional_claims() {
        // Verify the trait impl exists (compile-time check, but run it to be thorough)
        fn assert_additional_claims<T: AdditionalClaims>() {}
        assert_additional_claims::<GroupClaims>();
    }

    #[test]
    fn group_claims_debug() {
        let claims = GroupClaims {
            groups: vec!["g1".into()],
        };
        let dbg = format!("{claims:?}");
        assert!(dbg.contains("GroupClaims"));
        assert!(dbg.contains("g1"));
    }

    #[test]
    fn group_claims_clone() {
        let claims = GroupClaims {
            groups: vec!["a".into(), "b".into()],
        };
        let cloned = claims.clone();
        assert_eq!(cloned.groups, claims.groups);
    }
}
