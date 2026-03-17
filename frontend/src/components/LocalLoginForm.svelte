<script lang="ts">
  import { auth } from "~/lib/auth.svelte";
  import { ApiClientError } from "~/api";

  let username = $state("");
  let password = $state("");
  let error = $state<string | null>(null);
  let loading = $state(false);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = null;
    loading = true;
    try {
      await auth.localLogin(username, password);
    } catch (err) {
      if (err instanceof ApiClientError) {
        error = err.message;
      } else {
        error = "An unexpected error occurred";
      }
    } finally {
      loading = false;
    }
  }
</script>

<form onsubmit={handleSubmit} class="w-full max-w-sm flex flex-col gap-4">
  {#if error}
    <div class="bg-red-900/30 border border-red-700 text-red-300 px-4 py-3 rounded-lg text-sm">
      {error}
    </div>
  {/if}

  <div class="flex flex-col gap-1.5">
    <label for="username" class="text-sm text-text-secondary">
      Username
    </label>
    <input
      id="username"
      type="text"
      bind:value={username}
      class="bg-surface-raised border border-border rounded-lg px-4 py-3 text-text-primary placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-primary min-h-[44px]"
      placeholder="Enter your username"
      autocomplete="username"
      required
    />
  </div>

  <div class="flex flex-col gap-1.5">
    <label for="password" class="text-sm text-text-secondary">
      Password
    </label>
    <input
      id="password"
      type="password"
      bind:value={password}
      class="bg-surface-raised border border-border rounded-lg px-4 py-3 text-text-primary placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-primary min-h-[44px]"
      placeholder="Enter your password"
      autocomplete="current-password"
      required
    />
  </div>

  <button
    type="submit"
    disabled={loading}
    class="mt-2 px-8 py-3 bg-primary hover:bg-primary-hover disabled:opacity-50 text-white font-semibold rounded-lg transition-colors min-h-[44px]"
  >
    {loading ? "Signing in..." : "Sign In"}
  </button>
</form>
