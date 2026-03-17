import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Batch Add Items", () => {
  test("page loads with heading and default rows", async ({ page }) => {
    await page.goto("/add/batch");
    await expect(page.getByRole("heading", { name: "Batch Add Items" })).toBeVisible();
    // Should have a destination picker
    await expect(page.getByText("Select where to store these items")).toBeVisible();
    // Should have initial empty rows with placeholder
    await expect(page.getByPlaceholder("Item name *").first()).toBeVisible();
  });

  test("add another item adds a new row", async ({ page }) => {
    await page.goto("/add/batch");
    const initialRows = await page.getByPlaceholder("Item name *").count();
    await page.getByText("+ Add another item").evaluate((el: HTMLElement) => el.click());
    // Wait for DOM to update
    await expect(page.getByPlaceholder("Item name *")).toHaveCount(initialRows + 1, { timeout: 3000 });
  });

  test("Single Add navigates to single add page", async ({ page }) => {
    await page.goto("/add/batch");
    await page.getByText("Single Add").click();
    await expect(page).toHaveURL(/\/add$/);
  });

  test("Import CSV toggle shows textarea", async ({ page }) => {
    await page.goto("/add/batch");
    await page.getByText("Import CSV").click();
    await expect(page.getByPlaceholder(/Hammer/)).toBeVisible();
  });
});
