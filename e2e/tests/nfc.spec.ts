import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("NFC", () => {
  test("register and assign NFC tag", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("NFC Box", "location", loc.id);
    const tagUri = `nfc://e2e-tag-${Date.now()}`;

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByRole("heading", { name: container.name })).toBeVisible();

    // Click "+ Manual" in NFC section
    await page.getByRole("button", { name: "+ Manual" }).click();

    // Enter tag URI in the input
    await page.getByPlaceholder(/tag uid or uri/i).fill(tagUri);

    // Click the "Add" button
    await page.getByRole("button", { name: "Add", exact: true }).click();

    // Verify tag appears in the list
    await expect(page.getByText(tagUri)).toBeVisible();
  });

  test("unassign NFC tag", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("NFC Unassign Box", "location", loc.id);
    const tagUri = `nfc://e2e-unassign-${Date.now()}`;
    const tag = await api.createNfcTag(tagUri);
    await api.assignNfcTag(tag.id, "container", container.id);

    await page.goto(`/containers/${container.id}`);
    await expect(page.getByText(tagUri)).toBeVisible();

    // Click Unassign
    await page.getByRole("button", { name: "Unassign" }).click();

    // Tag should no longer be listed
    await expect(page.getByText(tagUri)).not.toBeVisible();
  });

  test("resolve NFC tag navigates to entity", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const container = await api.createContainer("NFC Resolve Box", "location", loc.id);
    const tagUri = `nfc://e2e-resolve-${Date.now()}`;
    const tag = await api.createNfcTag(tagUri);
    await api.assignNfcTag(tag.id, "container", container.id);

    // Navigate to the NFC resolve URL
    const encodedUri = encodeURIComponent(tagUri);
    await page.goto(`/nfc/${encodedUri}`);

    // Should redirect to the container page
    await expect(page).toHaveURL(new RegExp(`/containers/${container.id}`), { timeout: 5000 });
    await expect(page.getByRole("heading", { name: container.name })).toBeVisible();
  });

  test("UID resolve: unknown tag shows assign UI", async ({ page }) => {
    const uid = `04${Date.now().toString(16).toUpperCase().slice(0, 12)}`;
    await page.goto(`/nfc/tag?uid=${uid}`);

    await expect(page.getByText("New NFC Tag")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText(uid)).toBeVisible();
    await expect(page.getByText("Link to Location or Container")).toBeVisible();
  });

  test("UID resolve: assigned tag navigates to entity", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const uid = `04AA${Date.now().toString(16).toUpperCase().slice(0, 10)}`;

    // Register and assign via API
    await request.post("/api/v1/nfc-tags/register-and-assign", {
      data: {
        tag_uri: uid,
        entity_type: "location",
        entity_id: loc.id,
      },
    });

    await page.goto(`/nfc/tag?uid=${uid}`);
    await expect(page).toHaveURL(new RegExp(`/locations/${loc.id}`), { timeout: 5000 });
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();
  });

  test("UID resolve: assign unknown tag to location", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();
    const uid = `04BB${Date.now().toString(16).toUpperCase().slice(0, 10)}`;

    await page.goto(`/nfc/tag?uid=${uid}`);
    await expect(page.getByText("New NFC Tag")).toBeVisible({ timeout: 5000 });

    // Click "Link to Location or Container"
    await page.getByText("Link to Location or Container").click();

    // ParentPicker modal should appear
    await expect(page.getByText("Select Parent")).toBeVisible();

    // Click the location
    await page.getByText(loc.name).click();

    // Should navigate to the location
    await expect(page).toHaveURL(new RegExp(`/locations/${loc.id}`), { timeout: 10000 });
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();
  });
});
