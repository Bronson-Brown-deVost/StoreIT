# Configuration

All configuration is done via environment variables. No config file is needed.

## Core Settings

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite:./data/storeit.db?mode=rwc` | SQLite database path |
| `STOREIT_BIND` | `0.0.0.0:8080` | Server listen address |
| `STOREIT_IMAGE_PATH` | `./data/images` | Image storage directory |

## Authentication

| Variable | Default | Description |
|---|---|---|
| `STOREIT_AUTH_ISSUER` | *(none — uses mock auth)* | OIDC issuer URL |
| `STOREIT_AUTH_CLIENT_ID` | `storeit` | OIDC client ID |
| `STOREIT_AUTH_CLIENT_SECRET` | `changeme` | OIDC client secret |
| `STOREIT_AUTH_REDIRECT_URI` | `http://localhost:8080/api/v1/auth/callback` | OIDC callback URL |
| `STOREIT_AUTH_GROUP_PREFIX` | `storeit:` | OIDC group claim prefix filter |
| `STOREIT_SESSION_SECRET` | *(dev default)* | Cookie signing secret (**change in production**) |
| `STOREIT_SESSION_TTL_HOURS` | `24` | Session lifetime in hours |

## AI Identification

| Variable | Default | Description |
|---|---|---|
| `STOREIT_ANTHROPIC_API_KEY` | *(none)* | Anthropic API key for AI identification |
| `STOREIT_AI_MODEL` | `claude-haiku-4-5-20251001` | Claude model to use |
| `STOREIT_CLAUDE_PATH` | `claude` | Path to Claude CLI binary (fallback) |

AI identification is optional. Without an API key, the identify feature is disabled but all other features work normally.

## Example `.env` File

```bash
DATABASE_URL=sqlite:./data/storeit.db?mode=rwc
STOREIT_BIND=0.0.0.0:8080
STOREIT_IMAGE_PATH=./data/images
STOREIT_SESSION_SECRET=your-random-secret-at-least-32-chars
STOREIT_ANTHROPIC_API_KEY=sk-ant-...
```
