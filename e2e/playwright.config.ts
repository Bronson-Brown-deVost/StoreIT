import { defineConfig, devices } from "@playwright/test";

const chromiumLaunchOptions = {
  executablePath: process.env.PLAYWRIGHT_CHROMIUM_PATH || undefined,
};

export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: "list",
  globalSetup: "./setup/global-setup.ts",
  globalTeardown: "./setup/global-teardown.ts",
  projects: [
    // --- Local auth projects (default user, port 3100) ---
    {
      name: "chromium",
      testIgnore: /admin\//,
      use: {
        browserName: "chromium",
        baseURL: "http://127.0.0.1:3100",
        storageState: "./setup/.auth-state.json",
        trace: "on-first-retry",
        serviceWorkers: "block",
        launchOptions: chromiumLaunchOptions,
      },
    },
    {
      name: "android-phone",
      testIgnore: /admin\//,
      use: {
        ...devices["Pixel 7"],
        browserName: "chromium",
        baseURL: "http://127.0.0.1:3100",
        storageState: "./setup/.auth-state.json",
        trace: "on-first-retry",
        serviceWorkers: "block",
        launchOptions: chromiumLaunchOptions,
      },
    },
    {
      name: "iphone",
      testIgnore: /admin\//,
      use: {
        ...devices["iPhone 14"],
        browserName: "chromium",
        baseURL: "http://127.0.0.1:3100",
        storageState: "./setup/.auth-state.json",
        trace: "on-first-retry",
        serviceWorkers: "block",
        launchOptions: chromiumLaunchOptions,
      },
    },
    {
      name: "android-tablet",
      testIgnore: /admin\//,
      use: {
        ...devices["Galaxy Tab S4"],
        browserName: "chromium",
        baseURL: "http://127.0.0.1:3100",
        storageState: "./setup/.auth-state.json",
        trace: "on-first-retry",
        serviceWorkers: "block",
        launchOptions: chromiumLaunchOptions,
      },
    },
    {
      name: "ipad",
      testIgnore: /admin\//,
      use: {
        ...devices["iPad (gen 7)"],
        browserName: "chromium",
        baseURL: "http://127.0.0.1:3100",
        storageState: "./setup/.auth-state.json",
        trace: "on-first-retry",
        serviceWorkers: "block",
        launchOptions: chromiumLaunchOptions,
      },
    },
    // --- Admin project (local auth, port 3101) ---
    {
      name: "admin",
      testMatch: /admin\//,
      use: {
        browserName: "chromium",
        baseURL: "http://127.0.0.1:3101",
        storageState: "./setup/.admin-auth-state.json",
        trace: "on-first-retry",
        serviceWorkers: "block",
        launchOptions: chromiumLaunchOptions,
      },
    },
  ],
});
