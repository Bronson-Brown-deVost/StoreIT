import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Empty States", () => {
  test("empty location shows empty state", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("EmptyLoc");

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByText(/this location is empty/i)).toBeVisible();
  });

  test("empty container shows empty state", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("EmptyBox", "location", loc.id);

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByText(/this container is empty/i)).toBeVisible();
  });

  test("search with no results shows message", async ({ page }) => {
    await page.goto("/search");
    await page.getByPlaceholder(/search/i).fill("ZZZNonexistentXYZ99999");
    await expect(page.getByText(/no results/i)).toBeVisible({ timeout: 5000 });
  });
});
