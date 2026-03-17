# Authentication

StoreIT supports three authentication providers, configured via environment variables.

## Provider Selection

| Configuration | Provider |
|---|---|
| `STOREIT_AUTH_ISSUER` set | OIDC provider |
| No issuer, local users exist | Local login |
| No issuer, no local users | Mock provider (dev only) |

## OIDC Flow

1. User visits `/api/v1/auth/login`
2. Server generates PKCE challenge and redirects to OIDC provider
3. User authenticates with provider
4. Provider redirects to `/api/v1/auth/callback` with authorization code
5. Server exchanges code for tokens, validates ID token
6. Server creates session and sets cookie
7. User is redirected to the app

## Group-Based Access

OIDC tokens include group claims. StoreIT filters groups by `STOREIT_AUTH_GROUP_PREFIX` (default: `storeit:`). Users only see inventory for groups they belong to.

## Session Management

Sessions are stored in SQLite with configurable TTL (`STOREIT_SESSION_TTL_HOURS`, default: 24). Session cookies are signed with `STOREIT_SESSION_SECRET`.

## Mock Provider

When no auth is configured, the mock provider accepts any credentials. This is used for development and E2E tests. **Never use in production.**
