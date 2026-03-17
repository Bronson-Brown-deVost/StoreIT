import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Move", () => {
  test("move item to different location", async ({ page, request }) => {
    const api = new TestApi(request);
    const suffix = Date.now();
    const locA = await api.createLocation(`LocA_${suffix}`);
    const locB = await api.createLocation(`LocB_${suffix}`);
    const container = await api.createContainer(undefined, "location", locA.id);
    const item = await api.createItem(undefined, "container", container.id);

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: item.name })).toBeVisible();

    await page.getByRole("button", { name: "Move Item", exact: true }).dispatchEvent("click");

    await expect(page.getByText("Select Parent")).toBeVisible();
    await page.getByRole("button", { name: locB.name }).dispatchEvent("click");

    await expect(page.getByText("Confirm Move")).toBeVisible();
    await page.getByRole("button", { name: "Move", exact: true }).dispatchEvent("click");

    await expect(page.getByText("Moved!")).toBeVisible();
  });

  test("move container to different location", async ({ page, request }) => {
    const api = new TestApi(request);
    const suffix = Date.now();
    const locA = await api.createLocation(`LocMoveA_${suffix}`);
    const locB = await api.createLocation(`LocMoveB_${suffix}`);
    const container = await api.createContainer(undefined, "location", locA.id);

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByRole("heading", { name: container.name })).toBeVisible();

    await page.getByTitle("Move container").dispatchEvent("click");

    await expect(page.getByText("Select Parent")).toBeVisible();
    await page.getByRole("button", { name: locB.name }).dispatchEvent("click");

    await expect(page.getByText("Confirm Move")).toBeVisible();
    await page.getByRole("button", { name: "Move", exact: true }).dispatchEvent("click");

    await expect(page.getByText("Moved!")).toBeVisible();
  });
});
