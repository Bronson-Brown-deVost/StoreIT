# AI Integration

StoreIT uses Claude (Anthropic's AI) for item identification from photos.

## How It Works

The `POST /api/v1/identify` endpoint accepts a multipart photo upload. The server sends the image to Claude with a prompt asking it to identify the item and return structured metadata.

## Response Format

Claude returns JSON with:
- `name` — suggested item name
- `description` — what the item is
- `tags` — array of searchable keywords (color, material, category, etc.)

## Providers

### Anthropic API (Primary)

Direct HTTP calls to `https://api.anthropic.com/v1/messages` with the image as base64 input. Configured via `STOREIT_ANTHROPIC_API_KEY`.

### Claude CLI (Fallback)

If no API key is set but the `claude` CLI binary is available, StoreIT shells out to it. The path is configurable via `STOREIT_CLAUDE_PATH`.

## Model Selection

The default model is `claude-haiku-4-5-20251001` (fast and inexpensive). Override with `STOREIT_AI_MODEL` for different quality/cost tradeoffs.

## Graceful Degradation

If neither an API key nor CLI is available, the identify endpoint returns an error and the frontend falls back to manual item entry. All other features work normally without AI.
