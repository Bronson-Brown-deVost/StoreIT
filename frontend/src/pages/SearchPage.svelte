<script lang="ts">
  import {
    search,
    getItem,
    getContainer,
    getLocation,
    type SearchResultItem,
    type ItemResponse,
    type ContainerResponse,
    type LocationResponse,
  } from "~/api";
  import SearchBar from "~/components/SearchBar.svelte";
  import EntityCard from "~/components/EntityCard.svelte";
  import LoadingSpinner from "~/components/LoadingSpinner.svelte";

  type HydratedResult = SearchResultItem & {
    name: string;
    description: string | null;
    color?: string | null;
  };

  let results = $state<HydratedResult[]>([]);
  let loading = $state(false);
  let searched = $state(false);

  async function handleSearch(query: string) {
    if (!query.trim()) {
      results = [];
      searched = false;
      return;
    }

    loading = true;
    searched = true;

    try {
      const resp = await search(query.trim(), 20);

      const hydrated = await Promise.all(
        resp.results.map(async (r) => {
          try {
            let name = "Unknown";
            let description: string | null = null;
            let color: string | null = null;

            if (r.entity_type === "item") {
              const item: ItemResponse = await getItem(r.entity_id);
              name = item.name;
              description = item.description;
            } else if (r.entity_type === "container") {
              const c: ContainerResponse = await getContainer(r.entity_id);
              name = c.name;
              description = c.description;
              color = c.color;
            } else if (r.entity_type === "location") {
              const l: LocationResponse = await getLocation(r.entity_id);
              name = l.name;
              description = l.description;
            }

            return { ...r, name, description, color };
          } catch {
            return {
              ...r,
              name: "Error loading",
              description: null,
              color: null,
            };
          }
        }),
      );

      results = hydrated;
    } catch {
      results = [];
    } finally {
      loading = false;
    }
  }

  function entityHref(r: HydratedResult): string {
    switch (r.entity_type) {
      case "item":
        return `/items/${r.entity_id}`;
      case "container":
        return `/containers/${r.entity_id}`;
      case "location":
        return `/locations/${r.entity_id}`;
      default:
        return "#";
    }
  }
</script>

<div class="px-4 pt-[env(safe-area-inset-top)]">
  <div class="pt-4 pb-4">
    <h1 class="text-2xl font-bold mb-4">Search</h1>
    <SearchBar onSearch={handleSearch} autofocus placeholder="Search items, containers, locations..." />
  </div>

  {#if loading}
    <LoadingSpinner class="py-8" />
  {/if}

  {#if !loading && searched && !results.length}
    <p class="text-text-secondary text-center py-8">No results found</p>
  {/if}

  <div class="flex flex-col gap-2">
    {#each results as r (r.entity_id)}
      <EntityCard
        href={entityHref(r)}
        name={r.name}
        description={r.description}
        color={r.color}
        badge={r.entity_type}
        entityType={r.entity_type}
        entityId={r.entity_id}
      />
    {/each}
  </div>
</div>
