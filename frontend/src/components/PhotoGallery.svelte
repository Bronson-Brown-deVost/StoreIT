<script lang="ts">
  import { onMount } from "svelte";
  import { getEntityPhotos, uploadPhoto, deletePhoto, rotatePhoto } from "~/api";
  import PhotoThumbnail from "./PhotoThumbnail.svelte";
  import PhotoLightbox from "./PhotoLightbox.svelte";
  import ExifReader from "exifreader";

  let { entityType, entityId, onCoordsFound }: {
    entityType: "location" | "container" | "item";
    entityId: string;
    onCoordsFound?: (lat: number, lng: number) => void;
  } = $props();

  let photos = $state<Array<{ id: string; rotation?: number }>>([]);
  let uploading = $state(false);
  let uploadError = $state<string | null>(null);
  let gpsPrompt = $state<{ lat: number; lng: number } | null>(null);
  let lightboxPhotoId = $state<string | null>(null);
  let deleting = $state(false);
  let fileInput: HTMLInputElement;

  async function fetchPhotos() {
    try {
      photos = await getEntityPhotos(entityType, entityId);
    } catch {
      photos = [];
    }
  }

  function extractGpsFromExif(file: File): Promise<{ lat: number; lng: number } | null> {
    return new Promise((resolve) => {
      const reader = new FileReader();
      reader.onload = () => {
        try {
          const tags = ExifReader.load(reader.result as ArrayBuffer);
          const lat = tags.GPSLatitude;
          const lng = tags.GPSLongitude;
          if (lat && lng && typeof lat.description === "number" && typeof lng.description === "number") {
            const latRef = tags.GPSLatitudeRef?.value as string[] | undefined;
            const lngRef = tags.GPSLongitudeRef?.value as string[] | undefined;
            let latVal = lat.description as number;
            let lngVal = lng.description as number;
            if (latRef?.[0] === "S") latVal = -latVal;
            if (lngRef?.[0] === "W") lngVal = -lngVal;
            resolve({ lat: latVal, lng: lngVal });
          } else {
            resolve(null);
          }
        } catch {
          resolve(null);
        }
      };
      reader.onerror = () => resolve(null);
      reader.readAsArrayBuffer(file);
    });
  }

  async function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    input.value = "";

    uploading = true;
    uploadError = null;
    try {
      await uploadPhoto(entityType, entityId, file);
      await fetchPhotos();

      if (entityType === "location" && onCoordsFound) {
        const coords = await extractGpsFromExif(file);
        if (coords) {
          gpsPrompt = coords;
        }
      }
    } catch (err) {
      uploadError = err instanceof Error ? err.message : "Failed to upload photo";
    } finally {
      uploading = false;
    }
  }

  let rotating = $state(false);

  async function handleRotate(photoId: string, degrees: number) {
    rotating = true;
    try {
      await rotatePhoto(photoId, degrees);
      await fetchPhotos();
    } catch {
      // ignore
    } finally {
      rotating = false;
    }
  }

  async function handleDelete(photoId: string) {
    deleting = true;
    try {
      await deletePhoto(photoId);
      lightboxPhotoId = null;
      await fetchPhotos();
    } catch {
      // ignore
    } finally {
      deleting = false;
    }
  }

  function acceptCoords() {
    if (gpsPrompt && onCoordsFound) {
      onCoordsFound(gpsPrompt.lat, gpsPrompt.lng);
    }
    gpsPrompt = null;
  }

  // Refetch when entityType or entityId changes
  $effect(() => {
    entityType;
    entityId;
    fetchPhotos();
  });
</script>

<div class="mb-4">
  <div class="flex items-center gap-2 mb-2">
    <h3 class="text-sm font-medium text-text-secondary">Photos</h3>
    <button
      onclick={() => fileInput?.click()}
      disabled={uploading}
      class="px-2 py-1 text-xs bg-surface hover:bg-surface-hover rounded transition-colors min-h-[28px] flex items-center gap-1"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="12" y1="5" x2="12" y2="19" />
        <line x1="5" y1="12" x2="19" y2="12" />
      </svg>
      {uploading ? "Uploading..." : "Add Photo"}
    </button>
    <input
      bind:this={fileInput}
      type="file"
      accept="image/*"
      class="hidden"
      onchange={handleFileSelect}
    />
  </div>

  {#if uploadError}
    <div class="p-2 mb-2 bg-danger/10 text-danger rounded-lg text-sm" data-testid="upload-error">
      {uploadError}
    </div>
  {/if}

  {#if photos.length > 0}
    <div class="flex gap-2 overflow-x-auto scrollbar-none -mx-1 px-1">
      {#each photos as photo (photo.id)}
        <PhotoThumbnail
          photoId={photo.id}
          rotation={photo.rotation ?? 0}
          class="w-20 h-20 flex-shrink-0 rounded-lg"
          onclick={() => (lightboxPhotoId = photo.id)}
        />
      {/each}
    </div>
  {/if}

  {#if lightboxPhotoId}
    {@const lightboxRotation = photos.find(p => p.id === lightboxPhotoId)?.rotation ?? 0}
    <PhotoLightbox
      photoId={lightboxPhotoId}
      rotation={lightboxRotation}
      onClose={() => (lightboxPhotoId = null)}
      onDelete={() => handleDelete(lightboxPhotoId!)}
      onRotate={(degrees) => handleRotate(lightboxPhotoId!, degrees)}
      {deleting}
      {rotating}
    />
  {/if}

  {#if gpsPrompt}
    <div class="mt-2 p-3 bg-surface-raised rounded-lg border border-border text-sm">
      <p class="mb-2">
        GPS coordinates found in photo: {gpsPrompt.lat.toFixed(6)}, {gpsPrompt.lng.toFixed(6)}
      </p>
      <div class="flex gap-2">
        <button
          onclick={acceptCoords}
          class="px-3 py-1.5 bg-primary text-white rounded text-xs font-medium min-h-[32px]"
        >
          Use for Location
        </button>
        <button
          onclick={() => (gpsPrompt = null)}
          class="px-3 py-1.5 bg-surface hover:bg-surface-hover rounded text-xs min-h-[32px]"
        >
          Dismiss
        </button>
      </div>
    </div>
  {/if}
</div>
