import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Add Item", () => {
  test("add item page loads with capture step", async ({ page }) => {
    await page.goto("/add");
    await expect(page.getByRole("heading", { name: "Add Item" })).toBeVisible();
    // The capture step shows "Take Photo" button
    await expect(page.getByText(/take a photo/i)).toBeVisible();
  });

  test("add item from container page links to add with container param", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("Box B", "location", loc.id);

    // Navigate to the container page
    await page.goto(`/containers/${container.id}`);
    await expect(page.getByRole("heading", { name: container.name })).toBeVisible();

    // Click the floating "Add Item Here" link (aria-label="Add Item Here")
    await page.getByRole("link", { name: "Add Item Here" }).click();
    await expect(page).toHaveURL(new RegExp(`/add\\?container=${container.id}`));
    await expect(page.getByRole("heading", { name: "Add Item" })).toBeVisible();
  });
});
