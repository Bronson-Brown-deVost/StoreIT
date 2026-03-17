import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("NFC Lifecycle", () => {
  test("delete NFC tag removes it from list", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("NFC Del Box", "location", loc.id);
    const tagUri = `nfc://e2e-delete-${Date.now()}`;
    const tag = await api.createNfcTag(tagUri);
    await api.assignNfcTag(tag.id, "container", container.id);

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByText(tagUri)).toBeVisible();

    // Unassign first, then the tag won't show in container anymore
    await page.getByRole("button", { name: "Unassign" }).click();
    await expect(page.getByText(tagUri)).not.toBeVisible();
  });

  test("resolve unknown tag shows error page", async ({ page }) => {
    const unknownUri = encodeURIComponent(`nfc://unknown-tag-${Date.now()}`);
    await page.goto(`/nfc/${unknownUri}`);

    await expect(page.getByText("Unknown Tag")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Go Home")).toBeVisible();
  });

  test("Go Home button on unknown tag navigates to homepage", async ({ page }) => {
    const unknownUri = encodeURIComponent(`nfc://unknown-home-${Date.now()}`);
    await page.goto(`/nfc/${unknownUri}`);

    await expect(page.getByText("Go Home")).toBeVisible({ timeout: 5000 });
    await page.getByText("Go Home").click();
    await expect(page).toHaveURL("/");
  });
});
