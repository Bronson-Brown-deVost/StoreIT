import { test, expect } from "@playwright/test";

test.describe("Admin Backup", () => {
  test("backup section shows options and start button", async ({ page }) => {
    await page.goto("/admin");
    await expect(page.getByRole("heading", { name: "Backup" })).toBeVisible();
    await expect(page.getByText("Data only")).toBeVisible();
    await expect(page.getByText("Data + Images")).toBeVisible();
    await expect(page.getByRole("button", { name: "Start Backup" })).toBeVisible();
  });

  test("start backup shows progress and download link", async ({ page }) => {
    await page.goto("/admin");
    await page.getByRole("button", { name: "Start Backup" }).click();

    // Should show progress or complete
    await expect(page.getByText("Download Backup")).toBeVisible({ timeout: 15000 });
  });
});
