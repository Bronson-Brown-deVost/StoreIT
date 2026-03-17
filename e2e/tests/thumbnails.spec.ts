import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Entity thumbnails in listings", () => {
  test("location with photo shows thumbnail on homepage", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    await api.uploadPhoto("location", loc.id, "test.png");

    await page.goto("/");
    await expect(page.getByText(loc.name)).toBeVisible();

    // The location card should have an img element (photo thumbnail)
    const card = page.locator(`a[href="/locations/${loc.id}"]`);
    await expect(card.locator("img")).toBeAttached({ timeout: 5000 });
  });

  test("container with photo shows thumbnail on location page", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    await api.uploadPhoto("container", container.id, "test.png");

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByText(container.name)).toBeVisible();

    const card = page.locator(`a[href="/containers/${container.id}"]`);
    await expect(card.locator("img")).toBeAttached({ timeout: 5000 });
  });

  test("item with photo shows thumbnail on container page", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem(undefined, "container", container.id);
    await api.uploadPhoto("item", item.id, "test.png");

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByText(item.name)).toBeVisible();

    const card = page.locator(`a[href="/items/${item.id}"]`);
    await expect(card.locator("img")).toBeAttached({ timeout: 5000 });
  });

  test("sub-location with photo shows thumbnail on location page", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const child = await api.createLocation();
    // Need to create it as a child - use API directly
    const res = await request.post("/api/v1/locations", {
      data: { name: `SubLoc_${Date.now()}`, parent_id: loc.id },
    });
    const subLoc = await res.json();
    await api.uploadPhoto("location", subLoc.id, "test.png");

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByText(subLoc.name)).toBeVisible();

    const card = page.locator(`a[href="/locations/${subLoc.id}"]`);
    await expect(card.locator("img")).toBeAttached({ timeout: 5000 });
  });

  test("child container with photo shows thumbnail on container page", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const parent = await api.createContainer(undefined, "location", loc.id);
    const child = await api.createContainer(undefined, "container", parent.id);
    await api.uploadPhoto("container", child.id, "test.png");

    await page.goto(`/containers/${parent.id}`);
    await expect(page.getByText(child.name)).toBeVisible();

    const card = page.locator(`a[href="/containers/${child.id}"]`);
    await expect(card.locator("img")).toBeAttached({ timeout: 5000 });
  });

  test("entity without photo shows placeholder icon, not img", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByText(container.name)).toBeVisible();

    // Should have SVG placeholder, not an img
    const card = page.locator(`a[href="/containers/${container.id}"]`);
    await expect(card.locator("svg").first()).toBeVisible();
    expect(await card.locator("img").count()).toBe(0);
  });
});

test.describe("Full-size photo viewing", () => {
  test("clicking photo thumbnail on item detail opens full-size view", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem(undefined, "container", container.id);
    await api.uploadPhoto("item", item.id, "test.png");

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();

    // Click the thumbnail container (div with role=button wrapping the img)
    const thumbnailBtn = page.locator("[role='button']").first();
    await expect(thumbnailBtn).toBeVisible({ timeout: 5000 });
    await thumbnailBtn.dispatchEvent("click");

    // Should see a full-size overlay/modal with the image
    await expect(page.locator("[data-testid='photo-lightbox']")).toBeVisible({ timeout: 3000 });
    await expect(page.locator("[data-testid='photo-lightbox'] img")).toBeAttached();
  });

  test("clicking photo on location page opens full-size view", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    await api.uploadPhoto("location", loc.id, "test.png");

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();

    // Click the thumbnail container
    const thumbnailBtn = page.locator("[role='button']").first();
    await expect(thumbnailBtn).toBeVisible({ timeout: 5000 });
    await thumbnailBtn.dispatchEvent("click");

    await expect(page.locator("[data-testid='photo-lightbox']")).toBeVisible({ timeout: 3000 });
    await expect(page.locator("[data-testid='photo-lightbox'] img")).toBeAttached();
  });

  test("lightbox can be closed", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem(undefined, "container", container.id);
    await api.uploadPhoto("item", item.id, "test.png");

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();

    const thumbnailBtn = page.locator("[role='button']").first();
    await expect(thumbnailBtn).toBeVisible({ timeout: 5000 });
    await thumbnailBtn.dispatchEvent("click");

    const lightbox = page.locator("[data-testid='photo-lightbox']");
    await expect(lightbox).toBeVisible({ timeout: 3000 });

    // Close the lightbox by clicking the close button
    await page.getByLabel("Close").dispatchEvent("click");
    await expect(lightbox).not.toBeVisible();
  });
});
