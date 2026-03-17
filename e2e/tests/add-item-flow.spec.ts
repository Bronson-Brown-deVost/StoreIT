import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Add Item Flow", () => {
  test("Batch Add link navigates to batch page", async ({ page }) => {
    await page.goto("/add");
    await page.getByText("Batch Add").click();
    await expect(page).toHaveURL(/\/add\/batch/);
    await expect(page.getByRole("heading", { name: "Batch Add Items" })).toBeVisible();
  });

  test("capture step shows Take Photo button", async ({ page }) => {
    await page.goto("/add");
    await expect(page.getByRole("heading", { name: "Add Item" })).toBeVisible();
    await expect(page.getByText("Take Photo")).toBeVisible();
  });

  test("selecting a file shows review step with form fields", async ({ page }) => {
    await page.goto("/add");

    // Create a tiny PNG buffer and upload via the hidden file input
    const fileInput = page.locator('input[type="file"]');
    const buffer = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==", "base64");
    await fileInput.setInputFiles({ name: "test.png", mimeType: "image/png", buffer });

    // Review step should show form fields
    await expect(page.getByPlaceholder("Item name")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Save Item")).toBeVisible();
  });

  test("add description toggle shows textarea", async ({ page }) => {
    await page.goto("/add");

    const fileInput = page.locator('input[type="file"]');
    const buffer = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==", "base64");
    await fileInput.setInputFiles({ name: "test.png", mimeType: "image/png", buffer });

    await expect(page.getByText("+ Add description")).toBeVisible({ timeout: 5000 });
    await page.getByText("+ Add description").click();
    await expect(page.getByPlaceholder("Description")).toBeVisible();
  });
});
