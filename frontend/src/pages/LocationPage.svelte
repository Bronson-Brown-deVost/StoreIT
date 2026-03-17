<script lang="ts">
  import {
    getLocation,
    getLocationChildren,
    getLocationContainers,
    getLocationItems,
    createLocation,
    createContainer,
    updateLocation,
    uploadPhoto,
  } from "~/api";
  import EntityCard from "~/components/EntityCard.svelte";
  import EmptyState from "~/components/EmptyState.svelte";
  import LoadingSpinner from "~/components/LoadingSpinner.svelte";
  import Breadcrumbs from "~/components/Breadcrumbs.svelte";
  import NfcTagManager from "~/components/NfcTagManager.svelte";
  import PrintLabel from "~/components/PrintLabel.svelte";
  import CreateDialog from "~/components/CreateDialog.svelte";
  import PhotoGallery from "~/components/PhotoGallery.svelte";
  import type { RouteResult } from "@mateothegreat/svelte5-router";

  let { route }: { route: RouteResult } = $props();

  let id = $derived(route?.result?.path?.params?.id as string ?? "");
  let location = $state<any>(null);
  let children = $state<any[]>([]);
  let containers = $state<any[]>([]);
  let items = $state<any[]>([]);
  let loading = $state(true);
  let createType = $state<"location" | "container" | null>(null);
  let showPrint = $state(false);

  async function loadAll() {
    loading = true;
    try {
      const [loc, ch, cont, it] = await Promise.all([
        getLocation(id),
        getLocationChildren(id),
        getLocationContainers(id),
        getLocationItems(id),
      ]);
      location = loc;
      children = ch;
      containers = cont;
      items = it;
    } catch {
      // handle error
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (id) loadAll();
  });

  let breadcrumbItems = $derived(
    location?.parent_id
      ? [{ entity_type: "location", id: location.parent_id, name: "Parent" }]
      : []
  );

  async function handleCreateLocation(name: string, description?: string, _color?: string, latitude?: number, longitude?: number, photo?: File) {
    const loc = await createLocation({ parent_id: id, name, description, latitude, longitude });
    if (photo) await uploadPhoto("location", loc.id, photo);
    children = await getLocationChildren(id);
  }

  async function handleCreateContainer(name: string, description?: string, color?: string, _lat?: number, _lng?: number, photo?: File) {
    const c = await createContainer({ parent_type: "location", parent_id: id, name, description, color });
    if (photo) await uploadPhoto("container", c.id, photo);
    containers = await getLocationContainers(id);
  }

  async function handleCoordsFound(lat: number, lng: number) {
    await updateLocation(id, { latitude: lat, longitude: lng });
    location = await getLocation(id);
  }
</script>

<div class="px-4 pt-[env(safe-area-inset-top)]">
  <div class="pt-4">
    <Breadcrumbs items={breadcrumbItems} current={location?.name} />
  </div>

  {#if loading}
    <LoadingSpinner class="py-12" />
  {:else if location}
    <div class="flex items-center gap-2 mb-1">
      <h1 class="text-2xl font-bold flex-1">{location.name}</h1>
      <button
        onclick={() => createType = "location"}
        class="px-3 py-1.5 text-sm bg-surface hover:bg-surface-hover rounded-lg transition-colors min-h-[36px] flex items-center gap-1"
        title="Add sub-location"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
        Location
      </button>
      <button
        onclick={() => createType = "container"}
        class="px-3 py-1.5 text-sm bg-surface hover:bg-surface-hover rounded-lg transition-colors min-h-[36px] flex items-center gap-1"
        title="Add container"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
        Container
      </button>
      <button
        onclick={() => showPrint = true}
        class="p-2 text-text-muted hover:text-text-secondary transition-colors min-h-[36px] min-w-[36px] flex items-center justify-center"
        title="Print label"
      >
        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 6 2 18 2 18 9" />
          <path d="M6 18H4a2 2 0 01-2-2v-5a2 2 0 012-2h16a2 2 0 012 2v5a2 2 0 01-2 2h-2" />
          <rect x="6" y="14" width="12" height="8" />
        </svg>
      </button>
    </div>

    {#if location.description}
      <p class="text-text-secondary mb-2">{location.description}</p>
    {/if}

    {#if location.latitude != null && location.longitude != null}
      <p class="text-xs text-text-muted mb-2">
        <a
          href={`https://maps.google.com/?q=${location.latitude},${location.longitude}`}
          target="_blank"
          rel="noopener noreferrer"
          class="hover:text-primary underline"
        >
          {location.latitude.toFixed(6)}, {location.longitude.toFixed(6)}
        </a>
      </p>
    {/if}

    <PhotoGallery
      entityType="location"
      entityId={location.id}
      onCoordsFound={handleCoordsFound}
    />

    {#if children.length > 0}
      <h2 class="text-lg font-semibold mt-6 mb-3">Sub-locations</h2>
      <div class="flex flex-col gap-2">
        {#each children as child (child.id)}
          <EntityCard
            href={`/locations/${child.id}`}
            name={child.name}
            description={child.description}
            badge="location"
            entityType="location"
            entityId={child.id}
          />
        {/each}
      </div>
    {/if}

    {#if containers.length > 0}
      <h2 class="text-lg font-semibold mt-6 mb-3">Containers</h2>
      <div class="flex flex-col gap-2">
        {#each containers as c (c.id)}
          <EntityCard
            href={`/containers/${c.id}`}
            name={c.name}
            description={c.description}
            color={c.color}
            badge="container"
            entityType="container"
            entityId={c.id}
          />
        {/each}
      </div>
    {/if}

    {#if items.length > 0}
      <h2 class="text-lg font-semibold mt-6 mb-3">Items</h2>
      <div class="flex flex-col gap-2">
        {#each items as item (item.id)}
          <EntityCard
            href={`/items/${item.id}`}
            name={item.name}
            description={item.description}
            entityType="item"
            entityId={item.id}
          />
        {/each}
      </div>
    {/if}

    {#if !children.length && !containers.length && !items.length}
      <EmptyState message="This location is empty" />
    {/if}

    <div class="mt-6">
      <NfcTagManager entityType="location" entityId={location.id} />
    </div>

    {#if createType === "location"}
      <CreateDialog
        title="New Sub-Location"
        showLocation
        onSubmit={handleCreateLocation}
        onClose={() => createType = null}
      />
    {/if}

    {#if createType === "container"}
      <CreateDialog
        title="New Container"
        showColor
        onSubmit={handleCreateContainer}
        onClose={() => createType = null}
      />
    {/if}

    {#if showPrint}
      <PrintLabel
        name={location.name}
        entityType="location"
        entityId={location.id}
        description={location.description}
        onClose={() => showPrint = false}
      />
    {/if}
  {/if}
</div>
