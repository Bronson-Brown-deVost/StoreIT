import { test, expect } from "@playwright/test";

test.describe("Admin Groups", () => {
  test("groups section lists default group", async ({ page }) => {
    await page.goto("/admin");
    await expect(page.getByRole("heading", { name: "Groups" })).toBeVisible();
    // A "default" group is seeded by migrations
    await expect(page.getByText("default")).toBeVisible();
  });

  test("create group and see in list", async ({ page }) => {
    await page.goto("/admin");

    const groupName = `testgroup-${Date.now()}`;
    await page.getByPlaceholder("New group name").fill(groupName);
    await page.getByRole("button", { name: "Create", exact: true }).last().click();

    await expect(page.getByText(groupName)).toBeVisible({ timeout: 5000 });
  });

  test("clicking group shows members section", async ({ page }) => {
    await page.goto("/admin");

    // Click the "default" group to view members
    await page.getByRole("button", { name: "default" }).click();

    await expect(page.getByText('Members of "default"')).toBeVisible({ timeout: 5000 });
  });
});
