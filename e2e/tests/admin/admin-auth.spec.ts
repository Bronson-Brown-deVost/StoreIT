import { test, expect } from "@playwright/test";

test.describe("Admin Auth", () => {
  test("auth mode endpoint returns local", async ({ request }) => {
    const res = await request.get("/api/v1/auth/mode");
    expect(res.ok()).toBeTruthy();
    const body = await res.json();
    expect(body.mode).toBe("local");
  });

  test("unauthenticated user sees login form", async ({ browser }) => {
    const context = await browser.newContext({
      storageState: undefined,
      serviceWorkers: "block",
    });
    const page = await context.newPage();

    // Navigate to the app - should show login form since there's no session
    await page.goto("/");
    // The app checks auth mode and shows LocalLoginForm
    await expect(page.getByRole("button", { name: "Sign In" })).toBeVisible({ timeout: 10000 });

    await context.close();
  });

  test("admin user can access admin page", async ({ page }) => {
    await page.goto("/admin");
    // Should NOT show "Access Denied"
    await expect(page.getByText("Access Denied")).not.toBeVisible({ timeout: 5000 });
    await expect(page.getByRole("heading", { name: "Admin" })).toBeVisible();
  });
});
