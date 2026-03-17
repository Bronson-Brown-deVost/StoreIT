import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Entity Metadata Display", () => {
  test("item shows all metadata fields", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);

    // Create item with all metadata via API
    const res = await request.post("/api/v1/items", {
      data: {
        parent_type: "container",
        parent_id: container.id,
        name: "MetaItem",
        description: "A detailed item",
        category: "Electronics",
        material: "Plastic",
        color: "Blue",
        condition_notes: "Like new",
        quantity: 3,
        aliases: ["gadget", "widget"],
        keywords: ["tech", "device"],
      },
    });
    const item = await res.json();

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: "MetaItem" })).toBeVisible();
    await expect(page.getByText("A detailed item")).toBeVisible();
    await expect(page.getByText("Electronics")).toBeVisible();
    await expect(page.getByText("Plastic")).toBeVisible();
    await expect(page.getByText("Blue")).toBeVisible();
    await expect(page.getByText("Like new")).toBeVisible();
    await expect(page.getByText("3", { exact: true })).toBeVisible();
    await expect(page.getByText("gadget, widget")).toBeVisible();
    await expect(page.getByText("tech, device")).toBeVisible();
  });

  test("container with all fields displays correctly", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("MetaLoc");
    const container = await api.createContainer("MetaBox", "location", loc.id, {
      description: "A storage box",
      color: "#3498db",
    });

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByRole("heading", { name: "MetaBox" })).toBeVisible();
    await expect(page.getByText("A storage box")).toBeVisible();
  });

  test("location with description and coordinates displays both", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("FullLoc");
    await api.updateLocation(loc.id, {
      description: "Main storage area",
      latitude: 40.7128,
      longitude: -74.006,
    });

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: "FullLoc" })).toBeVisible();
    await expect(page.getByText("Main storage area")).toBeVisible();
    await expect(page.locator('a[href*="maps.google.com"]')).toBeVisible();
  });
});
