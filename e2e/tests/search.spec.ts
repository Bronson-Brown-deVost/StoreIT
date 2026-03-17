import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Search", () => {
  test("search finds item", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const uniqueName = `UniqueWidget_${Date.now()}`;
    await api.createItem(uniqueName, "container", container.id);

    await page.goto("/search");
    await page.getByPlaceholder(/search/i).fill(uniqueName);

    // Wait for search results to load (debounced)
    await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
  });

  test("search shows no results", async ({ page }) => {
    await page.goto("/search");
    await page.getByPlaceholder(/search/i).fill("NonexistentXYZ99999");

    // Wait for the no-results message
    await expect(page.getByText(/no results/i)).toBeVisible({ timeout: 5000 });
  });
});
