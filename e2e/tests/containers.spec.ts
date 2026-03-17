import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Containers", () => {
  test("nested containers", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const parent = await api.createContainer("Parent Box", "location", loc.id);
    const child = await api.createContainer("Child Box", "container", parent.id);

    await page.goto(`/containers/${parent.id}`);
    await expect(page.getByRole("heading", { name: parent.name })).toBeVisible();
    await expect(page.getByText(child.name)).toBeVisible();
  });

  test("empty container shows empty state", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("Empty Box", "location", loc.id);

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByRole("heading", { name: container.name })).toBeVisible();
    await expect(page.getByText(/this container is empty/i)).toBeVisible();
  });
});
