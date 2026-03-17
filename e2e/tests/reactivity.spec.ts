import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Reactivity - UI updates without reload", () => {
  test("creating a container in a location shows it immediately", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();

    // Click add container button
    await page.getByTitle("Add container").click();
    await expect(page.getByText("New Container")).toBeVisible();

    // Fill and submit
    const containerName = `reactivity_container_${Date.now()}`;
    await page.getByPlaceholder("Enter name").fill(containerName);
    await page.getByRole("button", { name: "Create" }).click();

    // Container should appear without reload
    await expect(page.getByText(containerName)).toBeVisible({ timeout: 5000 });
  });

  test("creating a sub-location shows it immediately", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();

    // Click add sub-location button
    await page.getByTitle("Add sub-location").click();
    await expect(page.getByText("New Sub-Location")).toBeVisible();

    const subLocName = `reactivity_subloc_${Date.now()}`;
    await page.getByPlaceholder("Enter name").fill(subLocName);
    await page.getByRole("button", { name: "Create" }).click();

    await expect(page.getByText(subLocName)).toBeVisible({ timeout: 5000 });
  });

  test("creating a location on home page shows it immediately (with existing locations)", async ({ page, request }) => {
    // Ensure at least one location exists so the "+" button appears
    const api = new TestApi(request);
    await api.createLocation();

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();
    await expect(page.getByTitle("Add location")).toBeVisible();

    await page.getByTitle("Add location").dispatchEvent("click");
    await expect(page.getByText("New Location")).toBeVisible();

    const locName = `reactivity_loc_${Date.now()}`;
    await page.getByPlaceholder("Enter name").fill(locName);
    await page.getByRole("button", { name: "Create" }).click();

    await expect(page.getByText(locName)).toBeVisible({ timeout: 5000 });
  });

  test("creating first location on empty home page shows it immediately", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    // When empty, "Add Location" button is in the EmptyState
    const addBtn = page.getByRole("button", { name: "Add Location" });
    await expect(addBtn).toBeVisible();
    await addBtn.click();
    await expect(page.getByText("New Location")).toBeVisible();

    const locName = `reactivity_first_loc_${Date.now()}`;
    await page.getByPlaceholder("Enter name").fill(locName);
    await page.getByRole("button", { name: "Create" }).click();

    // The new location should appear in the list without reload
    await expect(page.getByText(locName)).toBeVisible({ timeout: 5000 });
  });

  test("creating a sub-container shows it immediately", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByRole("heading", { name: container.name })).toBeVisible();

    await page.getByTitle("Add sub-container").click();
    await expect(page.getByText("New Sub-Container")).toBeVisible();

    const subName = `reactivity_subcontainer_${Date.now()}`;
    await page.getByPlaceholder("Enter name").fill(subName);
    await page.getByRole("button", { name: "Create" }).click();

    await expect(page.getByText(subName)).toBeVisible({ timeout: 5000 });
  });

  test("editing an item updates the detail page immediately", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const item = await api.createItem("Original Name", "location", loc.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByText("Original Name")).toBeVisible();

    // Click edit
    await page.getByTitle("Edit item").click();
    await expect(page.getByText("Edit Item")).toBeVisible();

    // Change name
    const nameInput = page.locator("form input[type='text']").first();
    await nameInput.fill("Updated Name");
    await page.getByRole("button", { name: "Save" }).click();

    // Should show updated name without reload
    await expect(page.getByText("Updated Name")).toBeVisible({ timeout: 5000 });
  });

  test("moving an item updates the location path", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc1 = await api.createLocation("Source Location");
    const loc2 = await api.createLocation("Target Location");
    const item = await api.createItem("Movable Item", "location", loc1.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByText("Source Location")).toBeVisible();

    // Click move button in header
    await page.getByTitle("Move item").click();

    // ParentPicker shows — click target location
    await page.getByText("Target Location").click();

    // Confirm dialog — click Move button (exact match to avoid "Move Item")
    await page.getByRole("button", { name: "Move", exact: true }).click();

    // Wait for "Moved!" success then auto-close
    await expect(page.getByText("Moved!")).toBeVisible({ timeout: 5000 });

    // Location path should update after move completes
    await expect(page.getByText("Target Location")).toBeVisible({ timeout: 5000 });
  });

  test("deleting an item navigates away", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const item = await api.createItem("Deletable Item", "location", loc.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByText("Deletable Item")).toBeVisible();

    // Click delete
    await page.getByRole("button", { name: "Delete Item" }).click();
    await page.getByRole("button", { name: "Confirm Delete" }).click();

    // Should navigate away from item detail
    await expect(page).not.toHaveURL(`/items/${item.id}`, { timeout: 5000 });
  });

  test("uploading a photo shows it in the gallery immediately", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();

    // Initially no photos
    const photosSection = page.locator("img");
    const initialCount = await photosSection.count();

    // Upload a photo via file input
    const fileInput = page.locator("input[type='file'][accept='image/*']");
    await fileInput.setInputFiles({
      name: "test.png",
      mimeType: "image/png",
      buffer: Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==", "base64"),
    });

    // Photo should appear
    await expect(page.locator("img")).toHaveCount(initialCount + 1, { timeout: 10000 });
  });
});
