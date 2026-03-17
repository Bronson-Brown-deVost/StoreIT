<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { flushPendingQueue, getPendingQueue } from "~/lib/offlineQueue";

  let online = $state(navigator.onLine);
  let showReconnected = $state(false);
  let pendingCount = $state(0);

  let reconnectTimer: ReturnType<typeof setTimeout> | undefined;

  const handleOnline = () => {
    online = true;
    showReconnected = true;
    flushPendingQueue().then((remaining) => {
      pendingCount = remaining;
    });
    reconnectTimer = setTimeout(() => (showReconnected = false), 3000);
  };

  const handleOffline = () => {
    online = false;
    showReconnected = false;
  };

  const checkPendingQueue = () => {
    pendingCount = getPendingQueue().length;
  };

  onMount(() => {
    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);
    window.addEventListener("storeit:queue-updated", checkPendingQueue);
    checkPendingQueue();
  });

  onDestroy(() => {
    window.removeEventListener("online", handleOnline);
    window.removeEventListener("offline", handleOffline);
    window.removeEventListener("storeit:queue-updated", checkPendingQueue);
    if (reconnectTimer) clearTimeout(reconnectTimer);
  });
</script>

{#if !online}
  <div class="fixed top-0 left-0 right-0 z-[60] bg-warning text-warning-text text-center py-1 text-sm font-medium pt-[env(safe-area-inset-top)]">
    Offline — browsing cached data
  </div>
{/if}

{#if showReconnected}
  <div class="fixed top-0 left-0 right-0 z-[60] bg-success text-white text-center py-1 text-sm font-medium pt-[env(safe-area-inset-top)]">
    Back online
  </div>
{/if}

{#if online && pendingCount > 0}
  <div class="fixed top-0 left-0 right-0 z-[60] bg-primary text-white text-center py-1 text-sm font-medium pt-[env(safe-area-inset-top)]">
    Syncing {pendingCount} pending {pendingCount === 1 ? "change" : "changes"}...
  </div>
{/if}
