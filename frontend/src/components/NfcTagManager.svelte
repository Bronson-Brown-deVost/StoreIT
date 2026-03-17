<script lang="ts">
  import {
    createNfcTag,
    assignNfcTag,
    unassignNfcTag,
    deleteNfcTag,
    listNfcTags,
    registerAndAssignNfcTag,
    type NfcTagResponse,
  } from "~/api";

  let { entityType, entityId }: {
    entityType: "location" | "container";
    entityId: string;
  } = $props();

  let tags = $state<NfcTagResponse[]>([]);
  let tagsLoading = $state(true);
  let showAdd = $state(false);
  let tagUri = $state("");
  let adding = $state(false);
  let showAssign = $state(false);
  let unassignedTags = $state<NfcTagResponse[]>([]);
  let scanning = $state(false);
  let scanError = $state<string | null>(null);

  const hasWebNfc = () => "NDEFReader" in window;

  async function fetchTags() {
    tagsLoading = true;
    try {
      const all = await listNfcTags();
      tags = all.filter(
        (t) =>
          t.entity_type === entityType && t.entity_id === entityId,
      );
    } catch {
      tags = [];
    } finally {
      tagsLoading = false;
    }
  }

  // Refetch when entityType or entityId changes
  $effect(() => {
    entityType;
    entityId;
    fetchTags();
  });

  async function handleRegisterAndAssign() {
    if (!tagUri.trim()) return;
    adding = true;
    try {
      await registerAndAssignNfcTag({
        tag_uri: tagUri.trim(),
        entity_type: entityType,
        entity_id: entityId,
      });
      tagUri = "";
      showAdd = false;
      fetchTags();
    } catch {
      // error handled silently
    } finally {
      adding = false;
    }
  }

  async function handleScanAndAssign() {
    scanError = null;
    scanning = true;
    try {
      const ndef = new (window as any).NDEFReader();
      const ac = new AbortController();

      await ndef.scan({ signal: ac.signal });

      await new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(() => {
          ac.abort();
          reject(new Error("Scan timed out — no tag detected"));
        }, 15000);

        ndef.addEventListener(
          "reading",
          async (event: any) => {
            clearTimeout(timeout);
            ac.abort();
            const uid: string = event.serialNumber?.replace(/:/g, "").toUpperCase() || "";
            if (!uid) {
              reject(new Error("Could not read tag UID"));
              return;
            }
            try {
              await registerAndAssignNfcTag({
                tag_uri: uid,
                entity_type: entityType,
                entity_id: entityId,
              });
              fetchTags();
              resolve();
            } catch (err) {
              reject(err);
            }
          },
          { signal: ac.signal }
        );
      });
    } catch (err: any) {
      if (err.name === "NotAllowedError") {
        scanError = "NFC permission denied.";
      } else {
        scanError = err.message || "Scan failed";
      }
    } finally {
      scanning = false;
    }
  }

  async function handleShowAssign() {
    const all = await listNfcTags();
    const free = all.filter((t) => !t.entity_type);
    unassignedTags = free;
    showAssign = true;
  }

  async function handleAssignExisting(tagId: string) {
    await assignNfcTag(tagId, {
      entity_type: entityType,
      entity_id: entityId,
    });
    showAssign = false;
    fetchTags();
  }

  async function handleUnassign(tagId: string) {
    await unassignNfcTag(tagId);
    fetchTags();
  }

  async function handleDelete(tagId: string) {
    await deleteNfcTag(tagId);
    fetchTags();
  }
</script>

<div class="bg-surface-raised rounded-lg p-4">
  <div class="flex items-center justify-between mb-3">
    <h2 class="text-sm font-medium text-text-secondary uppercase tracking-wider">
      NFC Tags
    </h2>
    <div class="flex gap-2">
      {#if hasWebNfc()}
        <button
          onclick={handleScanAndAssign}
          disabled={scanning}
          class="text-xs text-primary hover:text-primary-hover min-h-[32px] px-2 disabled:opacity-50"
        >
          {scanning ? "Scanning..." : "Scan"}
        </button>
      {/if}
      <button
        onclick={handleShowAssign}
        class="text-xs text-primary hover:text-primary-hover min-h-[32px] px-2"
      >
        Assign
      </button>
      <button
        onclick={() => (showAdd = true)}
        class="text-xs text-primary hover:text-primary-hover min-h-[32px] px-2"
      >
        + Manual
      </button>
    </div>
  </div>

  {#if scanning}
    <div class="mb-3 p-3 bg-primary/10 rounded-lg text-sm text-text-secondary text-center">
      Hold your phone near an NFC tag...
    </div>
  {/if}

  {#if scanError}
    <div class="mb-3 text-red-400 text-sm">{scanError}</div>
  {/if}

  {#if tags.length > 0}
    <div class="space-y-2">
      {#each tags as tag (tag.id)}
        <div class="flex items-center justify-between py-2 border-b border-border last:border-0">
          <code class="text-sm text-text-primary truncate flex-1 mr-2">
            {tag.tag_uri}
          </code>
          <div class="flex gap-1 flex-shrink-0">
            <button
              onclick={() => handleUnassign(tag.id)}
              class="text-xs text-text-muted hover:text-danger px-2 py-1 min-h-[32px]"
            >
              Unassign
            </button>
            <button
              onclick={() => handleDelete(tag.id)}
              class="text-xs text-text-muted hover:text-danger px-2 py-1 min-h-[32px]"
            >
              Delete
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if tags.length === 0 && !tagsLoading}
    <p class="text-sm text-text-muted">No NFC tags assigned</p>
  {/if}

  <!-- Register new tag manually -->
  {#if showAdd}
    <div class="mt-3 flex gap-2">
      <input
        type="text"
        value={tagUri}
        oninput={(e) => (tagUri = e.currentTarget.value)}
        placeholder="Tag UID or URI"
        class="flex-1 px-3 py-2 bg-surface rounded-lg text-text-primary placeholder:text-text-muted outline-none focus:ring-2 focus:ring-primary/50 text-sm min-h-[44px]"
      />
      <button
        onclick={handleRegisterAndAssign}
        disabled={adding || !tagUri.trim()}
        class="px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm transition-colors min-h-[44px] disabled:opacity-50"
      >
        Add
      </button>
      <button
        onclick={() => (showAdd = false)}
        class="px-3 py-2 text-text-muted hover:text-text-secondary min-h-[44px]"
      >
        Cancel
      </button>
    </div>
  {/if}

  <!-- Assign existing unassigned tag -->
  {#if showAssign}
    <div class="mt-3">
      {#if unassignedTags.length > 0}
        <p class="text-sm text-text-secondary mb-2">Select a tag:</p>
        <div class="space-y-1">
          {#each unassignedTags as tag (tag.id)}
            <button
              onclick={() => handleAssignExisting(tag.id)}
              class="w-full text-left px-3 py-2 bg-surface hover:bg-surface-hover rounded-lg text-sm transition-colors min-h-[44px]"
            >
              <code>{tag.tag_uri}</code>
            </button>
          {/each}
        </div>
      {:else}
        <p class="text-sm text-text-muted">
          No unassigned tags available.
          <button
            onclick={() => {
              showAssign = false;
              showAdd = true;
            }}
            class="text-primary"
          >
            Add one manually
          </button>
        </p>
      {/if}
      <button
        onclick={() => (showAssign = false)}
        class="mt-2 text-xs text-text-muted hover:text-text-secondary min-h-[32px]"
      >
        Cancel
      </button>
    </div>
  {/if}
</div>
