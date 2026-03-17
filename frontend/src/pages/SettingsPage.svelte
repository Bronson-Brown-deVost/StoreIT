<script lang="ts">
  import { onMount } from "svelte";
  import { auth } from "~/lib/auth.svelte";
  import {
    listItems,
    listContainers,
    getLocations,
    getLocationTree,
    type ItemResponse,
    type ContainerResponse,
    type LocationResponse,
    type LocationTreeNode,
  } from "~/api";
  import LoadingSpinner from "~/components/LoadingSpinner.svelte";

  let exporting = $state(false);
  let exportError = $state<string | null>(null);

  function downloadFile(content: string, filename: string, mimeType: string) {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  function escapeCsv(value: string | null | undefined): string {
    if (!value) return "";
    if (value.includes(",") || value.includes('"') || value.includes("\n")) {
      return `"${value.replace(/"/g, '""')}"`;
    }
    return value;
  }

  function buildLocationPathMap(
    nodes: LocationTreeNode[],
    parentPath: string[] = [],
  ): Map<string, string[]> {
    const map = new Map<string, string[]>();
    for (const node of nodes) {
      const path = [...parentPath, node.name];
      map.set(node.id, path);
      const childMap = buildLocationPathMap(node.children, path);
      for (const [k, v] of childMap) {
        map.set(k, v);
      }
    }
    return map;
  }

  async function handleExportCsv() {
    exporting = true;
    exportError = null;
    try {
      const [itemsList, containersList, locations, tree] = await Promise.all([
        listItems(),
        listContainers(),
        getLocations(),
        getLocationTree(),
      ]);

      const containerMap = new Map<string, ContainerResponse>();
      for (const c of containersList) containerMap.set(c.id, c);

      const locationMap = new Map<string, LocationResponse>();
      for (const l of locations) locationMap.set(l.id, l);

      const locationPathMap = buildLocationPathMap(tree);

      const resolveContainerPath = (containerId: string): string => {
        const parts: string[] = [];
        let current = containerMap.get(containerId);
        while (current) {
          parts.unshift(current.name);
          if (current.parent_container_id) {
            current = containerMap.get(current.parent_container_id);
          } else if (current.parent_location_id) {
            const locPath = locationPathMap.get(current.parent_location_id);
            if (locPath) parts.unshift(...locPath);
            break;
          } else {
            break;
          }
        }
        return parts.join(" > ");
      };

      const resolveItemPath = (item: ItemResponse): string => {
        if (item.container_id) return resolveContainerPath(item.container_id);
        if (item.location_id) {
          const locPath = locationPathMap.get(item.location_id);
          return locPath ? locPath.join(" > ") : "";
        }
        return "";
      };

      const header =
        "Name,Description,Category,Material,Color,Quantity,Condition,Barcode,Aliases,Keywords,Location Path";
      const rows = itemsList.map((i) =>
        [
          escapeCsv(i.name),
          escapeCsv(i.description),
          escapeCsv(i.category),
          escapeCsv(i.material),
          escapeCsv(i.color),
          String(i.quantity),
          escapeCsv(i.condition_notes),
          escapeCsv(i.barcode),
          escapeCsv(i.aliases.join("; ")),
          escapeCsv(i.keywords.join("; ")),
          escapeCsv(resolveItemPath(i)),
        ].join(","),
      );

      const csv = [header, ...rows].join("\n");
      const date = new Date().toISOString().slice(0, 10);
      downloadFile(csv, `storeit-inventory-${date}.csv`, "text/csv");
    } catch (err) {
      exportError = err instanceof Error ? err.message : "Export failed";
    } finally {
      exporting = false;
    }
  }

  async function handleExportJson() {
    exporting = true;
    exportError = null;
    try {
      const [itemsList, containersList, locations, tree] = await Promise.all([
        listItems(),
        listContainers(),
        getLocations(),
        getLocationTree(),
      ]);

      const data = { locations, location_tree: tree, containers: containersList, items: itemsList };
      const json = JSON.stringify(data, null, 2);
      const date = new Date().toISOString().slice(0, 10);
      downloadFile(json, `storeit-inventory-${date}.json`, "application/json");
    } catch (err) {
      exportError = err instanceof Error ? err.message : "Export failed";
    } finally {
      exporting = false;
    }
  }
</script>

<div class="px-4 pt-[env(safe-area-inset-top)]">
  <div class="pt-4 pb-4">
    <h1 class="text-2xl font-bold">Settings</h1>
  </div>

  {#if auth.user}
    <div class="space-y-6">
      <!-- User info -->
      <div class="bg-surface-raised rounded-lg p-4">
        <h2 class="text-sm font-medium text-text-secondary mb-3 uppercase tracking-wider">
          Account
        </h2>
        <div class="space-y-2">
          <div class="flex justify-between py-1">
            <span class="text-text-secondary">Name</span>
            <span>{auth.user.display_name}</span>
          </div>
          <div class="flex justify-between py-1">
            <span class="text-text-secondary">Email</span>
            <span class="truncate ml-4">{auth.user.email}</span>
          </div>
        </div>
      </div>

      <!-- Group switcher -->
      {#if auth.groups.length > 1}
        <div class="bg-surface-raised rounded-lg p-4">
          <h2 class="text-sm font-medium text-text-secondary mb-3 uppercase tracking-wider">
            Group
          </h2>
          <div class="space-y-1">
            {#each auth.groups as group (group.id)}
              <button
                onclick={() => auth.switchGroup(group.id)}
                class={`w-full flex items-center justify-between px-3 py-2.5 rounded-lg transition-colors min-h-[44px] ${
                  group.id === auth.activeGroupId
                    ? "bg-primary/10 text-primary"
                    : "hover:bg-surface-hover"
                }`}
              >
                <span>{group.name}</span>
                {#if group.id === auth.activeGroupId}
                  <svg
                    class="w-5 h-5"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                {/if}
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Export -->
      <div class="bg-surface-raised rounded-lg p-4">
        <h2 class="text-sm font-medium text-text-secondary mb-3 uppercase tracking-wider">
          Export Data
        </h2>
        <p class="text-sm text-text-secondary mb-3">
          Download your entire inventory for backup or insurance purposes.
        </p>
        {#if exportError}
          <div class="p-3 bg-danger/10 text-danger rounded-lg text-sm mb-3">
            {exportError}
          </div>
        {/if}
        <div class="flex gap-3">
          <button
            onclick={handleExportCsv}
            disabled={exporting}
            class="flex-1 py-3 bg-surface hover:bg-surface-hover rounded-lg transition-colors min-h-[44px] disabled:opacity-50 flex items-center justify-center gap-2"
          >
            {#if exporting}
              <LoadingSpinner />
            {:else}
              <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
              CSV
            {/if}
          </button>
          <button
            onclick={handleExportJson}
            disabled={exporting}
            class="flex-1 py-3 bg-surface hover:bg-surface-hover rounded-lg transition-colors min-h-[44px] disabled:opacity-50 flex items-center justify-center gap-2"
          >
            {#if exporting}
              <LoadingSpinner />
            {:else}
              <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
              JSON
            {/if}
          </button>
        </div>
      </div>

      <!-- Logout -->
      <button
        onclick={() => auth.logout()}
        class="w-full py-3 text-danger border border-danger/30 hover:bg-danger/10 rounded-lg transition-colors min-h-[44px]"
      >
        Sign Out
      </button>
    </div>
  {/if}
</div>
