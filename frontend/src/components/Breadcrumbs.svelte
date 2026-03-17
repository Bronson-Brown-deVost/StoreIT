<script lang="ts">
  import { route } from "@mateothegreat/svelte5-router";
  import type { AncestryNode } from "~/api";

  let { items, current }: { items: AncestryNode[]; current?: string } = $props();

  function entityHref(node: AncestryNode): string {
    switch (node.entity_type) {
      case "location":
        return `/locations/${node.id}`;
      case "container":
        return `/containers/${node.id}`;
      default:
        return "#";
    }
  }
</script>

<nav class="flex items-center gap-1 overflow-x-auto scrollbar-none text-sm py-2 -mx-1 px-1">
  <a
    use:route
    href="/"
    class="text-text-muted hover:text-text-secondary flex-shrink-0 min-h-[32px] flex items-center"
    aria-label="Home"
  >
    <svg
      class="w-4 h-4"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <path d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
    </svg>
  </a>
  {#each items as node}
    <svg
      class="w-4 h-4 text-text-muted flex-shrink-0"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <polyline points="9 18 15 12 9 6" />
    </svg>
    <a
      use:route
      href={entityHref(node)}
      class="text-text-muted hover:text-text-secondary flex-shrink-0 whitespace-nowrap min-h-[32px] flex items-center"
    >
      {node.name}
    </a>
  {/each}
  {#if current}
    <svg
      class="w-4 h-4 text-text-muted flex-shrink-0"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <polyline points="9 18 15 12 9 6" />
    </svg>
    <span class="text-text-primary font-medium flex-shrink-0 whitespace-nowrap">
      {current}
    </span>
  {/if}
</nav>
