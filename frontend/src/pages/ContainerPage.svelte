<script lang="ts">
  import {
    getContainer,
    getContainerAncestry,
    getContainerContainers,
    getContainerItems,
    createContainer,
    uploadPhoto,
  } from "~/api";
  import EntityCard from "~/components/EntityCard.svelte";
  import EmptyState from "~/components/EmptyState.svelte";
  import LoadingSpinner from "~/components/LoadingSpinner.svelte";
  import Breadcrumbs from "~/components/Breadcrumbs.svelte";
  import MoveFlow from "~/components/MoveFlow.svelte";
  import NfcTagManager from "~/components/NfcTagManager.svelte";
  import PrintLabel from "~/components/PrintLabel.svelte";
  import CreateDialog from "~/components/CreateDialog.svelte";
  import PhotoGallery from "~/components/PhotoGallery.svelte";
  import { route as routeAction, type RouteResult } from "@mateothegreat/svelte5-router";

  let { route: routeData }: { route: RouteResult } = $props();

  let id = $derived(routeData?.result?.path?.params?.id as string ?? "");
  let container = $state<any>(null);
  let ancestry = $state<any[]>([]);
  let childContainers = $state<any[]>([]);
  let items = $state<any[]>([]);
  let loading = $state(true);
  let showMove = $state(false);
  let showPrint = $state(false);
  let showCreate = $state(false);

  async function loadAll() {
    loading = true;
    try {
      const [c, anc, ch, it] = await Promise.all([
        getContainer(id),
        getContainerAncestry(id),
        getContainerContainers(id),
        getContainerItems(id),
      ]);
      container = c;
      ancestry = anc;
      childContainers = ch;
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

  let breadcrumbAncestry = $derived(
    ancestry.filter((n) => n.id !== id)
  );

  async function handleCreateContainer(name: string, description?: string, color?: string, _lat?: number, _lng?: number, photo?: File) {
    const c = await createContainer({ parent_type: "container", parent_id: id, name, description, color });
    if (photo) await uploadPhoto("container", c.id, photo);
    childContainers = await getContainerContainers(id);
  }
</script>

<div class="px-4 pt-[env(safe-area-inset-top)]">
  <div class="pt-4">
    <Breadcrumbs
      items={breadcrumbAncestry}
      current={container?.name}
    />
  </div>

  {#if loading}
    <LoadingSpinner class="py-12" />
  {:else if container}
    <div class="flex items-center gap-3 mb-1">
      <h1 class="text-2xl font-bold flex-1">{container.name}</h1>
      {#if container.color}
        <div
          class="w-5 h-5 rounded-full flex-shrink-0"
          style:background-color={container.color}
        ></div>
      {/if}
      <button
        onclick={() => showCreate = true}
        class="p-2 text-text-muted hover:text-text-secondary transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
        title="Add sub-container"
      >
        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
      <button
        onclick={() => showPrint = true}
        class="p-2 text-text-muted hover:text-text-secondary transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
        title="Print label"
      >
        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 6 2 18 2 18 9" />
          <path d="M6 18H4a2 2 0 01-2-2v-5a2 2 0 012-2h16a2 2 0 012 2v5a2 2 0 01-2 2h-2" />
          <rect x="6" y="14" width="12" height="8" />
        </svg>
      </button>
      <button
        onclick={() => showMove = true}
        class="p-2 text-text-muted hover:text-text-secondary transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
        title="Move container"
      >
        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="5 9 2 12 5 15" />
          <polyline points="9 5 12 2 15 5" />
          <polyline points="19 9 22 12 19 15" />
          <polyline points="9 19 12 22 15 19" />
          <line x1="2" y1="12" x2="22" y2="12" />
          <line x1="12" y1="2" x2="12" y2="22" />
        </svg>
      </button>
    </div>

    {#if container.description}
      <p class="text-text-secondary mb-4">{container.description}</p>
    {/if}

    <PhotoGallery entityType="container" entityId={container.id} />

    {#if childContainers.length > 0}
      <h2 class="text-lg font-semibold mt-4 mb-3">Containers</h2>
      <div class="flex flex-col gap-2">
        {#each childContainers as child (child.id)}
          <EntityCard
            href={`/containers/${child.id}`}
            name={child.name}
            description={child.description}
            color={child.color}
            badge="container"
            entityType="container"
            entityId={child.id}
          />
        {/each}
      </div>
    {/if}

    {#if items.length > 0}
      <h2 class="text-lg font-semibold mt-4 mb-3">Items</h2>
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

    {#if !childContainers.length && !items.length}
      <EmptyState message="This container is empty" />
    {/if}

    <div class="mt-6">
      <NfcTagManager entityType="container" entityId={container.id} />
    </div>

    <a
      use:routeAction
      href={`/add?container=${container.id}`}
      class="fixed bottom-24 right-4 w-14 h-14 bg-primary hover:bg-primary-hover text-white rounded-full flex items-center justify-center shadow-lg transition-colors z-40"
      aria-label="Add Item Here"
    >
      <svg class="w-7 h-7" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="12" y1="5" x2="12" y2="19" />
        <line x1="5" y1="12" x2="19" y2="12" />
      </svg>
    </a>

    {#if showMove}
      <MoveFlow
        entityType="container"
        entityId={container.id}
        entityName={container.name}
        onComplete={() => {
          showMove = false;
          loadAll();
        }}
        onClose={() => showMove = false}
      />
    {/if}

    {#if showPrint}
      <PrintLabel
        name={container.name}
        entityType="container"
        entityId={container.id}
        description={container.description}
        locationPath={ancestry.map((n) => n.name)}
        onClose={() => showPrint = false}
      />
    {/if}

    {#if showCreate}
      <CreateDialog
        title="New Sub-Container"
        showColor
        onSubmit={handleCreateContainer}
        onClose={() => showCreate = false}
      />
    {/if}
  {/if}
</div>
