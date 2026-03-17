# Reactivity & State Management

## Svelte 5 Runes

The frontend uses Svelte 5's rune-based reactivity system:

- `$state` — Reactive mutable state
- `$derived` — Computed values that update automatically
- `$effect` — Side effects that run when dependencies change
- `$props` — Component props

### Auth Store

The central auth store (`lib/auth.svelte.ts`) uses a class with rune-based fields:

```typescript
class AuthStore {
  data = $state<MeResponse | null>(null);
  loading = $state(true);
  authMode = $state<"oidc" | "local">("oidc");

  user = $derived(this.data?.user);
  groups = $derived(this.data?.groups ?? []);
  activeGroupId = $derived(this.data?.active_group_id);
}
```

### Data Fetching Pattern

Pages fetch data in `$effect` blocks or use `{#await}` blocks:

```svelte
<script lang="ts">
  let items = $state<Item[]>([]);

  async function loadItems() {
    items = await fetchItemsByContainer(containerId);
  }

  $effect(() => { loadItems(); });

  // After mutations, re-fetch to update the UI
  async function handleDelete(id: string) {
    await deleteItem(id);
    await loadItems();
  }
</script>
```

Always re-fetch after mutations to ensure the UI reflects the current server state.

## Service Worker Caching

The PWA service worker uses different caching strategies for different data types:

| Data | Strategy | Reason |
|------|----------|--------|
| Browse/entity API | NetworkFirst | Mutable — always show latest |
| Auth API | NetworkFirst | Session state changes |
| Search API | NetworkFirst | Results depend on current data |
| Photo files | CacheFirst | Immutable (content-addressed) |
| Static assets | Precache | Bundled at build time |

**Never use `StaleWhileRevalidate`** for mutable data — it serves stale responses first, which causes confusing UI where changes seem to disappear briefly.

## Image Loading

Use `loading="eager"` (not `lazy`) for thumbnails in scrollable containers. Lazy loading breaks `onLoad` events in dynamically-rendered lists, preventing the loading spinner from being replaced.

## E2E Test Isolation

Playwright tests run with `serviceWorkers: "block"` to ensure all requests hit the server directly. This prevents cached responses from masking reactivity bugs.
