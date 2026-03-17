<script lang="ts">
  import { ApiClientError } from "~/api";

  let { children }: { children: any } = $props();

  let error = $state<Error | null>(null);

  function getMessage(err: Error): string {
    if (err instanceof ApiClientError) {
      if (err.status === 404) return "Not found";
      return err.message;
    }
    return "Something went wrong";
  }

  function retry() {
    error = null;
  }
</script>

<svelte:boundary onerror={(err: Error) => (error = err)}>
  {#if error}
    <div class="flex flex-col items-center justify-center py-12 text-center px-4">
      <svg
        class="w-12 h-12 text-danger mb-4"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
      <p class="text-text-secondary mb-4">{getMessage(error)}</p>
      <button
        onclick={retry}
        class="px-4 py-2 bg-surface-raised hover:bg-surface-hover text-text-primary rounded-lg transition-colors min-h-[44px]"
      >
        Try Again
      </button>
    </div>
  {:else}
    {@render children()}
  {/if}
</svelte:boundary>
