import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Navigation", () => {
  test("bottom nav Browse link navigates to homepage", async ({ page }) => {
    await page.goto("/search");
    await expect(page.getByRole("heading", { name: "Search" })).toBeVisible();

    await page.getByText("Browse").click();
    await expect(page).toHaveURL("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();
    // Verify home page content loaded (locations section or empty state)
    await expect(
      page.getByText("Locations").or(page.getByText("No locations yet"))
    ).toBeVisible({ timeout: 5000 });
  });

  test("bottom nav Search link navigates to search page", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    await page.getByRole("link", { name: "Search", exact: true }).click();
    await expect(page).toHaveURL(/\/search/);
    await expect(page.getByRole("heading", { name: "Search" })).toBeVisible();
    await expect(
      page.getByPlaceholder("Search items, containers, locations...")
    ).toBeVisible({ timeout: 5000 });
  });

  test("bottom nav Add link navigates to add page", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    // Click the "Add" text in the bottom nav (not the floating button)
    await page.locator("nav").getByText("Add").click();
    await expect(page).toHaveURL(/\/add/);
    await expect(page.getByRole("heading", { name: "Add Item" })).toBeVisible({
      timeout: 5000,
    });
  });

  test("bottom nav Settings link navigates to settings page", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    await page.getByText("Settings").click();
    await expect(page).toHaveURL(/\/settings/);
    await expect(
      page.getByRole("heading", { name: "Settings" })
    ).toBeVisible({ timeout: 5000 });
  });

  test("breadcrumb navigation from container back to location", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("Nav Location");
    const container = await api.createContainer(
      "Nav Container",
      "location",
      loc.id
    );

    await page.goto(`/containers/${container.id}`);
    await expect(
      page.getByRole("heading", { name: container.name })
    ).toBeVisible();

    // Breadcrumbs should show the location name
    await expect(page.getByText(loc.name)).toBeVisible();

    // Click the location name in breadcrumbs to navigate back
    await page.getByRole("link", { name: loc.name }).click();
    await expect(page).toHaveURL(new RegExp(`/locations/${loc.id}`));
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });
    // Verify the location page actually rendered its content
    await expect(page.getByText(container.name)).toBeVisible({ timeout: 5000 });
  });

  test("breadcrumb navigation from nested container shows full path", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("Deep Location");
    const parent = await api.createContainer("Outer Box", "location", loc.id);
    const child = await api.createContainer("Inner Box", "container", parent.id);

    await page.goto(`/containers/${child.id}`);
    await expect(
      page.getByRole("heading", { name: child.name })
    ).toBeVisible();

    // Breadcrumbs should show both the location and parent container
    await expect(page.getByText(loc.name)).toBeVisible();
    await expect(page.getByText(parent.name)).toBeVisible();
  });

  test("breadcrumb home icon navigates to homepage", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("Home Nav Location");
    const container = await api.createContainer(
      "Home Nav Container",
      "location",
      loc.id
    );

    await page.goto(`/containers/${container.id}`);
    await expect(
      page.getByRole("heading", { name: container.name })
    ).toBeVisible();

    // Click the home icon in breadcrumbs (first link in nav)
    await page.locator("nav a[href='/']").first().click();
    await expect(page).toHaveURL("/");
    await expect(
      page.getByRole("heading", { name: "StoreIT" })
    ).toBeVisible({ timeout: 5000 });
    // Verify the home page shows the location we created
    await expect(page.getByText(loc.name)).toBeVisible({ timeout: 5000 });
  });

  test("homepage search bar navigates to search page", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    // Click the search bar on the homepage
    await page.getByText("Search items...").click();
    await expect(page).toHaveURL(/\/search/);
    await expect(page.getByRole("heading", { name: "Search" })).toBeVisible({
      timeout: 5000,
    });
    await expect(
      page.getByPlaceholder("Search items, containers, locations...")
    ).toBeVisible({ timeout: 5000 });
  });

  test("clicking location on homepage navigates to location page", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    await page.goto("/");
    await expect(page.getByText(loc.name)).toBeVisible();
    await page.getByText(loc.name).click();
    await expect(page).toHaveURL(new RegExp(`/locations/${loc.id}`));
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });
  });

  test("SPA: click location on home, then click container, then click item — full drill-down without page.goto", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("DrillLoc");
    const container = await api.createContainer(
      "DrillContainer",
      "location",
      loc.id
    );
    const item = await api.createItem("DrillItem", "container", container.id, {
      description: "A drillable item",
      category: "Tools",
    });

    // Only page.goto at the start
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();
    await expect(page.getByText(loc.name)).toBeVisible({ timeout: 5000 });

    // Click the location
    await page.getByText(loc.name).click();
    await expect(page).toHaveURL(new RegExp(`/locations/${loc.id}`));
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });
    // Verify location page content: container is listed
    await expect(page.getByText(container.name)).toBeVisible({ timeout: 5000 });

    // Click the container
    await page.getByText(container.name).click();
    await expect(page).toHaveURL(new RegExp(`/containers/${container.id}`));
    await expect(
      page.getByRole("heading", { name: container.name })
    ).toBeVisible({ timeout: 5000 });
    // Verify container page content: item is listed
    await expect(page.getByText(item.name)).toBeVisible({ timeout: 5000 });
    // Verify breadcrumbs show the location
    await expect(page.getByText(loc.name)).toBeVisible();

    // Click the item
    await page.getByText(item.name).click();
    await expect(page).toHaveURL(new RegExp(`/items/${item.id}`));
    // Verify item detail content loaded
    await expect(
      page.getByRole("heading", { name: item.name })
    ).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Quantity")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Category")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Tools")).toBeVisible({ timeout: 5000 });
  });

  test("SPA: back button after SPA navigation restores previous page", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("BackLoc");
    const container = await api.createContainer(
      "BackContainer",
      "location",
      loc.id
    );
    await api.createItem("BackItem", "container", container.id);

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    // Navigate: home -> location -> container via clicks
    await page.getByText(loc.name).click();
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });

    await page.getByText(container.name).click();
    await expect(
      page.getByRole("heading", { name: container.name })
    ).toBeVisible({ timeout: 5000 });

    // Press back — should go to location page
    await page.goBack();
    await expect(page).toHaveURL(new RegExp(`/locations/${loc.id}`));
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });
    // Location page should still show the container
    await expect(page.getByText(container.name)).toBeVisible({ timeout: 5000 });
  });

  test("SPA: breadcrumb click after SPA navigation", async ({
    page,
    request,
  }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("BreadLoc");
    const outer = await api.createContainer("OuterBox", "location", loc.id);
    const inner = await api.createContainer("InnerBox", "container", outer.id);

    // Start at home
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    // Navigate via clicks: home -> location -> outer container -> inner container
    await page.getByText(loc.name).click();
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });

    await page.getByText(outer.name).click();
    await expect(
      page.getByRole("heading", { name: outer.name })
    ).toBeVisible({ timeout: 5000 });

    await page.getByText(inner.name).click();
    await expect(
      page.getByRole("heading", { name: inner.name })
    ).toBeVisible({ timeout: 5000 });

    // Now click a breadcrumb to go back to the location
    await page.getByRole("link", { name: loc.name }).click();
    await expect(page).toHaveURL(new RegExp(`/locations/${loc.id}`));
    await expect(
      page.getByRole("heading", { name: loc.name })
    ).toBeVisible({ timeout: 5000 });
    // Verify the location page content loaded
    await expect(page.getByText(outer.name)).toBeVisible({ timeout: 5000 });
  });
});
