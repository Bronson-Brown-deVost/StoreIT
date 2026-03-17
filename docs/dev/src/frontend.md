# Frontend Structure

The frontend is a Svelte 5 single-page application built with Vite and Tailwind CSS, deployed as a Progressive Web App.

## Directory Layout

```
frontend/src/
  api/
    client.ts          # HTTP client with auth cookie handling
    index.ts           # All API functions (fetchLocations, createItem, etc.)
    types.ts           # TypeScript types matching backend DTOs
  components/
    BottomNav.svelte       # Tab navigation bar
    Breadcrumbs.svelte     # Location path breadcrumbs
    CreateDialog.svelte    # Modal for creating entities
    EntityCard.svelte      # Reusable card for lists
    MoveFlow.svelte        # Multi-step move workflow
    ParentPicker.svelte    # Tree browser for selecting parents
    PhotoGallery.svelte    # Photo grid with upload
    PhotoLightbox.svelte   # Full-screen photo viewer
    PhotoThumbnail.svelte  # Single thumbnail with loading state
    PrintLabel.svelte      # QR code print label
    QrCode.svelte          # QR code SVG generator
    NfcTagManager.svelte   # NFC tag assignment UI
    SearchBar.svelte       # Search input with navigation
    ...
  lib/
    auth.svelte.ts     # Auth store using Svelte 5 runes
    offlineQueue.ts    # Queue for offline mutations
  pages/
    HomePage.svelte         # Location browser (root view)
    LocationPage.svelte     # Location detail + children
    ContainerPage.svelte    # Container detail + children
    ItemDetailPage.svelte   # Item detail + photos + NFC
    SearchPage.svelte       # Full-text search results
    AddItemPage.svelte      # AI-powered item creation
    BatchAddItemPage.svelte # Batch item creation
    AdminPage.svelte        # User/group management + backup
    SettingsPage.svelte     # Image storage settings
    NfcResolvePage.svelte   # NFC tag resolution landing
  styles/
    index.css          # Tailwind imports
  App.svelte           # Root component with router
  main.ts              # Entry point
```

## Routing

The app uses `@mateothegreat/svelte5-router` for client-side routing:

| Path | Page |
|------|------|
| `/` | HomePage |
| `/locations/:id` | LocationPage |
| `/containers/:id` | ContainerPage |
| `/items/:id` | ItemDetailPage |
| `/search` | SearchPage |
| `/add` | AddItemPage |
| `/add/batch` | BatchAddItemPage |
| `/settings` | SettingsPage |
| `/admin` | AdminPage |
| `/nfc/:tagUri` | NfcResolvePage |

## Path Alias

The `~` alias maps to `src/`, configured in `vite.config.ts` and `tsconfig.json`. Use it for imports:

```typescript
import { fetchLocations } from "~/api";
```

## API Client

All API functions are in `frontend/src/api/index.ts`. They use the HTTP client from `client.ts` which handles auth cookies and base URL resolution. The Vite dev server proxies `/api` requests to the Rust backend on port 8080.
