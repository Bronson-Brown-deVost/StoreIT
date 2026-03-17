# Components

## Key Patterns

All components use Svelte 5 syntax with `$props()` for inputs and `$state`/`$derived` for local state.

### PhotoThumbnail

Displays a single photo thumbnail with a loading spinner.

- Uses `photoThumbnailUrl()` (not the full-size file URL)
- Must use `loading="eager"` — `loading="lazy"` breaks `onLoad` events in scrollable containers
- Shows a placeholder spinner until the image loads

### PhotoGallery

Grid of thumbnails with upload support.

- File input uses `accept="image/*"` but **no `capture` attribute** — adding `capture` blocks access to the photo gallery on mobile, forcing camera-only
- After upload, re-fetches the photo list to update the grid
- Supports setting a primary photo

### CreateDialog

Modal dialog for creating locations, containers, and items.

- Dialog panels need `max-h-[100dvh] overflow-y-auto` + `pb-[env(safe-area-inset-bottom)]` for mobile
- Calls `onCreated` callback after successful creation for the parent to re-fetch
- z-index: dialogs use `z-[60]` (above bottom nav at `z-50`)

### MoveFlow

Multi-step workflow for moving containers or items to a new parent.

- Step 1: Select destination via ParentPicker tree browser
- Step 2: Confirm the move
- Auto-closes and re-fetches parent data on success

### ParentPicker

Tree browser for selecting a location or container as a parent.

- Lazy-loads children on expand
- Highlights the current parent
- Returns a `ParentRef` (location or container ID)

### PhotoLightbox

Full-screen photo viewer.

- z-index: `z-[70]` (above dialogs)
- Swipe navigation between photos
- Shows rotation controls

### PrintLabel

Generates a printable label with QR code.

- Opens in a new window for printing
- Includes entity name, location path, and QR code
- QR code links to `{origin}/nfc/tag?uid={entityType}-{entityId}`

### QrCode

Renders a QR code as inline SVG using `qrcode-generator`.

### z-index Layering

| Layer | z-index | Component |
|-------|---------|-----------|
| Bottom nav | `z-50` | BottomNav |
| Dialogs | `z-[60]` | CreateDialog, MoveFlow |
| Lightbox | `z-[70]` | PhotoLightbox |
