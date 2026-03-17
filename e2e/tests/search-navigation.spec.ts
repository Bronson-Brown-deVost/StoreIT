import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Search Navigation", () => {
  test("clicking search result navigates to item", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const uniqueName = `SearchItem_${Date.now()}`;
    const item = await api.createItem(uniqueName, "container", container.id, {
      category: "Gadgets",
    });

    await page.goto("/search");
    await page.getByPlaceholder("Search items, containers, locations...").fill(uniqueName);
    await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
    await page.getByText(uniqueName).click();
    await expect(page).toHaveURL(new RegExp(`/items/${item.id}`));
    // Verify item detail content actually loaded
    await expect(
      page.getByRole("heading", { name: uniqueName })
    ).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Quantity")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Category")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Gadgets")).toBeVisible({ timeout: 5000 });
  });

  test("search finds containers", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const uniqueName = `SearchBox_${Date.now()}`;
    await api.createContainer(uniqueName, "location", loc.id);

    await page.goto("/search");
    await page.getByPlaceholder("Search items, containers, locations...").fill(uniqueName);
    await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
  });

  test("search finds locations", async ({ page, request }) => {
    const api = new TestApi(request);
    const uniqueName = `SearchGarage_${Date.now()}`;
    await api.createLocation(uniqueName);

    await page.goto("/search");
    await page.getByPlaceholder("Search items, containers, locations...").fill(uniqueName);
    await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
  });

  test("search heading is visible", async ({ page }) => {
    await page.goto("/search");
    await expect(page.getByRole("heading", { name: "Search" })).toBeVisible();
    await expect(
      page.getByPlaceholder("Search items, containers, locations...")
    ).toBeVisible();
  });

  test("SPA: search result click loads item detail with full content", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const uniqueName = `DetailItem_${Date.now()}`;
    const item = await api.createItem(uniqueName, "container", container.id, {
      description: "Detailed description for search test",
      category: "Books",
    });

    await page.goto("/search");
    await expect(page.getByRole("heading", { name: "Search" })).toBeVisible();
    await page.getByPlaceholder("Search items, containers, locations...").fill(uniqueName);
    await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });

    // Click the search result
    await page.getByText(uniqueName).click();
    await expect(page).toHaveURL(new RegExp(`/items/${item.id}`));

    // Verify full item detail content
    await expect(
      page.getByRole("heading", { name: uniqueName })
    ).toBeVisible({ timeout: 5000 });
    await expect(
      page.getByText("Detailed description for search test")
    ).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Category")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Books")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Quantity")).toBeVisible({ timeout: 5000 });
  });

  test("SPA: search result click then navigate back to search preserves results", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const uniqueName = `BackSearch_${Date.now()}`;
    await api.createItem(uniqueName, "container", container.id);

    await page.goto("/search");
    await expect(page.getByRole("heading", { name: "Search" })).toBeVisible();
    await page.getByPlaceholder("Search items, containers, locations...").fill(uniqueName);
    await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });

    // Click the result to navigate to item detail
    await page.getByText(uniqueName).click();
    await expect(
      page.getByRole("heading", { name: uniqueName })
    ).toBeVisible({ timeout: 5000 });

    // Go back to search
    await page.goBack();
    await expect(page).toHaveURL(/\/search/);
    await expect(page.getByRole("heading", { name: "Search" })).toBeVisible({
      timeout: 5000,
    });
    await expect(
      page.getByPlaceholder("Search items, containers, locations...")
    ).toBeVisible({ timeout: 5000 });
  });
});
