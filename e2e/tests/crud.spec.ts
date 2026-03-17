import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("CRUD", () => {
  test("create a location from homepage", async ({ page, request }) => {
    const api = new TestApi(request);
    await api.createLocation();

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    await page.getByTitle("Add location").dispatchEvent("click");

    await expect(page.getByText("New Location")).toBeVisible();

    const locName = `CRUDLoc_${Date.now()}`;
    await page.getByPlaceholder("Enter name").fill(locName);
    await page.getByRole("button", { name: "Create" }).dispatchEvent("click");

    await expect(page.getByText(locName)).toBeVisible({ timeout: 5000 });
  });

  test("create a container from location page", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();

    await page.getByTitle("Add container").dispatchEvent("click");

    await expect(page.getByText("New Container")).toBeVisible();

    const containerName = `CRUDContainer_${Date.now()}`;
    await page.getByPlaceholder("Enter name").fill(containerName);
    await page.getByRole("button", { name: "Create" }).dispatchEvent("click");

    await expect(page.getByText(containerName)).toBeVisible({ timeout: 5000 });
  });

  test("create a sub-location from location page", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();

    await page.getByTitle("Add sub-location").dispatchEvent("click");

    await expect(page.getByText("New Sub-Location")).toBeVisible();

    const subLocName = `CRUDSubLoc_${Date.now()}`;
    await page.getByPlaceholder("Enter name").fill(subLocName);
    await page.getByRole("button", { name: "Create" }).dispatchEvent("click");

    await expect(page.getByText(subLocName)).toBeVisible({ timeout: 5000 });
  });

  test("create a sub-container from container page", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByRole("heading", { name: container.name })).toBeVisible();

    await page.getByTitle("Add sub-container").dispatchEvent("click");

    await expect(page.getByText("New Sub-Container")).toBeVisible();

    const childName = `CRUDChild_${Date.now()}`;
    await page.getByPlaceholder("Enter name").fill(childName);
    await page.getByRole("button", { name: "Create" }).dispatchEvent("click");

    await expect(page.getByText(childName)).toBeVisible({ timeout: 5000 });
  });

  test("delete an item from item detail page", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem(undefined, "container", container.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();

    await page.getByRole("button", { name: "Delete Item" }).click();
    await page.getByRole("button", { name: "Confirm Delete" }).click();

    await expect(page).toHaveURL("/", { timeout: 5000 });
  });

  test("delete item confirmation can be cancelled", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer(undefined, "location", loc.id);
    const item = await api.createItem(undefined, "container", container.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();

    await page.getByRole("button", { name: "Delete Item" }).click();
    await page.getByRole("button", { name: "Cancel" }).click();

    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();
  });
});
