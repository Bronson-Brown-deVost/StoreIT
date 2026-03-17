<script lang="ts">
  let {
    message,
    actionLabel,
    onAction,
  }: {
    message: string;
    actionLabel?: string;
    onAction?: () => void;
  } = $props();
</script>

<div class="flex flex-col items-center justify-center py-12 text-center">
  <svg
    class="w-12 h-12 text-text-muted mb-4"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.5"
  >
    <path d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
  </svg>
  <p class="text-text-secondary mb-4">{message}</p>
  {#if actionLabel && onAction}
    <button
      onclick={onAction}
      class="px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg transition-colors min-h-[44px]"
    >
      {actionLabel}
    </button>
  {/if}
</div>
