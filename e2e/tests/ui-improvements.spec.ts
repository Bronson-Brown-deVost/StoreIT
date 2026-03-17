import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Breadcrumb deduplication", () => {
  test("container name appears only once in breadcrumbs", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByRole("heading", { name: container.name })).toBeVisible();

    // The container name should appear exactly once in the breadcrumb nav
    // (as the current page label, not also as a link)
    const breadcrumbNav = page.locator("nav").first();
    const containerNameCount = await breadcrumbNav.getByText(container.name).count();
    expect(containerNameCount).toBe(1);
  });

  test("nested container breadcrumbs have no duplicates", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const parent = await api.createContainer("ParentBox", "location", loc.id);
    const child = await api.createContainer("ChildBox", "container", parent.id);

    await page.goto(`/containers/${child.id}`);
    await expect(page.getByRole("heading", { name: "ChildBox" })).toBeVisible();

    const breadcrumbNav = page.locator("nav").first();
    // Parent should appear once as a link
    const parentCount = await breadcrumbNav.getByText("ParentBox").count();
    expect(parentCount).toBe(1);
    // Child should appear once as the current page label
    const childCount = await breadcrumbNav.getByText("ChildBox").count();
    expect(childCount).toBe(1);
  });
});

test.describe("Item photo in listings", () => {
  test("item with photo shows thumbnail in container listing", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem(undefined, "container", container.id);
    await api.uploadPhoto("item", item.id, "test.png");

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByText(item.name)).toBeVisible();

    // The item card should show an img tag (photo thumbnail) instead of the default SVG icon
    const itemCard = page.locator(`a[href="/items/${item.id}"]`);
    await expect(itemCard.locator("img")).toBeAttached();
  });
});

test.describe("Item detail page shows location path", () => {
  test("item in container shows full location path", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("TestRoom");
    const container = await api.createContainer("TestShelf", "location", loc.id);
    const item = await api.createItem(undefined, "container", container.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();

    // Should show the location path: TestRoom > TestShelf
    await expect(page.getByRole("link", { name: "TestRoom" })).toBeVisible();
    await expect(page.getByRole("link", { name: "TestShelf" })).toBeVisible();
  });

  test("item in location shows location name", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("DirectRoom");
    const item = await api.createItem(undefined, "location", loc.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();
    await expect(page.getByRole("link", { name: "DirectRoom" })).toBeVisible();
  });
});

test.describe("Edit item", () => {
  test("edit button opens edit form and saves changes", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem("OriginalName", "container", container.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: "OriginalName" })).toBeVisible();

    // Click the Edit Item button
    await page.getByTitle("Edit item").dispatchEvent("click");
    await expect(page.getByText("Edit Item").first()).toBeVisible();

    // Change the name
    const nameInput = page.locator("form input[required]");
    await nameInput.fill("UpdatedName");

    // Save
    await page.getByRole("button", { name: "Save" }).dispatchEvent("click");

    // Verify the name was updated
    await expect(page.getByRole("heading", { name: "UpdatedName" })).toBeVisible({ timeout: 5000 });
  });

  test("edit form cancel discards changes", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem("KeepName", "container", container.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: "KeepName" })).toBeVisible();

    await page.getByTitle("Edit item").dispatchEvent("click");
    const nameInput = page.locator("form input[required]");
    await nameInput.fill("ShouldNotSave");

    await page.getByRole("button", { name: "Cancel" }).dispatchEvent("click");
    await expect(page.getByRole("heading", { name: "KeepName" })).toBeVisible();
  });
});

test.describe("Photo gallery on item detail", () => {
  test("item detail page shows Add Photo button", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem(undefined, "container", container.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();
    await expect(page.getByText("Add Photo")).toBeVisible();
  });

  test("item with photo shows photo on detail page", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem(undefined, "container", container.id);
    await api.uploadPhoto("item", item.id, "test.png");

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();
    // Should have an img element for the photo
    await expect(page.locator("img").first()).toBeAttached();
  });
});

test.describe("Photo upload at creation time", () => {
  test("create dialog shows photo upload option", async ({ page, request }) => {
    const api = new TestApi(request);
    await api.createLocation();

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    await page.getByTitle("Add location").dispatchEvent("click");
    await expect(page.getByText("New Location")).toBeVisible();
    await expect(page.getByText("Photo (optional)")).toBeVisible();
    await expect(page.getByText("Add Photo")).toBeVisible();
  });
});
