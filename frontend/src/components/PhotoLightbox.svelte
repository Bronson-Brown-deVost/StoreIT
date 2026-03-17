<script lang="ts">
  import { photoLargeUrl } from "~/api";

  let { photoId, alt = "", rotation = 0, onClose, onDelete, onRotate, deleting = false, rotating = false }: {
    photoId: string;
    alt?: string;
    rotation?: number;
    onClose: () => void;
    onDelete?: () => void;
    onRotate?: (degrees: number) => void;
    deleting?: boolean;
    rotating?: boolean;
  } = $props();

  let loaded = $state(false);
  let confirmDelete = $state(false);

  let rotateStyle = $derived(rotation ? `transform: rotate(${rotation}deg)` : "");
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  data-testid="photo-lightbox"
  class="fixed inset-0 bg-black/90 z-[70] flex items-center justify-center"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose();
  }}
>
  <div class="absolute top-4 right-4 pt-[env(safe-area-inset-top)] flex items-center gap-2 z-10">
    {#if onRotate && !confirmDelete}
      <button
        onclick={() => onRotate?.(270)}
        disabled={rotating}
        class="text-white/60 hover:text-white min-h-[44px] min-w-[44px] flex items-center justify-center disabled:opacity-30"
        aria-label="Rotate left"
        title="Rotate left"
      >
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M1 4v6h6" />
          <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
        </svg>
      </button>
      <button
        onclick={() => onRotate?.(90)}
        disabled={rotating}
        class="text-white/60 hover:text-white min-h-[44px] min-w-[44px] flex items-center justify-center disabled:opacity-30"
        aria-label="Rotate right"
        title="Rotate right"
      >
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="transform: scaleX(-1)">
          <path d="M1 4v6h6" />
          <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
        </svg>
      </button>
    {/if}
    {#if onDelete}
      {#if confirmDelete}
        <button
          onclick={() => { confirmDelete = false; onDelete?.(); }}
          disabled={deleting}
          class="px-3 py-2 bg-danger text-white rounded-lg text-sm font-medium min-h-[44px] disabled:opacity-50"
        >
          {deleting ? "Deleting..." : "Confirm Delete"}
        </button>
        <button
          onclick={() => (confirmDelete = false)}
          disabled={deleting}
          class="px-3 py-2 bg-white/20 text-white rounded-lg text-sm min-h-[44px]"
        >
          Cancel
        </button>
      {:else}
        <button
          onclick={() => (confirmDelete = true)}
          class="text-white/60 hover:text-danger min-h-[44px] min-w-[44px] flex items-center justify-center"
          aria-label="Delete photo"
          title="Delete photo"
        >
          <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6" />
            <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
          </svg>
        </button>
      {/if}
    {/if}
    <button
      onclick={onClose}
      class="text-white/80 hover:text-white min-h-[44px] min-w-[44px] flex items-center justify-center"
      aria-label="Close"
    >
      <svg class="w-8 h-8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </button>
  </div>

  {#if !loaded}
    <div class="absolute inset-0 flex items-center justify-center">
      <div class="w-8 h-8 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
    </div>
  {/if}

  <img
    src={photoLargeUrl(photoId)}
    alt={alt}
    onload={() => (loaded = true)}
    class="max-w-full max-h-full object-contain p-4"
    style={rotateStyle}
  />
</div>
