import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Container Detail", () => {
  test("container with color shows color indicator", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("ColorBox", "location", loc.id, { color: "#ff5733" });

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByRole("heading", { name: "ColorBox" })).toBeVisible();
    // Color circle should be rendered
    const colorDot = page.locator('div[style*="background-color"]');
    await expect(colorDot).toBeVisible();
  });

  test("container with description shows it", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("DescBox", "location", loc.id, { description: "Box for tools" });

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByText("Box for tools")).toBeVisible();
  });

  test("nested breadcrumb shows full path", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("CrumbLoc");
    const parent = await api.createContainer("ParentBox", "location", loc.id);
    const child = await api.createContainer("ChildBox", "container", parent.id);

    await page.goto(`/containers/${child.id}`);
    await expect(page.getByRole("heading", { name: "ChildBox" })).toBeVisible();
    // Breadcrumbs should show parent container and location
    await expect(page.getByText("CrumbLoc")).toBeVisible();
    await expect(page.getByText("ParentBox")).toBeVisible();
  });

  test("print label button exists", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("PrintBox", "location", loc.id);

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByTitle("Print label")).toBeVisible();
  });

  test("add sub-container button exists", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("SubBox", "location", loc.id);

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByTitle("Add sub-container")).toBeVisible();
  });
});
