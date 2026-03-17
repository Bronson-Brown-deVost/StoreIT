<script lang="ts">
  import { route } from "@mateothegreat/svelte5-router";
  import { getEntityPhotos } from "~/api";
  import PhotoThumbnail from "./PhotoThumbnail.svelte";

  let {
    href,
    name,
    description,
    photoId,
    color,
    badge,
    entityType,
    entityId,
  }: {
    href: string;
    name: string;
    description?: string | null;
    photoId?: string | null;
    color?: string | null;
    badge?: string;
    entityType?: string;
    entityId?: string;
  } = $props();

  let autoPhotos = $state<{ id: string; rotation?: number }[] | null>(null);

  let resolvedPhotoId = $derived(photoId ?? autoPhotos?.[0]?.id ?? null);
  let resolvedRotation = $derived(autoPhotos?.[0]?.rotation ?? 0);

  $effect(() => {
    if (entityType && entityId && !photoId) {
      getEntityPhotos(entityType, entityId).then((photos) => {
        autoPhotos = photos;
      });
    }
  });
</script>

<a
  use:route
  {href}
  class="flex items-center gap-3 p-3 bg-surface-raised hover:bg-surface-hover rounded-lg transition-colors min-h-[56px]"
>
  {#if resolvedPhotoId}
    <PhotoThumbnail
      photoId={resolvedPhotoId}
      rotation={resolvedRotation}
      alt={name}
      class="w-12 h-12 flex-shrink-0"
    />
  {:else}
    <div
      class="w-12 h-12 rounded-lg flex-shrink-0 flex items-center justify-center"
      style:background-color={color ?? "#334155"}
    >
      <svg
        class="w-6 h-6 text-white/70"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <path d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
      </svg>
    </div>
  {/if}
  <div class="flex-1 min-w-0">
    <div class="flex items-center gap-2">
      <span class="font-medium truncate">{name}</span>
      {#if badge}
        <span class="text-[10px] uppercase tracking-wider px-1.5 py-0.5 bg-primary/20 text-primary rounded flex-shrink-0">
          {badge}
        </span>
      {/if}
    </div>
    {#if description}
      <p class="text-sm text-text-secondary truncate mt-0.5">
        {description}
      </p>
    {/if}
  </div>
  <svg
    class="w-5 h-5 text-text-muted flex-shrink-0"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
  >
    <polyline points="9 18 15 12 9 6" />
  </svg>
</a>
