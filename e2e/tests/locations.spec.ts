import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Locations", () => {
  test("location detail page shows containers and items", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("Detail Location");
    const container = await api.createContainer("Shelf A", "location", loc.id);
    const item = await api.createItem("Loose Wrench", "location", loc.id);

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();
    await expect(page.getByText(container.name)).toBeVisible();
    await expect(page.getByText(item.name)).toBeVisible();
  });

  test("location page shows breadcrumbs with home link", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("Breadcrumb Location");

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();
    // Breadcrumbs should contain a home link
    const breadcrumbs = page.locator("nav").first();
    await expect(breadcrumbs.locator("a[href='/']")).toBeVisible();
  });

  test("location created via API is visible on detail page", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("API Created Location");
    const containerA = await api.createContainer("Box Alpha", "location", loc.id);
    const containerB = await api.createContainer("Box Beta", "location", loc.id);

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();
    await expect(page.getByText(containerA.name)).toBeVisible();
    await expect(page.getByText(containerB.name)).toBeVisible();
  });

  test("location with description displays description", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("Described Location");
    // Create a container with description to verify the location page loads
    await api.createContainer("Bin", "location", loc.id, { description: "A storage bin" });

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();
    await expect(page.getByText("A storage bin")).toBeVisible();
  });

  test("empty location shows empty state", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("Empty Location");

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();
    await expect(page.getByText(/this location is empty/i)).toBeVisible();
  });
});
