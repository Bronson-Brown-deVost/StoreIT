import { test, expect } from "@playwright/test";

test.describe("Auth", () => {
  test("authenticated user sees homepage", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();
  });

  test("unauthenticated user sees login form", async ({ browser }) => {
    // Create a fresh context with no stored auth
    const context = await browser.newContext({
      storageState: undefined,
      serviceWorkers: "block",
    });
    const page = await context.newPage();

    await page.goto("/");
    // In local auth mode, unauthenticated users see the login form
    await expect(
      page.getByRole("button", { name: "Sign In" })
    ).toBeVisible({ timeout: 10000 });

    await context.close();
  });
});
