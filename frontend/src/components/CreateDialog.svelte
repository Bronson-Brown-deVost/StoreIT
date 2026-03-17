<script lang="ts">
  let { title, onSubmit, onClose, showColor = false, showLocation = false }: {
    title: string;
    onSubmit: (name: string, description?: string, color?: string, latitude?: number, longitude?: number, photo?: File) => Promise<void>;
    onClose: () => void;
    showColor?: boolean;
    showLocation?: boolean;
  } = $props();

  let name = $state("");
  let description = $state("");
  let color = $state("");
  let coords = $state<{ lat: number; lng: number } | null>(null);
  let geoLoading = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let photo = $state<File | null>(null);
  let photoPreview = $state<string | null>(null);

  let fileInput: HTMLInputElement;

  function getLocation() {
    if (!navigator.geolocation) return;
    geoLoading = true;
    navigator.geolocation.getCurrentPosition(
      (pos) => {
        coords = { lat: pos.coords.latitude, lng: pos.coords.longitude };
        geoLoading = false;
      },
      () => {
        geoLoading = false;
      },
      { enableHighAccuracy: true, timeout: 10000 },
    );
  }

  function handlePhotoSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    photo = file;
    photoPreview = URL.createObjectURL(file);
  }

  function clearPhoto() {
    if (photoPreview) URL.revokeObjectURL(photoPreview);
    photo = null;
    photoPreview = null;
    if (fileInput) fileInput.value = "";
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    const n = name.trim();
    if (!n) return;

    loading = true;
    error = null;

    try {
      await onSubmit(
        n,
        description.trim() || undefined,
        showColor && color.trim() ? color.trim() : undefined,
        coords?.lat,
        coords?.lng,
        photo ?? undefined,
      );
      onClose();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to create";
    } finally {
      loading = false;
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 bg-black/60 z-[60] flex items-end sm:items-center justify-center"
  onclick={(e) => {
    if (e.target === e.currentTarget && !loading) onClose();
  }}
>
  <div class="bg-surface-raised w-full max-w-lg rounded-t-2xl sm:rounded-2xl p-6 pb-[env(safe-area-inset-bottom)] max-h-[100dvh] overflow-y-auto">
    <h2 class="text-lg font-semibold mb-4">{title}</h2>

    {#if error}
      <div class="p-3 bg-danger/10 text-danger rounded-lg text-sm mb-4">
        {error}
      </div>
    {/if}

    <form onsubmit={handleSubmit} class="space-y-4">
      <div>
        <label class="block text-sm text-text-secondary mb-1">Name
        <!-- svelte-ignore a11y_autofocus -->
        <input
          type="text"
          value={name}
          oninput={(e) => (name = e.currentTarget.value)}
          class="w-full px-3 py-2 bg-surface rounded-lg border border-border focus:border-primary focus:outline-none min-h-[44px]"
          placeholder="Enter name"
          autofocus
          required
        />
        </label>
      </div>

      <div>
        <label class="block text-sm text-text-secondary mb-1">
          Description (optional)
        <input
          type="text"
          value={description}
          oninput={(e) => (description = e.currentTarget.value)}
          class="w-full px-3 py-2 bg-surface rounded-lg border border-border focus:border-primary focus:outline-none min-h-[44px]"
          placeholder="Enter description"
        />
        </label>
      </div>

      <!-- Photo upload -->
      <div>
        <label class="block text-sm text-text-secondary mb-1">
          Photo (optional)
        <input
          bind:this={fileInput}
          type="file"
          accept="image/*"
          class="hidden"
          onchange={handlePhotoSelect}
        />
        </label>
        {#if photoPreview}
          <div class="flex items-center gap-3">
            <img src={photoPreview} class="w-16 h-16 rounded-lg object-cover" alt="Preview" />
            <button
              type="button"
              onclick={clearPhoto}
              class="text-text-muted hover:text-text-secondary text-sm"
            >
              Remove
            </button>
          </div>
        {:else}
          <button
            type="button"
            onclick={() => fileInput?.click()}
            class="px-3 py-2 bg-surface hover:bg-surface-hover rounded-lg border border-border text-sm min-h-[44px] flex items-center gap-2"
          >
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
              <circle cx="8.5" cy="8.5" r="1.5" />
              <polyline points="21 15 16 10 5 21" />
            </svg>
            Add Photo
          </button>
        {/if}
      </div>

      {#if showColor}
        <div>
          <label class="block text-sm text-text-secondary mb-1">
            Color (optional)
          <div class="flex gap-2 items-center">
            <input
              type="color"
              value={color || "#3b82f6"}
              oninput={(e) => (color = e.currentTarget.value)}
              class="w-10 h-10 rounded cursor-pointer border border-border"
            />
            <input
              type="text"
              value={color}
              oninput={(e) => (color = e.currentTarget.value)}
              class="flex-1 px-3 py-2 bg-surface rounded-lg border border-border focus:border-primary focus:outline-none min-h-[44px]"
              placeholder="#3b82f6"
            />
          </div>
          </label>
        </div>
      {/if}

      {#if showLocation}
        <div>
          <span class="block text-sm text-text-secondary mb-1">
            GPS Location (optional)
          </span>
          {#if coords}
            <div class="flex items-center gap-2 text-sm">
              <svg class="w-4 h-4 text-success" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0118 0z" />
                <circle cx="12" cy="10" r="3" />
              </svg>
              <span>{coords.lat.toFixed(6)}, {coords.lng.toFixed(6)}</span>
              <button
                type="button"
                onclick={() => (coords = null)}
                aria-label="Remove photo"
                class="text-text-muted hover:text-text-secondary ml-auto"
              >
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>
          {:else}
            <button
              type="button"
              onclick={getLocation}
              disabled={geoLoading}
              class="px-3 py-2 bg-surface hover:bg-surface-hover rounded-lg border border-border text-sm min-h-[44px] flex items-center gap-2"
            >
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0118 0z" />
                <circle cx="12" cy="10" r="3" />
              </svg>
              {geoLoading ? "Getting location..." : "Use My Location"}
            </button>
          {/if}
        </div>
      {/if}

      <div class="flex gap-3 pt-2">
        <button
          type="button"
          onclick={onClose}
          disabled={loading}
          class="flex-1 py-3 bg-surface hover:bg-surface-hover rounded-lg transition-colors min-h-[44px]"
        >
          Cancel
        </button>
        <button
          type="submit"
          disabled={loading || !name.trim()}
          class="flex-1 py-3 bg-primary hover:bg-primary-hover text-white font-semibold rounded-lg transition-colors min-h-[44px] disabled:opacity-50"
        >
          {loading ? "Creating..." : "Create"}
        </button>
      </div>
    </form>
  </div>
</div>
