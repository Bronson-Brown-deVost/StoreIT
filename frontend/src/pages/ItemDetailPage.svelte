<script lang="ts">
  import { goto, route as routeAction, type RouteResult } from "@mateothegreat/svelte5-router";
  import { getItem, deleteItem, updateItem, getContainerAncestry, getLocation } from "~/api";
  import type { AncestryNode } from "~/api";
  import PhotoGallery from "~/components/PhotoGallery.svelte";
  import LoadingSpinner from "~/components/LoadingSpinner.svelte";
  import MoveFlow from "~/components/MoveFlow.svelte";

  let { route: routeData }: { route: RouteResult } = $props();

  let id = $derived(routeData?.result?.path?.params?.id as string ?? "");
  let item = $state<any>(null);
  let itemLoading = $state(true);
  let locationPath = $state<AncestryNode[]>([]);
  let deleting = $state(false);
  let showDeleteConfirm = $state(false);
  let showMove = $state(false);
  let editing = $state(false);
  let saving = $state(false);

  // Edit form state
  let editName = $state("");
  let editDescription = $state("");
  let editCategory = $state("");
  let editMaterial = $state("");
  let editColor = $state("");
  let editBarcode = $state("");
  let editQuantity = $state(1);
  let editCondition = $state("");

  async function loadItem() {
    itemLoading = true;
    try {
      item = await getItem(id);
      await loadLocationPath();
    } catch {
      // handle error
    } finally {
      itemLoading = false;
    }
  }

  async function loadLocationPath() {
    if (!item) return;
    const path: AncestryNode[] = [];
    if (item.container_id) {
      const ancestry = await getContainerAncestry(item.container_id);
      path.push(...ancestry);
    } else if (item.location_id) {
      const loc = await getLocation(item.location_id);
      path.push({ entity_type: "location", id: loc.id, name: loc.name });
    }
    locationPath = path;
  }

  $effect(() => {
    if (id) loadItem();
  });

  function locationHref(node: AncestryNode): string {
    return node.entity_type === "location"
      ? `/locations/${node.id}`
      : `/containers/${node.id}`;
  }

  function startEdit() {
    if (!item) return;
    editName = item.name;
    editDescription = item.description ?? "";
    editCategory = item.category ?? "";
    editMaterial = item.material ?? "";
    editColor = item.color ?? "";
    editBarcode = item.barcode ?? "";
    editQuantity = item.quantity;
    editCondition = item.condition_notes ?? "";
    editing = true;
  }

  async function handleSave() {
    saving = true;
    try {
      await updateItem(id, {
        name: editName.trim() || undefined,
        description: editDescription.trim() || null,
        category: editCategory.trim() || null,
        material: editMaterial.trim() || null,
        color: editColor.trim() || null,
        barcode: editBarcode.trim() || null,
        quantity: editQuantity,
        condition_notes: editCondition.trim() || null,
      });
      item = await getItem(id);
      editing = false;
    } finally {
      saving = false;
    }
  }

  async function handleDelete() {
    deleting = true;
    try {
      await deleteItem(id);
      goto("/");
    } catch {
      deleting = false;
      showDeleteConfirm = false;
    }
  }
</script>

