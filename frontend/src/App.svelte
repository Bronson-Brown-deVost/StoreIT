<script lang="ts">
  import { Router, type RouteConfig, route } from "@mateothegreat/svelte5-router";
  import { auth } from "~/lib/auth.svelte";
  import BottomNav from "~/components/BottomNav.svelte";
  import OfflineIndicator from "~/components/OfflineIndicator.svelte";
  import LoadingSpinner from "~/components/LoadingSpinner.svelte";
  import LocalLoginForm from "~/components/LocalLoginForm.svelte";
  import HomePage from "~/pages/HomePage.svelte";
  import SearchPage from "~/pages/SearchPage.svelte";
  import AddItemPage from "~/pages/AddItemPage.svelte";
  import BatchAddItemPage from "~/pages/BatchAddItemPage.svelte";
  import LocationPage from "~/pages/LocationPage.svelte";
  import ContainerPage from "~/pages/ContainerPage.svelte";
  import ItemDetailPage from "~/pages/ItemDetailPage.svelte";
  import NfcResolvePage from "~/pages/NfcResolvePage.svelte";
  import SettingsPage from "~/pages/SettingsPage.svelte";
  import AdminPage from "~/pages/AdminPage.svelte";

  const routes: RouteConfig[] = [
    { path: "/", component: HomePage },
    { path: "/search$", component: SearchPage },
    { path: "/add/batch$", component: BatchAddItemPage },
    { path: "/add$", component: AddItemPage },
    { path: "/locations/(?<id>[^/]+)", component: LocationPage },
    { path: "/containers/(?<id>[^/]+)", component: ContainerPage },
    { path: "/items/(?<id>[^/]+)", component: ItemDetailPage },
    { path: "/nfc/(?<tagUri>.+)", component: NfcResolvePage },
    { path: "/settings$", component: SettingsPage },
    { path: "/admin$", component: AdminPage },
  ];
</script>

{#if auth.loading}
  <div class="min-h-screen bg-surface flex items-center justify-center">
    <LoadingSpinner />
  </div>
{:else if auth.user}
  <div class="min-h-screen bg-surface text-text-primary pb-20">
    <OfflineIndicator />
    <Router {routes} />
    <BottomNav />
  </div>
{:else}
  <div class="min-h-screen bg-surface text-text-primary flex flex-col items-center justify-center gap-6 px-4">
    <svg class="w-16 h-16 text-primary" viewBox="0 0 64 64" fill="none">
      <rect x="8" y="20" width="48" height="36" rx="4" fill="currentColor" opacity="0.9" />
      <path d="M4 16l28-10 28 10v8H4v-8z" fill="currentColor" />
    </svg>
    <h1 class="text-3xl font-bold">StoreIT</h1>
    <p class="text-text-secondary">Home Inventory Management</p>

    {#if auth.authMode === "local"}
      <LocalLoginForm />
    {:else if auth.authMode === "oidc"}
      <button
        onclick={() => auth.login()}
        class="mt-4 px-8 py-3 bg-primary hover:bg-primary-hover text-white font-semibold rounded-lg transition-colors min-h-[44px]"
      >
        Sign In
      </button>
    {/if}
  </div>
{/if}
