# AI Identification

StoreIT can use Claude (Anthropic's AI) to automatically identify items from photos.

## How It Works

1. You take a photo of an item
2. The photo is sent to the Claude API
3. Claude analyzes the image and returns:
   - A suggested **name** for the item
   - A **description** of what it is
   - **Tags** for searchability (color, material, category, etc.)
4. You review and confirm or edit the suggestions

## Setup

Set the `STOREIT_ANTHROPIC_API_KEY` environment variable with your Anthropic API key. You can get one at [console.anthropic.com](https://console.anthropic.com).

```bash
STOREIT_ANTHROPIC_API_KEY=sk-ant-api03-...
```

Optionally, configure the model:

```bash
STOREIT_AI_MODEL=claude-haiku-4-5-20251001  # default, fast and cheap
```

## Without AI

If no API key is configured, AI identification is disabled. You can still add items manually with name, description, and tags. All other features work normally.

## Privacy

Photos are sent to Anthropic's API for identification only. They are not stored by Anthropic beyond the API request. All data remains on your server.
