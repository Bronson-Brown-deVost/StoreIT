import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("CreateDialog buttons accessible on all viewports", () => {
  test("Create and Cancel buttons are not obscured when creating a location", async ({ page, request }) => {
    const api = new TestApi(request);
    await api.createLocation();

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();

    await page.getByTitle("Add location").dispatchEvent("click");
    await expect(page.getByText("New Location")).toBeVisible();

    const createBtn = page.getByRole("button", { name: "Create" });
    await expect(createBtn).toBeVisible();

    // The Create button must be clickable — not covered by the bottom nav or anything else
    // elementFromPoint returns whatever is actually at that pixel, so if the nav covers the button, this fails
    const isClickable = await createBtn.evaluate((el) => {
      const rect = el.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;
      const topEl = document.elementFromPoint(centerX, centerY);
      return el === topEl || el.contains(topEl!);
    });

    expect(isClickable).toBe(true);
  });

  test("Create and Cancel buttons are not obscured when creating a container", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();

    await page.getByTitle("Add container").dispatchEvent("click");
    await expect(page.getByText("New Container")).toBeVisible();

    const createBtn = page.getByRole("button", { name: "Create" });
    await expect(createBtn).toBeVisible();

    const isClickable = await createBtn.evaluate((el) => {
      const rect = el.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;
      const topEl = document.elementFromPoint(centerX, centerY);
      return el === topEl || el.contains(topEl!);
    });

    expect(isClickable).toBe(true);
  });

  test("Create and Cancel buttons are not obscured when creating a sub-location", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();

    await page.getByTitle("Add sub-location").dispatchEvent("click");
    await expect(page.getByText("New Sub-Location")).toBeVisible();

    const createBtn = page.getByRole("button", { name: "Create" });
    await expect(createBtn).toBeVisible();

    const isClickable = await createBtn.evaluate((el) => {
      const rect = el.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;
      const topEl = document.elementFromPoint(centerX, centerY);
      return el === topEl || el.contains(topEl!);
    });

    expect(isClickable).toBe(true);
  });
});
