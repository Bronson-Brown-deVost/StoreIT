<script lang="ts">
  import { onMount } from "svelte";
  import { goto, route } from "@mateothegreat/svelte5-router";
  import {
    identifyPhoto,
    identifyCorrect,
    createItem,
    uploadPhoto,
    getContainer,
    type IdentificationResponse,
    type CreateItemRequest,
  } from "~/api";
  import ParentPicker from "~/components/ParentPicker.svelte";
  import type { SelectedParent } from "~/components/ParentPicker.svelte";
  import LoadingSpinner from "~/components/LoadingSpinner.svelte";

  type Step = "capture" | "review" | "saving";

  let step = $state<Step>("capture");
  let file = $state<File | null>(null);
  let previewUrl = $state<string | null>(null);
  let identifying = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);

  // Form fields
  let name = $state("");
  let description = $state("");
  let category = $state("");
  let material = $state("");
  let color = $state("");
  let conditionNotes = $state("");
  let showDescription = $state(false);

  // Parent picker
  let parent = $state<SelectedParent | null>(null);
  let showPicker = $state(false);
  let savedItemId = $state<string | null>(null);

  // Pre-fill parent from URL if container query param present
  onMount(async () => {
    const params = new URLSearchParams(window.location.search);
    const containerId = params.get("container");
    if (containerId && !parent) {
      try {
        const c = await getContainer(containerId);
        parent = { type: "container", id: c.id, name: c.name };
      } catch {
        // ignore
      }
    }
  });

  function applyIdentification(result: IdentificationResponse) {
    name = result.name;
    if (result.description) description = result.description;
    if (result.category) category = result.category;
    if (result.material) material = result.material;
    if (result.color) color = result.color;
    if (result.condition_notes) conditionNotes = result.condition_notes;
  }

  async function handleFileSelect(e: Event & { currentTarget: HTMLInputElement }) {
    const f = e.currentTarget.files?.[0];
    if (!f) return;

    file = f;
    previewUrl = URL.createObjectURL(f);
    step = "review";
    error = null;

    // Auto-identify
    identifying = true;
    try {
      const result: IdentificationResponse = await identifyPhoto(f);
      applyIdentification(result);
    } catch (err) {
      console.warn("AI identification failed:", err);
    } finally {
      identifying = false;
    }
  }

  async function handleReidentify() {
    if (!file || !name) return;

    identifying = true;
    error = null;
    try {
      const result = await identifyCorrect(file, name);
      applyIdentification(result);
    } catch (err) {
      error = "Re-identification failed";
    } finally {
      identifying = false;
    }
  }

  async function handleSave() {
    if (!name.trim() || !parent || !file) {
      error = "Name and parent are required";
      return;
    }

    saving = true;
    step = "saving";
    error = null;

    try {
      const req: CreateItemRequest = {
        parent_type: parent.type,
        parent_id: parent.id,
        name: name.trim(),
        description: description.trim() || null,
        category: category.trim() || null,
        material: material.trim() || null,
        color: color.trim() || null,
        condition_notes: conditionNotes.trim() || null,
      };

      const item = await createItem(req);
      await uploadPhoto("item", item.id, file);
      savedItemId = item.id;
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to save";
      step = "review";
    } finally {
      saving = false;
    }
  }

  function resetForm() {
    step = "capture";
    file = null;
    if (previewUrl) URL.revokeObjectURL(previewUrl);
    previewUrl = null;
    name = "";
    description = "";
    category = "";
    material = "";
    color = "";
    conditionNotes = "";
    showDescription = false;
    error = null;
    savedItemId = null;
  }
</script>

