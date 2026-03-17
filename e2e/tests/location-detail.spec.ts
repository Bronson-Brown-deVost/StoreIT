import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Location Detail", () => {
  test("location with description shows it", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("DetailLoc");
    await api.updateLocation(loc.id, { description: "A detailed description" });

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByText("A detailed description")).toBeVisible();
  });

  test("location with coordinates shows map link", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("GpsLoc");
    await api.updateLocation(loc.id, { latitude: 47.6062, longitude: -122.3321 });

    await page.goto(`/locations/${loc.id}`);
    const link = page.locator('a[href*="maps.google.com"]');
    await expect(link).toBeVisible({ timeout: 5000 });
    await expect(link).toContainText("47.606200");
  });

  test("sub-locations are listed under parent", async ({ page, request }) => {
    const api = new TestApi(request);
    const parent = await api.createLocation("ParentLoc");
    // Create sub-location via API
    const res = await request.post("/api/v1/locations", {
      data: { name: "ChildLoc", parent_id: parent.id },
    });
    expect(res.ok()).toBeTruthy();

    await page.goto(`/locations/${parent.id}`);
    await expect(page.getByText("Sub-locations")).toBeVisible();
    await expect(page.getByText("ChildLoc")).toBeVisible();
  });

  test("location page shows add container button", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("BtnLoc");

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByTitle("Add container")).toBeVisible();
  });

  test("location NFC tag manager is visible", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("NfcLoc");

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("button", { name: "+ Manual" })).toBeVisible();
  });
});
