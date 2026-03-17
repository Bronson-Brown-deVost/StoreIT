<script lang="ts">
  import {
    getLocationTree,
    getLocationContainers,
    type LocationTreeNode,
    type ContainerResponse,
  } from "~/api";
  import LoadingSpinner from "./LoadingSpinner.svelte";
  import LocationNodePicker from "./LocationNodePicker.svelte";

  export interface SelectedParent {
    type: "location" | "container";
    id: string;
    name: string;
  }

  let { onSelect, onClose }: {
    onSelect: (parent: SelectedParent) => void;
    onClose: () => void;
  } = $props();

  let tree = $state<LocationTreeNode[]>([]);
  let treeLoading = $state(true);
  let locationId = $state<string | null>(null);
  let containers = $state<ContainerResponse[]>([]);
  let containersLoading = $state(false);
  let filter = $state("");

  let filteredTree = $derived.by(() => {
    const q = filter.toLowerCase();
    if (!q) return tree;
    const filterNodes = (nodes: LocationTreeNode[]): LocationTreeNode[] =>
      nodes
        .map((n) => ({
          ...n,
          children: filterNodes(n.children),
        }))
        .filter(
          (n) =>
            n.name.toLowerCase().includes(q) || n.children.length > 0,
        );
    return filterNodes(tree);
  });

  // Fetch the location tree on mount
  $effect(() => {
    treeLoading = true;
    getLocationTree().then((data) => {
      tree = data;
      treeLoading = false;
    }).catch(() => {
      tree = [];
      treeLoading = false;
    });
  });

  // Fetch containers when locationId changes
  $effect(() => {
    const id = locationId;
    if (id) {
      containersLoading = true;
      getLocationContainers(id).then((data) => {
        containers = data;
        containersLoading = false;
      }).catch(() => {
        containers = [];
        containersLoading = false;
      });
    } else {
      containers = [];
    }
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 bg-black/60 z-[60] flex items-end sm:items-center justify-center"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose();
  }}
>
  <div class="bg-surface-raised w-full max-w-lg max-h-[80vh] rounded-t-2xl sm:rounded-2xl overflow-hidden flex flex-col">
    <div class="flex items-center justify-between px-4 py-3 border-b border-border">
      <h2 class="text-lg font-semibold">Select Parent</h2>
      <button
        onclick={onClose}
        aria-label="Clear filter"
        class="min-h-[44px] min-w-[44px] flex items-center justify-center text-text-muted hover:text-text-secondary"
      >
        <svg
          class="w-6 h-6"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <div class="px-4 py-2">
      <input
        type="text"
        value={filter}
        oninput={(e) => (filter = e.currentTarget.value)}
        placeholder="Filter..."
        class="w-full px-3 py-2 bg-surface rounded-lg text-text-primary placeholder:text-text-muted outline-none focus:ring-2 focus:ring-primary/50 min-h-[44px]"
      />
    </div>

    <div class="overflow-y-auto flex-1 px-4 pb-4 pb-[env(safe-area-inset-bottom)]">
      {#if treeLoading}
        <LoadingSpinner class="py-8" />
      {:else}
        {#if locationId}
          <button
            onclick={() => (locationId = null)}
            class="flex items-center gap-2 py-2 text-primary min-h-[44px]"
          >
            <svg
              class="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="15 18 9 12 15 6" />
            </svg>
            Back to locations
          </button>

          {#if containersLoading}
            <LoadingSpinner class="py-4" />
          {:else if containers.length === 0}
            <p class="text-text-muted py-2">No containers here</p>
          {:else}
            {#each containers as c (c.id)}
              <button
                onclick={() =>
                  onSelect({
                    type: "container",
                    id: c.id,
                    name: c.name,
                  })
                }
                class="flex items-center gap-3 w-full py-3 hover:bg-surface-hover rounded-lg transition-colors text-left min-h-[44px]"
              >
                <div
                  class="w-8 h-8 rounded flex-shrink-0"
                  style="background-color: {c.color ?? '#334155'}"
                ></div>
                <span>{c.name}</span>
              </button>
            {/each}
          {/if}
        {:else}
          {#each filteredTree as node (node.id)}
            <LocationNodePicker
              {node}
              {onSelect}
              onDrill={(id) => (locationId = id)}
            />
          {/each}
        {/if}
      {/if}
    </div>
  </div>
</div>
