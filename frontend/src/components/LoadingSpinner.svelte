<script lang="ts">
  let { class: className = "" }: { class?: string } = $props();
</script>

<div class={`flex items-center justify-center ${className}`}>
  <div class="w-8 h-8 border-3 border-text-muted border-t-primary rounded-full animate-spin"></div>
</div>
