<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  let {
    value = "",
    onSearch,
    autofocus = false,
    placeholder = "Search...",
  }: {
    value?: string;
    onSearch: (query: string) => void;
    autofocus?: boolean;
    placeholder?: string;
  } = $props();

  let query = $state("");
  let timer: ReturnType<typeof setTimeout>;

  onMount(() => {
    query = value;
  });

  $effect(() => {
    query;
    clearTimeout(timer);
    timer = setTimeout(() => onSearch(query), 300);
  });

  onDestroy(() => clearTimeout(timer));

  function clear() {
    query = "";
    onSearch("");
  }
</script>

<div class="relative">
  <svg
    class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-text-muted pointer-events-none"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
  >
    <circle cx="11" cy="11" r="8" />
    <line x1="21" y1="21" x2="16.65" y2="16.65" />
  </svg>
  <!-- svelte-ignore a11y_autofocus -->
  <input
    type="text"
    bind:value={query}
    {autofocus}
    {placeholder}
    class="w-full pl-10 pr-10 py-3 bg-surface-raised rounded-lg text-text-primary placeholder:text-text-muted outline-none focus:ring-2 focus:ring-primary/50 min-h-[44px]"
  />
  {#if query}
    <button
      onclick={clear}
      aria-label="Clear search"
      class="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-text-muted hover:text-text-secondary min-h-[44px] min-w-[44px] flex items-center justify-center"
    >
      <svg
        class="w-5 h-5"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </button>
  {/if}
</div>