<div class="px-4 pt-[env(safe-area-inset-top)]">
  <div class="pt-4 pb-2 flex items-center gap-3">
    <button
      onclick={() => history.back()}
      aria-label="Go back"
      class="min-h-[44px] min-w-[44px] flex items-center justify-center -ml-2"
    >
      <svg
        class="w-6 h-6 text-text-secondary"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <polyline points="15 18 9 12 15 6" />
      </svg>
    </button>
    <h1 class="text-xl font-bold flex-1 truncate">{item?.name ?? "Item"}</h1>
    <button
      onclick={startEdit}
      class="p-2 text-text-muted hover:text-text-secondary transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
      title="Edit item"
    >
      <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" />
        <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" />
      </svg>
    </button>
    <button
      onclick={() => showMove = true}
      class="p-2 text-text-muted hover:text-text-secondary transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
      title="Move item"
    >
      <svg
        class="w-5 h-5"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <polyline points="5 9 2 12 5 15" />
        <polyline points="9 5 12 2 15 5" />
        <polyline points="19 9 22 12 19 15" />
        <polyline points="9 19 12 22 15 19" />
        <line x1="2" y1="12" x2="22" y2="12" />
        <line x1="12" y1="2" x2="12" y2="22" />
      </svg>
    </button>
  </div>

  {#if itemLoading}
    <LoadingSpinner class="py-12" />
  {:else if item}
    <!-- Location path -->
    {#if locationPath.length > 0}
      <div class="flex items-center gap-1.5 text-sm text-text-muted mb-3 overflow-x-auto scrollbar-none">
        <svg class="w-4 h-4 flex-shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0118 0z" />
          <circle cx="12" cy="10" r="3" />
        </svg>
        {#each locationPath as node, idx}
          {#if idx > 0}
            <span class="flex-shrink-0">&rsaquo;</span>
          {/if}
          <a
            use:routeAction
            href={locationHref(node)}
            class="hover:text-primary whitespace-nowrap flex-shrink-0"
          >
            {node.name}
          </a>
        {/each}
      </div>
    {/if}

    <!-- Photos with upload and lightbox -->
    <PhotoGallery entityType="item" entityId={item.id} />

    <!-- Edit form -->
    {#if editing}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="fixed inset-0 bg-black/60 z-[60] flex items-end sm:items-center justify-center" onclick={(e) => { if (e.target === e.currentTarget && !saving) editing = false; }}>
        <div class="bg-surface-raised w-full max-w-lg rounded-t-2xl sm:rounded-2xl p-6 pb-[env(safe-area-inset-bottom)] max-h-[100dvh] overflow-y-auto">
          <h2 class="text-lg font-semibold mb-4">Edit Item</h2>
          <form onsubmit={(e) => { e.preventDefault(); handleSave(); }} class="space-y-3">
            <div>
              <label class="block text-sm text-text-secondary mb-1">Name
              <input type="text" value={editName} oninput={(e) => editName = e.currentTarget.value} class="w-full px-3 py-2 bg-surface rounded-lg border border-border focus:border-primary focus:outline-none min-h-[44px]" required />
              </label>
            </div>
            <div>
              <label class="block text-sm text-text-secondary mb-1">Description
              <input type="text" value={editDescription} oninput={(e) => editDescription = e.currentTarget.value} class="w-full px-3 py-2 bg-surface rounded-lg border border-border focus:border-primary focus:outline-none min-h-[44px]" />
              </label>
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="block text-sm text-text-secondary mb-1">Category
                <input type="text" value={editCategory} oninput={(e) => editCategory = e.currentTarget.value} class="w-full px-3 py-2 bg-surface rounded-lg border border-border focus:border-primary focus:outline-none min-h-[44px]" />
                </label>
              </div>
              <div>
                <label class="block text-sm text-text-secondary mb-1">Material
                <input type="text" value={editMaterial} oninput={(e) => editMaterial = e.currentTarget.value} class="w-full px-3 py-2 bg-surface rounded-lg border border-border focus:border-primary focus:outline-none min-h-[44px]" />
                </label>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="block text-sm text-text-secondary mb-1">Color
                <input type="text" value={editColor} oninput={(e) => editColor = e.currentTarget.value} class="w-full px-3 py-2 bg-surface rounded-lg border border-border focus:border-primary focus:outline-none min-h-[44px]" />
                </label>
              </div>
              <div>
                <label class="block text-sm text-text-secondary mb-1">Barcode
                <input type="text" value={editBarcode} oninput={(e) => editBarcode = e.currentTarget.value} class="w-full px-3 py-2 bg-surface rounded-lg border border-border focus:border-primary focus:outline-none min-h-[44px]" />
                </label>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="block text-sm text-text-secondary mb-1">Quantity
                <input type="number" value={editQuantity} oninput={(e) => editQuantity = Number(e.currentTarget.value)} class="w-full px-3 py-2 bg-surface rounded-lg border border-border focus:border-primary focus:outline-none min-h-[44px]" min="1" />
                </label>
              </div>
              <div>
                <label class="block text-sm text-text-secondary mb-1">Condition
                <input type="text" value={editCondition} oninput={(e) => editCondition = e.currentTarget.value} class="w-full px-3 py-2 bg-surface rounded-lg border border-border focus:border-primary focus:outline-none min-h-[44px]" />
                </label>
              </div>
            </div>
            <div class="flex gap-3 pt-2">
              <button type="button" onclick={() => editing = false} disabled={saving} class="flex-1 py-3 bg-surface hover:bg-surface-hover rounded-lg transition-colors min-h-[44px]">Cancel</button>
              <button type="submit" disabled={saving || !editName.trim()} class="flex-1 py-3 bg-primary hover:bg-primary-hover text-white font-semibold rounded-lg transition-colors min-h-[44px] disabled:opacity-50">{saving ? "Saving..." : "Save"}</button>
            </div>
          </form>
        </div>
      </div>
    {/if}

    <!-- Read-only details -->
    {#if !editing}
      {#if item.description}
        <p class="text-text-secondary mb-4">{item.description}</p>
      {/if}

      <div class="bg-surface-raised rounded-lg px-4">
        {#if item.category}
          <div class="flex justify-between py-2 border-b border-border">
            <span class="text-text-secondary">Category</span>
            <span class="text-text-primary text-right">{item.category}</span>
          </div>
        {/if}
        {#if item.material}
          <div class="flex justify-between py-2 border-b border-border">
            <span class="text-text-secondary">Material</span>
            <span class="text-text-primary text-right">{item.material}</span>
          </div>
        {/if}
        {#if item.color}
          <div class="flex justify-between py-2 border-b border-border">
            <span class="text-text-secondary">Color</span>
            <span class="text-text-primary text-right">{item.color}</span>
          </div>
        {/if}
        {#if item.barcode}
          <div class="flex justify-between py-2 border-b border-border">
            <span class="text-text-secondary">Barcode</span>
            <span class="text-text-primary text-right">{item.barcode}</span>
          </div>
        {/if}
        <div class="flex justify-between py-2 border-b border-border">
          <span class="text-text-secondary">Quantity</span>
          <span class="text-text-primary text-right">{String(item.quantity)}</span>
        </div>
        {#if item.condition_notes}
          <div class="flex justify-between py-2 border-b border-border">
            <span class="text-text-secondary">Condition</span>
            <span class="text-text-primary text-right">{item.condition_notes}</span>
          </div>
        {/if}
        {#if item.aliases.length}
          <div class="flex justify-between py-2 border-b border-border">
            <span class="text-text-secondary">Aliases</span>
            <span class="text-text-primary text-right">{item.aliases.join(", ")}</span>
          </div>
        {/if}
        {#if item.keywords.length}
          <div class="flex justify-between py-2 border-b border-border">
            <span class="text-text-secondary">Keywords</span>
            <span class="text-text-primary text-right">{item.keywords.join(", ")}</span>
          </div>
        {/if}
      </div>

      <div class="mt-6 flex flex-col gap-3">
        <button
          onclick={startEdit}
          class="w-full py-3 bg-surface-raised hover:bg-surface-hover rounded-lg transition-colors min-h-[44px] flex items-center justify-center gap-2"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" />
            <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" />
          </svg>
          Edit Item
        </button>

        <button
          onclick={() => showMove = true}
          class="w-full py-3 bg-surface-raised hover:bg-surface-hover rounded-lg transition-colors min-h-[44px] flex items-center justify-center gap-2"
        >
          <svg
            class="w-5 h-5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <polyline points="5 9 2 12 5 15" />
            <polyline points="9 5 12 2 15 5" />
            <polyline points="19 9 22 12 19 15" />
            <polyline points="9 19 12 22 15 19" />
            <line x1="2" y1="12" x2="22" y2="12" />
            <line x1="12" y1="2" x2="12" y2="22" />
          </svg>
          Move Item
        </button>

        {#if showDeleteConfirm}
          <div class="flex gap-3">
            <button
              onclick={() => showDeleteConfirm = false}
              class="flex-1 py-3 bg-surface-raised hover:bg-surface-hover rounded-lg transition-colors min-h-[44px]"
            >
              Cancel
            </button>
            <button
              onclick={handleDelete}
              disabled={deleting}
              class="flex-1 py-3 bg-danger text-white rounded-lg transition-colors min-h-[44px] disabled:opacity-50"
            >
              {deleting ? "Deleting..." : "Confirm Delete"}
            </button>
          </div>
        {:else}
          <button
            onclick={() => showDeleteConfirm = true}
            class="w-full py-3 text-danger border border-danger/30 hover:bg-danger/10 rounded-lg transition-colors min-h-[44px]"
          >
            Delete Item
          </button>
        {/if}
      </div>
    {/if}

    {#if showMove}
      <MoveFlow
        entityType="item"
        entityId={item.id}
        entityName={item.name}
        onComplete={() => {
          showMove = false;
          loadItem();
        }}
        onClose={() => showMove = false}
      />
    {/if}
  {/if}
</div>
