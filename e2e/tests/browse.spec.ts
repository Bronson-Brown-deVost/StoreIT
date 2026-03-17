import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Browse", () => {
  test("homepage shows locations", async ({ page, request }) => {
    const api = new TestApi(request);
    const suffix = Date.now();
    const loc1 = await api.createLocation(`Garage_${suffix}`);
    const loc2 = await api.createLocation(`Kitchen_${suffix}`);

    await page.goto("/");
    await expect(page.getByText(loc1.name)).toBeVisible();
    await expect(page.getByText(loc2.name)).toBeVisible();
  });

  test("navigate into location", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);

    await page.goto("/");
    await page.getByText(loc.name).click();
    await expect(page).toHaveURL(new RegExp(`/locations/${loc.id}`));
    // Verify location page heading loaded
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });
    // Verify location page content: container is listed
    await expect(page.getByText(container.name)).toBeVisible({ timeout: 5000 });
  });

  test("navigate into container", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem(undefined, "container", container.id);

    await page.goto(`/containers/${container.id}`);
    await expect(
      page.getByRole("heading", { name: container.name })
    ).toBeVisible({ timeout: 5000 });
    await expect(page.getByText(item.name)).toBeVisible({ timeout: 5000 });
    // Verify breadcrumbs show the location name
    await expect(page.getByText(loc.name)).toBeVisible();
  });

  test("SPA: drill from home into location then into container via clicks", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("BrowseLoc");
    const container = await api.createContainer(
      "BrowseContainer",
      "location",
      loc.id
    );
    const item = await api.createItem("BrowseItem", "container", container.id);

    // Start at home
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();
    await expect(page.getByText(loc.name)).toBeVisible({ timeout: 5000 });

    // Click location
    await page.getByText(loc.name).click();
    await expect(page).toHaveURL(new RegExp(`/locations/${loc.id}`));
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });
    await expect(page.getByText(container.name)).toBeVisible({ timeout: 5000 });

    // Click container card
    await page.getByText(container.name).click();
    await expect(page).toHaveURL(new RegExp(`/containers/${container.id}`));
    await expect(
      page.getByRole("heading", { name: container.name })
    ).toBeVisible({ timeout: 5000 });
    // Verify container shows items
    await expect(page.getByText(item.name)).toBeVisible({ timeout: 5000 });
    // Verify breadcrumbs
    await expect(page.getByText(loc.name)).toBeVisible();
  });

  test("SPA: navigate to item from container listing", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("ItemNavLoc");
    const container = await api.createContainer(
      "ItemNavContainer",
      "location",
      loc.id
    );
    const item = await api.createItem("ItemNavItem", "container", container.id, {
      description: "A navigable item",
      category: "Electronics",
    });

    // Start at home, click through: home -> location -> container -> item
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    await page.getByText(loc.name).click();
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });

    await page.getByText(container.name).click();
    await expect(
      page.getByRole("heading", { name: container.name })
    ).toBeVisible({ timeout: 5000 });

    await page.getByText(item.name).click();
    await expect(page).toHaveURL(new RegExp(`/items/${item.id}`));
    // Verify item detail loaded with actual name
    await expect(
      page.getByRole("heading", { name: item.name })
    ).toBeVisible({ timeout: 5000 });
    // Verify metadata rendered
    await expect(page.getByText("Quantity")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Category")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Electronics")).toBeVisible({ timeout: 5000 });
  });
});
