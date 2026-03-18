<script lang="ts">
  import { photoFileUrl, photoLargeUrl } from "~/api";

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
  let fullSize = $state(false);
  let fullLoaded = $state(false);

  // Pan/zoom state for full-size mode
  let scale = $state(1);
  let translateX = $state(0);
  let translateY = $state(0);
  let isPanning = $state(false);
  let lastPanX = 0;
  let lastPanY = 0;

  // Pinch zoom state
  let initialPinchDist = 0;
  let initialScale = 1;

  let rotateStyle = $derived(rotation ? `rotate(${rotation}deg)` : "");
  let fullTransform = $derived(
    `translate(${translateX}px, ${translateY}px) scale(${scale})${rotation ? ` rotate(${rotation}deg)` : ""}`
  );

  function toggleFullSize() {
    if (fullSize) {
      fullSize = false;
      resetTransform();
    } else {
      fullSize = true;
      fullLoaded = false;
      resetTransform();
    }
  }

  function resetTransform() {
    scale = 1;
    translateX = 0;
    translateY = 0;
  }

  function zoomIn() {
    scale = Math.min(scale * 1.5, 10);
  }

  function zoomOut() {
    scale = Math.max(scale / 1.5, 0.25);
  }

  // Mouse drag to pan
  function onPointerDown(e: PointerEvent) {
    if (e.pointerType === "touch") return;
    isPanning = true;
    lastPanX = e.clientX;
    lastPanY = e.clientY;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!isPanning) return;
    translateX += e.clientX - lastPanX;
    translateY += e.clientY - lastPanY;
    lastPanX = e.clientX;
    lastPanY = e.clientY;
  }

  function onPointerUp() {
    isPanning = false;
  }

  function getTouchDist(touches: TouchList): number {
    const dx = touches[0].clientX - touches[1].clientX;
    const dy = touches[0].clientY - touches[1].clientY;
    return Math.sqrt(dx * dx + dy * dy);
  }

  let touchStartX = 0;
  let touchStartY = 0;
  let touchStartTransX = 0;
  let touchStartTransY = 0;
  let isTouchPanning = false;

  function onTouchStart(e: TouchEvent) {
    if (e.touches.length === 2) {
      e.preventDefault();
      initialPinchDist = getTouchDist(e.touches);
      initialScale = scale;
      isTouchPanning = false;
    } else if (e.touches.length === 1) {
      isTouchPanning = true;
      touchStartX = e.touches[0].clientX;
      touchStartY = e.touches[0].clientY;
      touchStartTransX = translateX;
      touchStartTransY = translateY;
    }
  }

  function onTouchMove(e: TouchEvent) {
    if (e.touches.length === 2) {
      e.preventDefault();
      const dist = getTouchDist(e.touches);
      scale = Math.min(Math.max(initialScale * (dist / initialPinchDist), 0.25), 10);
    } else if (e.touches.length === 1 && isTouchPanning) {
      e.preventDefault();
      translateX = touchStartTransX + (e.touches[0].clientX - touchStartX);
      translateY = touchStartTransY + (e.touches[0].clientY - touchStartY);
    }
  }

  function onTouchEnd(e: TouchEvent) {
    if (e.touches.length < 2) {
      initialPinchDist = 0;
    }
    if (e.touches.length === 0) {
      isTouchPanning = false;
    }
  }

  function onFullSizeLoad() {
    fullLoaded = true;
    scale = 1;
    translateX = 0;
    translateY = 0;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  data-testid="photo-lightbox"
  class="fixed inset-0 bg-black/90 z-[70]"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose();
  }}
