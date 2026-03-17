import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Items", () => {
  test("item detail page shows metadata", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("Metadata Box", "location", loc.id);
    const item = await api.createItem("Detailed Widget", "container", container.id, {
      description: "A very detailed widget",
      category: "Tools",
    });

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();
    await expect(page.getByText("A very detailed widget")).toBeVisible();
    await expect(page.getByText("Tools")).toBeVisible();
  });

  test("item detail page shows item name in heading", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem("Unique Gadget", "container", container.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();
  });

  test("item detail page shows move and delete buttons", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem("Actionable Item", "container", container.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();
    await expect(page.getByRole("button", { name: "Move Item", exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Delete Item" })).toBeVisible();
  });

  test("item detail page has back button", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("Nav Box", "location", loc.id);
    const item = await api.createItem("Back Button Item", "container", container.id);

    // Navigate to container first, then to item
    await page.goto(`/containers/${container.id}`);
    await expect(page.getByText(item.name)).toBeVisible();
    await page.getByText(item.name).click();

    await expect(page).toHaveURL(new RegExp(`/items/${item.id}`));
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();

    // Click back button (the chevron-left SVG button)
    await page.locator("button").filter({ has: page.locator("svg polyline") }).first().click();
    await expect(page).toHaveURL(new RegExp(`/containers/${container.id}`));
  });

  test("navigate to item from container page", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("Browse Box", "location", loc.id);
    const item = await api.createItem("Clickable Item", "container", container.id);

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByText(item.name)).toBeVisible();
    await page.getByText(item.name).click();

    await expect(page).toHaveURL(new RegExp(`/items/${item.id}`));
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();
  });
});
