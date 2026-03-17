<script lang="ts">
  import type { LocationTreeNode } from "~/api";
  import type { SelectedParent } from "./ParentPicker.svelte";
  import LocationNodePicker from "./LocationNodePicker.svelte";

  let { node, onSelect, onDrill, depth = 0 }: {
    node: LocationTreeNode;
    onSelect: (parent: SelectedParent) => void;
    onDrill: (id: string) => void;
    depth?: number;
  } = $props();
</script>

<div
  class="flex items-center gap-2 min-h-[44px]"
  style="padding-left: {depth * 16}px"
>
  <button
    onclick={() =>
      onSelect({
        type: "location",
        id: node.id,
        name: node.name,
      })
    }
    class="flex-1 text-left py-2 hover:text-primary transition-colors"
  >
    {node.name}
  </button>
  <button
    onclick={() => onDrill(node.id)}
    class="text-xs text-text-muted hover:text-primary px-2 py-1 min-h-[44px] flex items-center"
  >
    Containers &rarr;
  </button>
</div>

{#each node.children as child (child.id)}
  <LocationNodePicker
    node={child}
    {onSelect}
    {onDrill}
    depth={depth + 1}
  />
{/each}
