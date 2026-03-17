<script lang="ts">
  import { onMount } from "svelte";
  import { goto, type RouteResult } from "@mateothegreat/svelte5-router";

  let { route }: { route: RouteResult } = $props();
  import {
    resolveNfcTag,
    resolveNfcTagByUid,
    registerAndAssignNfcTag,
    assignNfcTag,
    type NfcUidResolveResponse,
  } from "~/api";
  import LoadingSpinner from "~/components/LoadingSpinner.svelte";
  import ParentPicker from "~/components/ParentPicker.svelte";
  import type { SelectedParent } from "~/components/ParentPicker.svelte";

  // Determine mode from URL
  // /nfc/tag?uid=... -> UID mode
  // /nfc/:tagUri -> legacy mode
  let isUidMode = $state(false);
  let tagUri = $state("");
  let uid = $state("");

  // Shared state
  let resolvedLoading = $state(true);
  let resolvedError = $state(false);
  let assigning = $state(false);
  let error = $state<string | null>(null);
  let showPicker = $state(false);

  // UID mode state
  let resolvedData = $state<NfcUidResolveResponse | null>(null);

  function parseUrl() {
    const pathname = window.location.pathname;
    const parts = pathname.split("/");
    // /nfc/tag?uid=... or /nfc/:tagUri
    const segment = parts[2] || "";
    const params = new URLSearchParams(window.location.search);

    if (segment === "tag" && params.get("uid")) {
      isUidMode = true;
      uid = params.get("uid") || "";
    } else {
      isUidMode = false;
      tagUri = decodeURIComponent(segment);
    }
  }

  async function resolveLegacy() {
    resolvedLoading = true;
    resolvedError = false;
    try {
      const result = await resolveNfcTag(tagUri);
      if (result.entity_type === "container") {
        goto(`/containers/${result.entity_id}`);
      } else if (result.entity_type === "location") {
        goto(`/locations/${result.entity_id}`);
      }
    } catch {
      resolvedError = true;
    } finally {
      resolvedLoading = false;
    }
  }

  async function resolveUid() {
    if (!uid) {
      resolvedError = true;
      resolvedLoading = false;
      return;
    }
    resolvedLoading = true;
    resolvedError = false;
    try {
      const result = await resolveNfcTagByUid(uid);
      resolvedData = result;
      // Auto-navigate if assigned
      if (result.status === "assigned" && result.entity_type && result.entity_id) {
        if (result.entity_type === "container") {
          goto(`/containers/${result.entity_id}`);
        } else if (result.entity_type === "location") {
          goto(`/locations/${result.entity_id}`);
        }
      }
    } catch {
      resolvedError = true;
    } finally {
      resolvedLoading = false;
    }
  }

  async function handleAssign(parent: SelectedParent) {
    assigning = true;
    error = null;
    showPicker = false;
    try {
      if (resolvedData?.status === "unassigned" && resolvedData.tag_id) {
        await assignNfcTag(resolvedData.tag_id, {
          entity_type: parent.type,
          entity_id: parent.id,
        });
      } else {
        await registerAndAssignNfcTag({
          tag_uri: uid,
          entity_type: parent.type,
          entity_id: parent.id,
        });
      }
      await resolveUid();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to link tag";
    } finally {
      assigning = false;
    }
  }

  onMount(() => {
    parseUrl();
    if (isUidMode) {
      resolveUid();
    } else {
      resolveLegacy();
    }
  });
</script>

