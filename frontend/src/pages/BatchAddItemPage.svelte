<script lang="ts">
  import { goto } from "@mateothegreat/svelte5-router";
  import {
    batchCreateItems,
    type CreateItemRequest,
  } from "~/api";
  import ParentPicker from "~/components/ParentPicker.svelte";
  import type { SelectedParent } from "~/components/ParentPicker.svelte";
  import LoadingSpinner from "~/components/LoadingSpinner.svelte";

  interface BatchRow {
    id: number;
    name: string;
    description: string;
    category: string;
    material: string;
    color: string;
    quantity: number;
  }

  let nextRowId = 1;

  function emptyRow(): BatchRow {
    return {
      id: nextRowId++,
      name: "",
      description: "",
      category: "",
      material: "",
      color: "",
      quantity: 1,
    };
  }

  let rows = $state<BatchRow[]>([emptyRow(), emptyRow(), emptyRow()]);
  let parent = $state<SelectedParent | null>(null);
  let showPicker = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let savedCount = $state<number | null>(null);
  let showCsvImport = $state(false);

  function addRow() {
    rows = [...rows, emptyRow()];
  }

  function removeRow(id: number) {
    if (rows.length <= 1) return;
    rows = rows.filter((r) => r.id !== id);
  }

  function updateRow(id: number, field: keyof Omit<BatchRow, "id">, value: string | number) {
    rows = rows.map((r) => (r.id === id ? { ...r, [field]: value } : r));
  }

  let validRows = $derived(rows.filter((r) => r.name.trim()));

  async function handleSave() {
    if (!parent) {
      error = "Select a location or container first";
      return;
    }
    if (validRows.length === 0) {
      error = "Add at least one item with a name";
      return;
    }

    saving = true;
    error = null;

    try {
      const reqs: CreateItemRequest[] = validRows.map((r) => ({
        parent_type: parent!.type,
        parent_id: parent!.id,
        name: r.name.trim(),
        description: r.description.trim() || null,
        category: r.category.trim() || null,
        material: r.material.trim() || null,
        color: r.color.trim() || null,
        quantity: r.quantity || 1,
      }));

      await batchCreateItems(reqs);
      savedCount = reqs.length;
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to save items";
    } finally {
      saving = false;
    }
  }

  function handleCsvImport(text: string) {
    const lines = text.split("\n").filter((l) => l.trim());
    if (lines.length === 0) return;

    const first = lines[0].toLowerCase();
    const hasHeader = first.includes("name") || first.includes("category");
    const dataLines = hasHeader ? lines.slice(1) : lines;

    const newRows: BatchRow[] = dataLines.map((line) => {
      const cols = line.split(",").map((c) => c.trim().replace(/^"|"$/g, ""));
      return {
        id: nextRowId++,
        name: cols[0] ?? "",
        description: cols[1] ?? "",
        category: cols[2] ?? "",
        material: cols[3] ?? "",
        color: cols[4] ?? "",
        quantity: parseInt(cols[5] ?? "1", 10) || 1,
      };
    }).filter((r) => r.name.trim());

    if (newRows.length > 0) {
      const existing = rows.filter((r) => r.name.trim());
      rows = [...existing, ...newRows];
      showCsvImport = false;
    }
  }

  function resetForm() {
    nextRowId = 1;
    rows = [emptyRow(), emptyRow(), emptyRow()];
    parent = null;
    savedCount = null;
    error = null;
  }
</script>

