<script lang="ts">
  import { moveItem, moveContainer, type MoveRequest } from "~/api";
  import ParentPicker from "./ParentPicker.svelte";
  import type { SelectedParent } from "./ParentPicker.svelte";
  import LoadingSpinner from "./LoadingSpinner.svelte";

  let { entityType, entityId, entityName, onComplete, onClose }: {
    entityType: "item" | "container";
    entityId: string;
    entityName: string;
    onComplete: () => void;
    onClose: () => void;
  } = $props();

  let step = $state<"pick" | "confirm" | "moving" | "done">("pick");
  let target = $state<SelectedParent | null>(null);
  let error = $state<string | null>(null);

  function handleSelect(parent: SelectedParent) {
    target = parent;
    step = "confirm";
  }

  async function handleMove() {
    if (!target) return;

    step = "moving";
    error = null;

    try {
      const req: MoveRequest = {
        target_type: target.type,
        target_id: target.id,
      };

      if (entityType === "item") {
        await moveItem(entityId, req);
      } else {
        await moveContainer(entityId, req);
      }

      step = "done";
      setTimeout(() => onComplete(), 800);
    } catch (err) {
      error = err instanceof Error ? err.message : "Move failed";
      step = "confirm";
    }
  }
</script>

{#if step === "pick"}
  <ParentPicker onSelect={handleSelect} {onClose} />
{/if}

{#if step === "confirm" || step === "moving" || step === "done"}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 bg-black/60 z-[60] flex items-end sm:items-center justify-center"
    onclick={(e) => {
      if (e.target === e.currentTarget && step === "confirm")
        onClose();
    }}
  >
    <div class="bg-surface-raised w-full max-w-lg rounded-t-2xl sm:rounded-2xl p-6 pb-[env(safe-area-inset-bottom)]">
      {#if step === "moving"}
        <div class="flex flex-col items-center py-6">
          <LoadingSpinner />
          <p class="text-text-secondary mt-4">Moving...</p>
        </div>
      {/if}

      {#if step === "done"}
        <div class="flex flex-col items-center py-6">
          <svg
            class="w-12 h-12 text-success mb-3"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M22 11.08V12a10 10 0 11-5.93-9.14" />
            <polyline points="22 4 12 14.01 9 11.01" />
          </svg>
          <p class="font-semibold">Moved!</p>
        </div>
      {/if}

      {#if step === "confirm"}
        <h2 class="text-lg font-semibold mb-4">Confirm Move</h2>

        {#if error}
          <div class="p-3 bg-danger/10 text-danger rounded-lg text-sm mb-4">
            {error}
          </div>
        {/if}

        <div class="bg-surface rounded-lg p-4 mb-4 space-y-2">
          <div class="flex justify-between">
            <span class="text-text-secondary">Moving</span>
            <span class="font-medium">{entityName}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-text-secondary">To</span>
            <span class="font-medium">
              {target?.name} ({target?.type})
            </span>
          </div>
        </div>

        <div class="flex gap-3">
          <button
            onclick={onClose}
            class="flex-1 py-3 bg-surface hover:bg-surface-hover rounded-lg transition-colors min-h-[44px]"
          >
            Cancel
          </button>
          <button
            onclick={() => (step = "pick")}
            class="flex-1 py-3 bg-surface hover:bg-surface-hover rounded-lg transition-colors min-h-[44px]"
          >
            Change
          </button>
          <button
            onclick={handleMove}
            class="flex-1 py-3 bg-primary hover:bg-primary-hover text-white font-semibold rounded-lg transition-colors min-h-[44px]"
          >
            Move
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
