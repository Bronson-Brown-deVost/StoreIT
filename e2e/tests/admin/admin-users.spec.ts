import { test, expect } from "@playwright/test";

test.describe("Admin Users", () => {
  test("users section lists seeded admin user", async ({ page }) => {
    await page.goto("/admin");
    // Scroll to users section
    await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
    // The admin user should be listed
    await expect(page.getByText("Test Admin")).toBeVisible();
    await expect(page.getByText("admin@test.com")).toBeVisible();
  });

  test("create user button shows form", async ({ page }) => {
    await page.goto("/admin");
    await page.getByRole("button", { name: "Create User" }).click();

    // Form should appear
    await expect(page.getByPlaceholder("Username")).toBeVisible();
    await expect(page.getByPlaceholder("Email")).toBeVisible();
    await expect(page.getByPlaceholder("Display Name")).toBeVisible();
    await expect(page.getByPlaceholder("Password")).toBeVisible();
  });

  test("create user and see in list", async ({ page }) => {
    await page.goto("/admin");
    await page.getByRole("button", { name: "Create User" }).click();

    const suffix = Date.now();
    await page.getByPlaceholder("Username").fill(`testuser${suffix}`);
    await page.getByPlaceholder("Email").fill(`test${suffix}@example.com`);
    await page.getByPlaceholder("Display Name").fill(`Test User ${suffix}`);
    await page.getByPlaceholder("Password").fill("password123");

    // Click the Create button in the user creation form (use first matching)
    await page.getByRole("button", { name: "Create", exact: true }).first().click();

    // User should appear in the list
    await expect(page.getByText(`Test User ${suffix}`)).toBeVisible({ timeout: 5000 });
  });

  test("reset password shows form", async ({ page }) => {
    await page.goto("/admin");

    // Find a "Reset Password" button and click it
    const resetBtn = page.getByRole("button", { name: "Reset Password" }).first();
    await resetBtn.click();

    // Should show the password reset form
    await expect(page.getByPlaceholder("New password")).toBeVisible();
    await expect(page.getByRole("button", { name: "Reset", exact: true })).toBeVisible();
  });
});
