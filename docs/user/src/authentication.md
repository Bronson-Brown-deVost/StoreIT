# Authentication Setup

StoreIT supports three authentication modes.

## Mock Auth (Development)

When no `STOREIT_AUTH_ISSUER` is set, StoreIT uses a built-in mock auth provider. This is suitable for development and testing — any username/password is accepted.

## Local Login

StoreIT supports local username/password authentication managed through the admin panel. This is suitable for small deployments without an external identity provider.

## OIDC (Recommended for Production)

For production use, configure an OpenID Connect provider like [Authentik](https://goauthentik.io/), Keycloak, or any OIDC-compliant identity provider.

### Configuration

```bash
STOREIT_AUTH_ISSUER=https://auth.example.com/application/o/storeit/
STOREIT_AUTH_CLIENT_ID=storeit
STOREIT_AUTH_CLIENT_SECRET=your-client-secret
STOREIT_AUTH_REDIRECT_URI=https://storeit.example.com/api/v1/auth/callback
STOREIT_SESSION_SECRET=random-string-at-least-32-characters
```

### Authentik Setup

1. Create a new OAuth2/OpenID Provider in Authentik
2. Set the redirect URI to `https://your-domain/api/v1/auth/callback`
3. Create an Application and link it to the provider
4. Copy the client ID and secret to your StoreIT configuration

### Groups

StoreIT uses OIDC group claims to manage access. Set `STOREIT_AUTH_GROUP_PREFIX` to filter which groups are relevant (default: `storeit:`). Users see inventory for groups they belong to.
