import { test, expect } from "@playwright/test";

test.describe("Admin Page", () => {
  test("admin page shows heading and sections", async ({ page }) => {
    await page.goto("/admin");
    await expect(page.getByRole("heading", { name: "Admin" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Backup" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Restore" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Groups" })).toBeVisible();
  });

  test("settings section shows image storage path", async ({ page }) => {
    await page.goto("/admin");
    // Wait for Settings section to load (requires API call)
    await expect(page.getByText("Image Storage Path").first()).toBeVisible({ timeout: 10000 });
  });
});
