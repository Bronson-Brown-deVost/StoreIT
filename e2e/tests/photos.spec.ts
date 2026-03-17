import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Photos", () => {
  test("uploaded photo appears on item detail page", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem("PhotoItem", "container", container.id);
    await api.uploadPhoto("item", item.id, "test.png");

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: "PhotoItem" })).toBeVisible();
    // Photo gallery section should be present with at least one photo thumbnail
    await expect(page.locator("[role='button'] img").first()).toBeAttached({ timeout: 5000 });
  });

  test("item without photo shows no image", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem("NoPhotoItem", "container", container.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: "NoPhotoItem" })).toBeVisible();
    expect(await page.locator("[role='button'] img").count()).toBe(0);
  });
});