<div class="px-4 pt-[env(safe-area-inset-top)]">
  <div class="pt-4 flex flex-col items-center justify-center py-12 text-center">
    {#if !isUidMode}
      <!-- Legacy mode -->
      {#if resolvedLoading}
        <div class="flex flex-col items-center gap-4">
          <LoadingSpinner />
          <p class="text-text-secondary">Looking up NFC tag...</p>
        </div>
      {:else if resolvedError}
        <svg
          class="w-16 h-16 text-text-muted mb-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <path d="M6 8.32a7.43 7.43 0 0 1 0 7.36" />
          <path d="M9.46 6.21a11.76 11.76 0 0 1 0 11.58" />
          <path d="M12.91 4.1a16.1 16.1 0 0 1 0 15.8" />
          <path d="M16.37 2a20.16 20.16 0 0 1 0 20" />
        </svg>
        <h1 class="text-2xl font-bold mb-2">Unknown Tag</h1>
        <p class="text-text-secondary mb-4">
          This NFC tag is not registered or not assigned to any entity.
        </p>
        <code class="text-sm text-text-muted bg-surface-raised px-3 py-1.5 rounded mb-6">
          {tagUri}
        </code>
        <button
          onclick={() => goto("/")}
          class="px-6 py-3 bg-primary hover:bg-primary-hover text-white rounded-lg transition-colors min-h-[44px]"
        >
          Go Home
        </button>
      {/if}
    {:else}
      <!-- UID mode -->
      {#if resolvedLoading || assigning}
        <div class="flex flex-col items-center gap-4">
          <LoadingSpinner />
          <p class="text-text-secondary">
            {assigning ? "Linking tag..." : "Looking up NFC tag..."}
          </p>
        </div>
      {:else if resolvedError}
        <svg
          class="w-16 h-16 text-text-muted mb-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <path d="M6 8.32a7.43 7.43 0 0 1 0 7.36" />
          <path d="M9.46 6.21a11.76 11.76 0 0 1 0 11.58" />
          <path d="M12.91 4.1a16.1 16.1 0 0 1 0 15.8" />
          <path d="M16.37 2a20.16 20.16 0 0 1 0 20" />
        </svg>
        <h1 class="text-2xl font-bold mb-2">Error</h1>
        <p class="text-text-secondary mb-4">
          Could not look up this NFC tag.
        </p>
        <button
          onclick={() => goto("/")}
          class="px-6 py-3 bg-primary hover:bg-primary-hover text-white rounded-lg transition-colors min-h-[44px]"
        >
          Go Home
        </button>
      {:else if resolvedData}
        {#if resolvedData.status === "assigned"}
          <div class="flex flex-col items-center gap-4">
            <LoadingSpinner />
            <p class="text-text-secondary">Navigating...</p>
          </div>
        {/if}

        {#if resolvedData.status === "unknown" || resolvedData.status === "unassigned"}
          <svg
            class="w-16 h-16 text-text-muted mb-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <path d="M6 8.32a7.43 7.43 0 0 1 0 7.36" />
            <path d="M9.46 6.21a11.76 11.76 0 0 1 0 11.58" />
            <path d="M12.91 4.1a16.1 16.1 0 0 1 0 15.8" />
            <path d="M16.37 2a20.16 20.16 0 0 1 0 20" />
          </svg>
          <h1 class="text-2xl font-bold mb-2">New NFC Tag</h1>
          <p class="text-text-secondary mb-2">
            This tag isn't linked to anything yet.
          </p>
          <code class="text-sm text-text-muted bg-surface-raised px-3 py-1.5 rounded mb-6">
            {uid}
          </code>

          <button
            onclick={() => showPicker = true}
            class="px-6 py-3 bg-primary hover:bg-primary-hover text-white rounded-lg transition-colors min-h-[44px] mb-4"
          >
            Link to Location or Container
          </button>

          {#if error}
            <p class="text-red-400 text-sm mt-2">{error}</p>
          {/if}

          <button
            onclick={() => goto("/")}
            class="mt-4 text-sm text-text-muted hover:text-text-secondary min-h-[44px]"
          >
            Go Home
          </button>
        {/if}
      {/if}
    {/if}
  </div>

  {#if showPicker}
    <ParentPicker
      onSelect={handleAssign}
      onClose={() => showPicker = false}
    />
  {/if}
</div>
