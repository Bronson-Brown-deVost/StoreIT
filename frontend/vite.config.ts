import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { VitePWA } from "vite-plugin-pwa";

export default defineConfig({
  plugins: [
    svelte(),
    tailwindcss(),
    VitePWA({
      registerType: "autoUpdate",
      manifest: {
        name: "StoreIT",
        short_name: "StoreIT",
        description: "Home Inventory Management",
        theme_color: "#1e293b",
        background_color: "#0f172a",
        display: "standalone",
        icons: [
          { src: "/icon-192.png", sizes: "192x192", type: "image/png" },
          { src: "/icon-512.png", sizes: "512x512", type: "image/png" },
        ],
      },
      workbox: {
        navigateFallbackDenylist: [/^\/api\//],
        globPatterns: ["**/*.{js,css,html,svg,png,ico,woff,woff2}"],
        runtimeCaching: [
          {
            urlPattern: /\/api\/v1\/(locations|containers|items|photos\/by-entity)/,
            handler: "NetworkFirst",
            options: {
              cacheName: "api-browse-cache",
              expiration: { maxEntries: 200, maxAgeSeconds: 60 * 60 },
              cacheableResponse: { statuses: [0, 200] },
            },
          },
          {
            urlPattern: /\/api\/v1\/auth/,
            handler: "NetworkFirst",
            options: {
              cacheName: "api-auth-cache",
              expiration: { maxEntries: 5, maxAgeSeconds: 60 * 5 },
              cacheableResponse: { statuses: [0, 200] },
            },
          },
          {
            urlPattern: /\/api\/v1\/search/,
            handler: "NetworkFirst",
            options: {
              cacheName: "api-search-cache",
              expiration: { maxEntries: 20, maxAgeSeconds: 60 * 5 },
              cacheableResponse: { statuses: [0, 200] },
            },
          },
          {
            urlPattern: /\/api\/v1\/photos\/[^/]+\/file/,
            handler: "CacheFirst",
            options: {
              cacheName: "api-photos-full-cache",
              expiration: { maxEntries: 100, maxAgeSeconds: 60 * 60 * 24 * 30 },
              cacheableResponse: { statuses: [0, 200] },
            },
          },
          {
            urlPattern: /\/api\/v1\/photos\/[^/]+\/large/,
            handler: "CacheFirst",
            options: {
              cacheName: "api-photos-large-cache",
              expiration: { maxEntries: 500, maxAgeSeconds: 60 * 60 * 24 * 30 },
              cacheableResponse: { statuses: [0, 200] },
            },
          },
          {
            urlPattern: /\/api\/v1\/photos\/[^/]+\/thumbnail/,
            handler: "CacheFirst",
            options: {
              cacheName: "api-photos-thumb-cache",
              expiration: { maxEntries: 1000, maxAgeSeconds: 60 * 60 * 24 * 30 },
              cacheableResponse: { statuses: [0, 200] },
            },
          },
        ],
      },
    }),
  ],
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:8080",
        changeOrigin: true,
      },
    },
  },
  resolve: {
    alias: {
      "~": "/src",
    },
  },
});
