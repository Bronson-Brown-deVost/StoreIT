<script lang="ts">
  import { photoThumbnailUrl } from "~/api";

  let { photoId, alt = "", rotation = 0, class: className = "w-12 h-12", onclick }: {
    photoId: string;
    alt?: string;
    rotation?: number;
    class?: string;
    onclick?: () => void;
  } = $props();

  let loaded = $state(false);
  let error = $state(false);

  let rotateStyle = $derived(rotation ? `transform: rotate(${rotation}deg)` : "");
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class={`bg-surface-hover rounded-lg overflow-hidden ${className} ${onclick ? "cursor-pointer" : ""}`}
  onclick={onclick}
  role={onclick ? "button" : undefined}
>
  {#if !error}
    {#if !loaded}
      <div class="w-full h-full animate-pulse bg-surface-hover"></div>
    {/if}
    <img
      src={photoThumbnailUrl(photoId)}
      alt={alt}
      loading="eager"
      onload={() => (loaded = true)}
      onerror={() => (error = true)}
      class={`w-full h-full object-cover ${loaded ? "" : "hidden"}`}
      style={rotateStyle}
    />
  {:else}
    <div class="w-full h-full flex items-center justify-center text-text-muted">
      <svg
        class="w-6 h-6"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
        <circle cx="8.5" cy="8.5" r="1.5" />
        <polyline points="21 15 16 10 5 21" />
      </svg>
    </div>
  {/if}
</div>