<div class="px-4 pt-[env(safe-area-inset-top)]">
  <div class="pt-4 pb-4 flex items-center justify-between">
    <h1 class="text-2xl font-bold">Add Item</h1>
    <a
      use:route
      href="/add/batch"
      class="text-sm text-primary hover:text-primary-hover min-h-[32px] flex items-center"
    >
      Batch Add
    </a>
  </div>

  <!-- Step 1: Capture -->
  {#if step === "capture"}
    <div class="flex flex-col items-center justify-center py-12">
      <svg
        class="w-16 h-16 text-text-muted mb-4"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <path d="M23 19a2 2 0 01-2 2H3a2 2 0 01-2-2V8a2 2 0 012-2h4l2-3h6l2 3h4a2 2 0 012 2z" />
        <circle cx="12" cy="13" r="4" />
      </svg>
      <p class="text-text-secondary mb-6">Take a photo to identify the item</p>
      <label class="cursor-pointer px-6 py-3 bg-primary hover:bg-primary-hover text-white font-semibold rounded-lg transition-colors min-h-[44px] flex items-center gap-2">
        <svg
          class="w-5 h-5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M23 19a2 2 0 01-2 2H3a2 2 0 01-2-2V8a2 2 0 012-2h4l2-3h6l2 3h4a2 2 0 012 2z" />
          <circle cx="12" cy="13" r="4" />
        </svg>
        Take Photo
        <input
          type="file"
          accept="image/*"
          onchange={handleFileSelect}
          class="hidden"
        />
      </label>
    </div>
  {/if}

  <!-- Step 2: Review + Edit -->
  {#if step === "review"}
    <div class="space-y-4">
      <!-- Photo preview -->
      {#if previewUrl}
        <div class="relative rounded-lg overflow-hidden">
          <img
            src={previewUrl}
            alt="Preview"
            class="w-full max-h-48 object-cover"
          />
          {#if identifying}
            <div class="absolute inset-0 bg-black/50 flex items-center justify-center">
              <div class="flex flex-col items-center gap-2">
                <LoadingSpinner />
                <span class="text-white text-sm">Identifying...</span>
              </div>
            </div>
          {/if}
        </div>
      {/if}

      {#if error}
        <div class="p-3 bg-danger/10 text-danger rounded-lg text-sm">
          {error}
        </div>
      {/if}

      <!-- Form fields -->
      <div>
        <label class="block text-sm text-text-secondary mb-1">Name *
        <input
          type="text"
          value={name}
          oninput={(e) => name = e.currentTarget.value}
          class="w-full px-3 py-2 bg-surface-raised rounded-lg text-text-primary outline-none focus:ring-2 focus:ring-primary/50 min-h-[44px]"
          placeholder="Item name"
        />
        </label>
      </div>

      {#if showDescription}
        <div>
          <label class="block text-sm text-text-secondary mb-1">
            Description
          <textarea
            value={description}
            oninput={(e) => description = e.currentTarget.value}
            rows="2"
            class="w-full px-3 py-2 bg-surface-raised rounded-lg text-text-primary outline-none focus:ring-2 focus:ring-primary/50 resize-none"
            placeholder="Description"
          ></textarea>
          </label>
        </div>
      {:else}
        <button
          onclick={() => showDescription = true}
          class="text-sm text-primary"
        >
          + Add description
        </button>
      {/if}

      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="block text-sm text-text-secondary mb-1">
            Category
          <input
            type="text"
            value={category}
            oninput={(e) => category = e.currentTarget.value}
            class="w-full px-3 py-2 bg-surface-raised rounded-lg text-text-primary outline-none focus:ring-2 focus:ring-primary/50 min-h-[44px]"
            placeholder="e.g. Tools"
          />
          </label>
        </div>
        <div>
          <label class="block text-sm text-text-secondary mb-1">
            Material
          <input
            type="text"
            value={material}
            oninput={(e) => material = e.currentTarget.value}
            class="w-full px-3 py-2 bg-surface-raised rounded-lg text-text-primary outline-none focus:ring-2 focus:ring-primary/50 min-h-[44px]"
            placeholder="e.g. Metal"
          />
          </label>
        </div>
        <div>
          <label class="block text-sm text-text-secondary mb-1">
            Color
          <input
            type="text"
            value={color}
            oninput={(e) => color = e.currentTarget.value}
            class="w-full px-3 py-2 bg-surface-raised rounded-lg text-text-primary outline-none focus:ring-2 focus:ring-primary/50 min-h-[44px]"
            placeholder="e.g. Red"
          />
          </label>
        </div>
        <div>
          <label class="block text-sm text-text-secondary mb-1">
            Condition
          <input
            type="text"
            value={conditionNotes}
            oninput={(e) => conditionNotes = e.currentTarget.value}
            class="w-full px-3 py-2 bg-surface-raised rounded-lg text-text-primary outline-none focus:ring-2 focus:ring-primary/50 min-h-[44px]"
            placeholder="e.g. Good"
          />
          </label>
        </div>
      </div>

      <!-- Parent picker -->
      <div>
        <span class="block text-sm text-text-secondary mb-1">
          Location / Container *
        </span>
        <button
          onclick={() => showPicker = true}
          class="w-full px-3 py-2 bg-surface-raised rounded-lg text-left min-h-[44px] flex items-center justify-between"
        >
          <span class={parent ? "text-text-primary" : "text-text-muted"}>
            {parent
              ? `${parent.name} (${parent.type})`
              : "Select where to store this item"}
          </span>
          <svg
            class="w-5 h-5 text-text-muted"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <polyline points="9 18 15 12 9 6" />
          </svg>
        </button>
      </div>

      <!-- Actions -->
      <div class="flex gap-3 pt-2">
        <button
          onclick={handleReidentify}
          disabled={identifying || !name}
          class="flex-1 py-3 bg-surface-raised hover:bg-surface-hover rounded-lg transition-colors min-h-[44px] disabled:opacity-50"
        >
          Re-identify
        </button>
        <button
          onclick={handleSave}
          disabled={identifying || !name.trim() || !parent}
          class="flex-1 py-3 bg-primary hover:bg-primary-hover text-white font-semibold rounded-lg transition-colors min-h-[44px] disabled:opacity-50"
        >
          Save Item
        </button>
      </div>
    </div>

    {#if showPicker}
      <ParentPicker
        onSelect={(p) => {
          parent = p;
          showPicker = false;
        }}
        onClose={() => showPicker = false}
      />
    {/if}
  {/if}

  <!-- Step 3: Saving / Success -->
  {#if step === "saving"}
    {#if !saving && savedItemId}
      <div class="flex flex-col items-center justify-center py-12">
        <svg
          class="w-16 h-16 text-success mb-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M22 11.08V12a10 10 0 11-5.93-9.14" />
          <polyline points="22 4 12 14.01 9 11.01" />
        </svg>
        <p class="text-lg font-semibold mb-6">Item saved!</p>
        <div class="flex gap-3">
          <button
            onclick={resetForm}
            class="px-6 py-3 bg-surface-raised hover:bg-surface-hover rounded-lg transition-colors min-h-[44px]"
          >
            Add Another
          </button>
          <button
            onclick={() => goto(`/items/${savedItemId}`)}
            class="px-6 py-3 bg-primary hover:bg-primary-hover text-white font-semibold rounded-lg transition-colors min-h-[44px]"
          >
            View Item
          </button>
        </div>
      </div>
    {:else}
      <div class="flex flex-col items-center justify-center py-12">
        <LoadingSpinner />
        <p class="text-text-secondary mt-4">Saving item...</p>
      </div>
    {/if}
  {/if}
</div>
