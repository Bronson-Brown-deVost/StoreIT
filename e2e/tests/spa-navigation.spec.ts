import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

/**
 * SPA Navigation Tests
 *
 * These tests verify that client-side (SPA) navigation works correctly.
 * Each test uses at most ONE page.goto() at the start. All subsequent
 * navigation happens via clicks. Every navigation verifies BOTH the URL
 * change AND that the destination page rendered with real data.
 *
 * This file would have caught the bug where route params weren't passed
 * correctly during SPA navigation, causing blank pages.
 */
test.describe("SPA Navigation", () => {
  test("full drill-down: home → location → container → item via clicks only", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("SpaLoc");
    const container = await api.createContainer(
      "SpaContainer",
      "location",
      loc.id,
      { description: "A container for SPA test" }
    );
    const item = await api.createItem("SpaItem", "container", container.id, {
      description: "SPA test item description",
      category: "Furniture",
    });

    // Single page.goto
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();
    await expect(page.getByText(loc.name)).toBeVisible({ timeout: 5000 });

    // Click location card
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
    await expect(page.getByText(item.name)).toBeVisible({ timeout: 5000 });
    // Verify breadcrumbs show location
    await expect(
      page.getByRole("link", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });

    // Click item card
    await page.getByText(item.name).click();
    await expect(page).toHaveURL(new RegExp(`/items/${item.id}`));
    await expect(
      page.getByRole("heading", { name: item.name })
    ).toBeVisible({ timeout: 5000 });
    // Verify item metadata rendered
    await expect(page.getByText("Quantity")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Category")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Furniture")).toBeVisible({ timeout: 5000 });
    await expect(
      page.getByText("SPA test item description")
    ).toBeVisible({ timeout: 5000 });
  });

  test("entity card click loads location page with content", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("CardClickLoc");
    const container = await api.createContainer(
      "CardClickContainer",
      "location",
      loc.id
    );

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();
    await expect(page.getByText(loc.name)).toBeVisible({ timeout: 5000 });

    // Click the location card
    await page.getByText(loc.name).click();
    await expect(page).toHaveURL(new RegExp(`/locations/${loc.id}`));
    // Verify heading matches location name
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });
    // Verify child content is loaded
    await expect(page.getByText(container.name)).toBeVisible({ timeout: 5000 });
  });

  test("entity card click loads container page with content", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("ContCardLoc");
    const container = await api.createContainer(
      "ContCardContainer",
      "location",
      loc.id,
      { description: "Container card test" }
    );
    const item = await api.createItem(
      "ContCardItem",
      "container",
      container.id
    );

    // Start at the location page via home click
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();
    await page.getByText(loc.name).click();
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });

    // Click container card
    await page.getByText(container.name).click();
    await expect(page).toHaveURL(new RegExp(`/containers/${container.id}`));
    // Verify heading
    await expect(
      page.getByRole("heading", { name: container.name })
    ).toBeVisible({ timeout: 5000 });
    // Verify breadcrumbs present
    await expect(
      page.getByRole("link", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });
    // Verify items are listed
    await expect(page.getByText(item.name)).toBeVisible({ timeout: 5000 });
  });

  test("entity card click loads item detail with content", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("ItemCardLoc");
    const container = await api.createContainer(
      "ItemCardContainer",
      "location",
      loc.id
    );
    const item = await api.createItem(
      "ItemCardItem",
      "container",
      container.id,
      { category: "Clothing" }
    );

    // Navigate to container page via clicks
    await page.goto("/");
    await page.getByText(loc.name).click();
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });
    await page.getByText(container.name).click();
    await expect(
      page.getByRole("heading", { name: container.name })
    ).toBeVisible({ timeout: 5000 });

    // Click item card
    await page.getByText(item.name).click();
    await expect(page).toHaveURL(new RegExp(`/items/${item.id}`));
    // Verify item name in heading
    await expect(
      page.getByRole("heading", { name: item.name })
    ).toBeVisible({ timeout: 5000 });
    // Verify metadata section is visible
    await expect(page.getByText("Quantity")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Category")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Clothing")).toBeVisible({ timeout: 5000 });
  });

  test("multiple navigations without page.goto: location → home → different location", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const locA = await api.createLocation("MultiNavA");
    const locB = await api.createLocation("MultiNavB");
    const containerA = await api.createContainer(
      "ContainerInA",
      "location",
      locA.id
    );
    const containerB = await api.createContainer(
      "ContainerInB",
      "location",
      locB.id
    );

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    // Click location A
    await page.getByText(locA.name).click();
    await expect(page).toHaveURL(new RegExp(`/locations/${locA.id}`));
    await expect(
      page.getByRole("heading", { name: locA.name })
    ).toBeVisible({ timeout: 5000 });
    await expect(page.getByText(containerA.name)).toBeVisible({ timeout: 5000 });

    // Click home via breadcrumb
    await page.locator("nav a[href='/']").first().click();
    await expect(page).toHaveURL("/");
    await expect(
      page.getByRole("heading", { name: "StoreIT" })
    ).toBeVisible({ timeout: 5000 });
    // Both locations should be visible on home
    await expect(page.getByText(locA.name)).toBeVisible({ timeout: 5000 });
    await expect(page.getByText(locB.name)).toBeVisible({ timeout: 5000 });

    // Click location B
    await page.getByText(locB.name).click();
    await expect(page).toHaveURL(new RegExp(`/locations/${locB.id}`));
    await expect(
      page.getByRole("heading", { name: locB.name })
    ).toBeVisible({ timeout: 5000 });
    await expect(page.getByText(containerB.name)).toBeVisible({ timeout: 5000 });
  });

  test("SPA navigation preserves no full reload", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("NoReloadLoc");
    await api.createContainer("NoReloadContainer", "location", loc.id);

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    // Check initial navigation count (should be 1 — the page.goto)
    const initialNavCount = await page.evaluate(
      () => performance.getEntriesByType("navigation").length
    );
    expect(initialNavCount).toBe(1);

    // Navigate via click
    await page.getByText(loc.name).click();
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });

    // Verify no full reload happened — navigation entries should still be 1
    const afterClickNavCount = await page.evaluate(
      () => performance.getEntriesByType("navigation").length
    );
    expect(afterClickNavCount).toBe(1);

    // Navigate again via breadcrumb home
    await page.locator("nav a[href='/']").first().click();
    await expect(
      page.getByRole("heading", { name: "StoreIT" })
    ).toBeVisible({ timeout: 5000 });

    // Still only 1 navigation entry
    const afterSecondClickNavCount = await page.evaluate(
      () => performance.getEntriesByType("navigation").length
    );
    expect(afterSecondClickNavCount).toBe(1);
  });

  test("bottom nav SPA navigation loads page content", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("BottomNavLoc");

    // Start on a location detail page
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();
    await page.getByText(loc.name).click();
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });

    // Click "Search" in bottom nav
    await page
      .locator("nav")
      .getByRole("link", { name: "Search", exact: true })
      .click();
    await expect(page).toHaveURL(/\/search/);
    await expect(
      page.getByRole("heading", { name: "Search" })
    ).toBeVisible({ timeout: 5000 });
    await expect(
      page.getByPlaceholder("Search items, containers, locations...")
    ).toBeVisible({ timeout: 5000 });

    // Click "Add" in bottom nav
    await page.locator("nav").getByText("Add").click();
    await expect(page).toHaveURL(/\/add/);
    await expect(
      page.getByRole("heading", { name: "Add Item" })
    ).toBeVisible({ timeout: 5000 });

    // Click "Browse" in bottom nav to go home
    await page.getByText("Browse").click();
    await expect(page).toHaveURL("/");
    await expect(
      page.getByRole("heading", { name: "StoreIT" })
    ).toBeVisible({ timeout: 5000 });
    // Verify home content loaded
    await expect(page.getByText(loc.name)).toBeVisible({ timeout: 5000 });
  });
});
