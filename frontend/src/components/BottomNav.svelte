<script lang="ts">
  import { route } from "@mateothegreat/svelte5-router";
  import { auth } from "~/lib/auth.svelte";

  let pathname = $state(window.location.pathname);

  function updatePathname() {
    pathname = window.location.pathname;
  }

  $effect(() => {
    window.addEventListener("popstate", updatePathname);
    window.addEventListener("pushState", updatePathname);
    return () => {
      window.removeEventListener("popstate", updatePathname);
      window.removeEventListener("pushState", updatePathname);
    };
  });

  function isActive(path: string): boolean {
    if (path === "/") return pathname === "/";
    return pathname.startsWith(path);
  }

  function linkClass(path: string): string {
    return `flex flex-col items-center gap-1 py-2 px-3 min-w-[64px] transition-colors ${
      isActive(path)
        ? "text-primary"
        : "text-text-muted hover:text-text-secondary"
    }`;
  }
</script>

<nav class="fixed bottom-0 left-0 right-0 bg-surface-raised border-t border-border pb-[env(safe-area-inset-bottom)] z-50">
  <div class="flex justify-around items-center max-w-lg mx-auto">
    <a use:route href="/" class={linkClass("/")}>
      <svg
        class="w-6 h-6"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
        <polyline points="9 22 9 12 15 12 15 22" />
      </svg>
      <span class="text-xs">Browse</span>
    </a>

    <a use:route href="/search" class={linkClass("/search")}>
      <svg
        class="w-6 h-6"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <span class="text-xs">Search</span>
    </a>

    <a use:route href="/add" class={linkClass("/add")}>
      <svg
        class="w-6 h-6"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="16" />
        <line x1="8" y1="12" x2="16" y2="12" />
      </svg>
      <span class="text-xs">Add</span>
    </a>

    {#if auth.user?.is_admin}
      <a use:route href="/admin" class={linkClass("/admin")}>
        <svg
          class="w-6 h-6"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197" />
        </svg>
        <span class="text-xs">Admin</span>
      </a>
    {/if}

    <a use:route href="/settings" class={linkClass("/settings")}>
      <svg
        class="w-6 h-6"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.32 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z" />
      </svg>
      <span class="text-xs">Settings</span>
    </a>
  </div>
</nav>