<div class="px-4 pt-[env(safe-area-inset-top)]">
  <div class="pt-4 pb-4">
    <h1 class="text-2xl font-bold">Batch Add Items</h1>
    <p class="text-sm text-text-secondary mt-1">
      Add multiple items at once to the same location
    </p>
  </div>

  <!-- Success state -->
  {#if savedCount !== null}
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
      <p class="text-lg font-semibold mb-2">{savedCount} items saved!</p>
      <div class="flex gap-3 mt-4">
        <button
          onclick={resetForm}
          class="px-6 py-3 bg-surface-raised hover:bg-surface-hover rounded-lg transition-colors min-h-[44px]"
        >
          Add More
        </button>
        <button
          onclick={() => goto("/")}
          class="px-6 py-3 bg-primary hover:bg-primary-hover text-white font-semibold rounded-lg transition-colors min-h-[44px]"
        >
          Done
        </button>
      </div>
    </div>
  {:else}
    <div class="space-y-4">
      <!-- Parent picker -->
      <div>
        <span class="block text-sm text-text-secondary mb-1">
          Destination *
        </span>
        <button
          onclick={() => showPicker = true}
          class="w-full px-3 py-2 bg-surface-raised rounded-lg text-left min-h-[44px] flex items-center justify-between"
        >
          <span class={parent ? "text-text-primary" : "text-text-muted"}>
            {parent
              ? `${parent.name} (${parent.type})`
              : "Select where to store these items"}
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

      {#if error}
        <div class="p-3 bg-danger/10 text-danger rounded-lg text-sm">
          {error}
        </div>
      {/if}

      <!-- CSV Import toggle -->
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium text-text-secondary">
          {validRows.length} item{validRows.length !== 1 ? "s" : ""} ready
        </span>
        <button
          onclick={() => showCsvImport = !showCsvImport}
          class="text-sm text-primary hover:text-primary-hover min-h-[32px]"
        >
          {showCsvImport ? "Hide CSV" : "Import CSV"}
        </button>
      </div>

      <!-- CSV Import area -->
      {#if showCsvImport}
        <div class="bg-surface-raised rounded-lg p-3">
          <p class="text-xs text-text-secondary mb-2">
            Paste CSV: name, description, category, material, color, quantity
          </p>
          <textarea
            rows="4"
            class="w-full px-3 py-2 bg-surface rounded-lg text-text-primary text-sm font-mono outline-none focus:ring-2 focus:ring-primary/50 resize-none"
            placeholder={"Hammer, Claw hammer, Tools, Metal, Silver, 1\nScrewdriver, Phillips head, Tools, Metal, Yellow, 2"}
            onpaste={(e) => {
              const text = e.clipboardData?.getData("text/plain");
              if (text) {
                e.preventDefault();
                handleCsvImport(text);
              }
            }}
            onchange={(e) => handleCsvImport(e.currentTarget.value)}
          ></textarea>
        </div>
      {/if}

      <!-- Item rows -->
      <div class="space-y-3">
        {#each rows as row (row.id)}
          <div class="bg-surface-raised rounded-lg p-3">
            <div class="flex items-center gap-2 mb-2">
              <input
                type="text"
                value={row.name}
                oninput={(e) => updateRow(row.id, "name", e.currentTarget.value)}
                class="flex-1 px-3 py-2 bg-surface rounded-lg text-text-primary outline-none focus:ring-2 focus:ring-primary/50 min-h-[44px] text-sm"
                placeholder="Item name *"
              />
              <button
                onclick={() => removeRow(row.id)}
                class="p-2 text-text-muted hover:text-danger min-h-[44px] min-w-[44px] flex items-center justify-center flex-shrink-0"
                title="Remove row"
              >
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>
            <div class="grid grid-cols-2 gap-2">
              <input
                type="text"
                value={row.category}
                oninput={(e) => updateRow(row.id, "category", e.currentTarget.value)}
                class="px-3 py-1.5 bg-surface rounded-lg text-text-primary outline-none focus:ring-2 focus:ring-primary/50 text-sm min-h-[36px]"
                placeholder="Category"
              />
              <input
                type="text"
                value={row.material}
                oninput={(e) => updateRow(row.id, "material", e.currentTarget.value)}
                class="px-3 py-1.5 bg-surface rounded-lg text-text-primary outline-none focus:ring-2 focus:ring-primary/50 text-sm min-h-[36px]"
                placeholder="Material"
              />
              <input
                type="text"
                value={row.color}
                oninput={(e) => updateRow(row.id, "color", e.currentTarget.value)}
                class="px-3 py-1.5 bg-surface rounded-lg text-text-primary outline-none focus:ring-2 focus:ring-primary/50 text-sm min-h-[36px]"
                placeholder="Color"
              />
              <input
                type="number"
                value={row.quantity}
                oninput={(e) => updateRow(row.id, "quantity", parseInt(e.currentTarget.value, 10) || 1)}
                class="px-3 py-1.5 bg-surface rounded-lg text-text-primary outline-none focus:ring-2 focus:ring-primary/50 text-sm min-h-[36px]"
                placeholder="Qty"
                min="1"
              />
            </div>
          </div>
        {/each}
      </div>

      <!-- Add row -->
      <button
        onclick={addRow}
        class="w-full py-2 border-2 border-dashed border-border hover:border-primary/50 rounded-lg text-text-muted hover:text-primary transition-colors min-h-[44px] text-sm"
      >
        + Add another item
      </button>

      <!-- Save -->
      <div class="flex gap-3 pt-2 pb-4">
        <button
          onclick={() => goto("/add")}
          class="flex-1 py-3 bg-surface-raised hover:bg-surface-hover rounded-lg transition-colors min-h-[44px]"
        >
          Single Add
        </button>
        <button
          onclick={handleSave}
          disabled={saving || validRows.length === 0 || !parent}
          class="flex-1 py-3 bg-primary hover:bg-primary-hover text-white font-semibold rounded-lg transition-colors min-h-[44px] disabled:opacity-50 flex items-center justify-center gap-2"
        >
          {#if saving}
            <LoadingSpinner />
          {:else}
            Save {validRows.length} Items
          {/if}
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
</div>
