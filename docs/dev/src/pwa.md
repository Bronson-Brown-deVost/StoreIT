# PWA & Offline Support

StoreIT is a Progressive Web App that can be installed on any device and works offline for browsing cached data.

## Service Worker

Configured via `vite-plugin-pwa` in `vite.config.ts`.

### Caching Strategies

| Pattern | Strategy | TTL | Why |
|---------|----------|-----|-----|
| `/api/v1/locations/**`, `/api/v1/containers/**`, etc. | NetworkFirst | — | Mutable data, show latest |
| `/api/v1/auth/**` | NetworkFirst | 5 min | Session state |
| `/api/v1/search/**` | NetworkFirst | 5 min | Results change with data |
| `/api/v1/photos/*/file`, `/api/v1/photos/*/thumbnail` | CacheFirst | 30 days | Immutable (content-addressed) |
| Static assets (JS, CSS, HTML) | Precache | — | Bundled at build |

### Why Not StaleWhileRevalidate

`StaleWhileRevalidate` serves a cached response immediately, then updates the cache in the background. For mutable inventory data, this means the UI briefly shows stale data after mutations — items appear to "un-delete" or changes revert momentarily. `NetworkFirst` avoids this by always trying the server first.

## PWA Manifest

```json
{
  "name": "StoreIT",
  "short_name": "StoreIT",
  "display": "standalone",
  "theme_color": "#1e293b",
  "background_color": "#0f172a"
}
```

Icons: 192x192 and 512x512 PNG in `frontend/public/`.

## Offline Queue

Mutations (create, update, delete) made while offline are queued in `lib/offlineQueue.ts` and replayed when connectivity is restored. The queue is stored in memory (not persisted across restarts).

## Image Loading

Thumbnails must use `loading="eager"`. The `loading="lazy"` attribute breaks `onLoad` events in dynamically-rendered scrollable lists, causing the loading spinner to never be replaced by the actual image.

## E2E Testing

Playwright tests block service workers (`serviceWorkers: "block"` in config) to ensure all requests go directly to the server. This prevents cached responses from masking bugs in data flow.
