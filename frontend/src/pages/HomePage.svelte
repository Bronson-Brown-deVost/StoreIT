<script lang="ts">
  import { onMount } from "svelte";
  import { goto, route } from "@mateothegreat/svelte5-router";
  import { getLocationTree, createLocation, uploadPhoto, type LocationTreeNode } from "~/api";
  import EntityCard from "~/components/EntityCard.svelte";
  import EmptyState from "~/components/EmptyState.svelte";
  import LoadingSpinner from "~/components/LoadingSpinner.svelte";
  import CreateDialog from "~/components/CreateDialog.svelte";

  let tree = $state<LocationTreeNode[] | null>(null);
  let loading = $state(true);
  let showCreate = $state(false);

  async function loadTree() {
    loading = true;
    try {
      tree = await getLocationTree();
    } catch {
      tree = null;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadTree();
  });

  async function handleCreate(name: string, description?: string, _color?: string, latitude?: number, longitude?: number, photo?: File) {
    const loc = await createLocation({ name, description, latitude, longitude });
    if (photo) await uploadPhoto("location", loc.id, photo);
    await loadTree();
  }
</script>

<div class="px-4 pt-[env(safe-area-inset-top)]">
  <div class="pt-4 pb-2">
    <h1 class="text-2xl font-bold">StoreIT</h1>
  </div>

  <button
    onclick={() => goto("/search")}
    class="w-full flex items-center gap-3 px-4 py-3 bg-surface-raised rounded-lg text-text-muted mb-6 min-h-[44px]"
  >
    <svg
      class="w-5 h-5"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
    </svg>
    Search items...
  </button>

  {#if loading}
    <LoadingSpinner class="py-12" />
  {:else if tree && tree.length > 0}
    <div class="flex items-center justify-between mb-3">
      <h2 class="text-lg font-semibold">Locations</h2>
      <button
        onclick={() => showCreate = true}
        class="p-2 text-text-muted hover:text-text-secondary transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
        title="Add location"
      >
        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
    </div>
    <div class="flex flex-col gap-2">
      {#each tree as node (node.id)}
        <EntityCard
          href={`/locations/${node.id}`}
          name={node.name}
          description={node.description}
          entityType="location"
          entityId={node.id}
        />
      {/each}
    </div>
  {:else}
    <EmptyState
      message="No locations yet"
      actionLabel="Add Location"
      onAction={() => showCreate = true}
    />
  {/if}

  <a
    use:route
    href="/add"
    class="fixed bottom-24 right-4 w-14 h-14 bg-primary hover:bg-primary-hover text-white rounded-full flex items-center justify-center shadow-lg transition-colors z-40"
    aria-label="Add Item"
  >
    <svg
      class="w-7 h-7"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2.5"
    >
      <line x1="12" y1="5" x2="12" y2="19" />
      <line x1="5" y1="12" x2="19" y2="12" />
    </svg>
  </a>

  {#if showCreate}
    <CreateDialog
      title="New Location"
      showLocation
      onSubmit={handleCreate}
      onClose={() => showCreate = false}
    />
  {/if}
</div>