>
  <!-- Toolbar — always fixed to viewport top-right -->
  <div class="fixed top-4 right-4 pt-[env(safe-area-inset-top)] flex items-center gap-2 z-[80]">
    {#if onRotate && !confirmDelete && !fullSize}
      <button
        onclick={() => onRotate?.(270)}
        disabled={rotating}
        class="text-white/60 hover:text-white min-h-[44px] min-w-[44px] flex items-center justify-center disabled:opacity-30 bg-black/40 rounded-full"
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
        class="text-white/60 hover:text-white min-h-[44px] min-w-[44px] flex items-center justify-center disabled:opacity-30 bg-black/40 rounded-full"
        aria-label="Rotate right"
        title="Rotate right"
      >
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="transform: scaleX(-1)">
          <path d="M1 4v6h6" />
          <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
        </svg>
      </button>
    {/if}
    {#if fullSize && !confirmDelete}
      <button
        onclick={zoomIn}
        class="text-white/60 hover:text-white min-h-[44px] min-w-[44px] flex items-center justify-center bg-black/40 rounded-full"
        aria-label="Zoom in"
        title="Zoom in"
      >
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
      <button
        onclick={zoomOut}
        class="text-white/60 hover:text-white min-h-[44px] min-w-[44px] flex items-center justify-center bg-black/40 rounded-full"
        aria-label="Zoom out"
        title="Zoom out"
      >
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
    {/if}
    {#if !confirmDelete}
      <button
        onclick={toggleFullSize}
        class="text-white/60 hover:text-white min-h-[44px] min-w-[44px] flex items-center justify-center bg-black/40 rounded-full"
        aria-label={fullSize ? "Fit to screen" : "View full size"}
        title={fullSize ? "Fit to screen" : "View full size"}
      >
        {#if fullSize}
          <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M4 14h6v6" />
            <path d="M20 10h-6V4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        {:else}
          <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M15 3h6v6" />
            <path d="M9 21H3v-6" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        {/if}
      </button>
    {/if}
    {#if onDelete && !fullSize}
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
          class="text-white/60 hover:text-danger min-h-[44px] min-w-[44px] flex items-center justify-center bg-black/40 rounded-full"
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
      class="text-white/80 hover:text-white min-h-[44px] min-w-[44px] flex items-center justify-center bg-black/40 rounded-full"
      aria-label="Close"
    >
      <svg class="w-8 h-8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </button>
  </div>

  {#if fullSize}
    <!-- Full-size mode: pan and zoom via transform -->
    {#if !fullLoaded}
      <div class="absolute inset-0 flex items-center justify-center">
        <div class="w-8 h-8 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
      </div>
    {/if}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="absolute inset-0 overflow-hidden cursor-grab select-none"
      class:cursor-grabbing={isPanning}
      onpointerdown={onPointerDown}
      onpointermove={onPointerMove}
      onpointerup={onPointerUp}
      onpointercancel={onPointerUp}
      ontouchstart={onTouchStart}
      ontouchmove={onTouchMove}
      ontouchend={onTouchEnd}
      role="img"
      aria-label={alt}
    >
      <div class="w-full h-full flex items-center justify-center">
        <img
          src={photoFileUrl(photoId)}
          {alt}
          onload={onFullSizeLoad}
          class="max-w-none max-h-none"
          style="transform: {fullTransform}; transform-origin: center center; touch-action: none;"
          draggable="false"
        />
      </div>
    </div>

    <!-- Zoom level indicator -->
    {#if fullLoaded && scale !== 1}
      <div class="fixed bottom-6 left-1/2 -translate-x-1/2 bg-black/60 text-white/80 text-sm px-3 py-1 rounded-full z-[80]">
        {Math.round(scale * 100)}%
      </div>
    {/if}
  {:else}
    <!-- Large display mode: fit to screen -->
    {#if !loaded}
      <div class="absolute inset-0 flex items-center justify-center">
        <div class="w-8 h-8 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
      </div>
    {/if}
    <div class="absolute inset-0 flex items-center justify-center">
      <img
        src={photoLargeUrl(photoId)}
        {alt}
        onload={() => (loaded = true)}
        class="max-w-full max-h-full object-contain p-4"
        style="transform: {rotateStyle}"
      />
    </div>
  {/if}
</div>
