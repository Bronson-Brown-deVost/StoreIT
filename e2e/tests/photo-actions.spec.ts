import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Photo Actions", () => {
  test("rotate photo changes the image", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("RotateLoc");
    const container = await api.createContainer("RotateBox", "location", loc.id);
    const item = await api.createItem("RotateItem", "container", container.id);
    await api.uploadPhoto("item", item.id, "test.png");

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: "RotateItem" })).toBeVisible({
      timeout: 5000,
    });

    // Wait for thumbnail to appear
    const thumbnail = page.locator("img").first();
    await expect(thumbnail).toBeVisible({ timeout: 5000 });

    // Click thumbnail to open lightbox
    await thumbnail.click();
    await expect(page.getByTestId("photo-lightbox")).toBeVisible();

    // Verify rotate buttons exist
    await expect(
      page.getByRole("button", { name: "Rotate right" })
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Rotate left" })
    ).toBeVisible();

    // Get the image src before rotation
    const lightboxImg = page.getByTestId("photo-lightbox").locator("img");
    await expect(lightboxImg).toBeVisible({ timeout: 5000 });
    const srcBefore = await lightboxImg.getAttribute("src");

    // Click rotate right
    await page.getByRole("button", { name: "Rotate right" }).click();

    // Wait for rotation to complete — "Rotating..." text should appear and disappear
    await expect(page.getByText("Rotating...")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Rotating...")).not.toBeVisible({
      timeout: 10000,
    });

    // Image src should have changed (cache bust query param)
    const srcAfter = await lightboxImg.getAttribute("src");
    expect(srcAfter).not.toBe(srcBefore);
    expect(srcAfter).toContain("?v=");
  });

  test("rotate left also works", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("RotateLeftLoc");
    const item = await api.createItem("RotateLeftItem", "location", loc.id);
    await api.uploadPhoto("item", item.id, "test.png");

    await page.goto(`/items/${item.id}`);
    const thumbnail = page.locator("img").first();
    await expect(thumbnail).toBeVisible({ timeout: 5000 });
    await thumbnail.click();
    await expect(page.getByTestId("photo-lightbox")).toBeVisible();

    await page.getByRole("button", { name: "Rotate left" }).click();
    await expect(page.getByText("Rotating...")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Rotating...")).not.toBeVisible({
      timeout: 10000,
    });

    const src = await page
      .getByTestId("photo-lightbox")
      .locator("img")
      .getAttribute("src");
    expect(src).toContain("?v=");
  });

  test("delete photo removes it from gallery", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("DeletePhotoLoc");
    const item = await api.createItem("DeletePhotoItem", "location", loc.id);
    await api.uploadPhoto("item", item.id, "test.png");

    await page.goto(`/items/${item.id}`);
    await expect(page.getByRole("heading", { name: "DeletePhotoItem" })).toBeVisible({
      timeout: 5000,
    });

    // Count images before delete
    const thumbnail = page.locator("img").first();
    await expect(thumbnail).toBeVisible({ timeout: 5000 });
    const countBefore = await page.locator("img").count();

    // Open lightbox
    await thumbnail.click();
    await expect(page.getByTestId("photo-lightbox")).toBeVisible();

    // Click delete
    await page.getByRole("button", { name: "Delete photo" }).click();

    // Confirm
    await expect(page.getByText("Confirm Delete")).toBeVisible();
    await page.getByText("Confirm Delete").click();

    // Lightbox should close
    await expect(page.getByTestId("photo-lightbox")).not.toBeVisible({
      timeout: 5000,
    });

    // Photo should be gone from gallery
    const countAfter = await page.locator("img").count();
    expect(countAfter).toBeLessThan(countBefore);
  });

  test("delete cancel does not remove photo", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation("CancelDeleteLoc");
    const item = await api.createItem("CancelDeleteItem", "location", loc.id);
    await api.uploadPhoto("item", item.id, "test.png");

    await page.goto(`/items/${item.id}`);
    const thumbnail = page.locator("img").first();
    await expect(thumbnail).toBeVisible({ timeout: 5000 });

    // Open lightbox
    await thumbnail.click();
    await expect(page.getByTestId("photo-lightbox")).toBeVisible();

    // Click delete then cancel
    await page.getByRole("button", { name: "Delete photo" }).click();
    await expect(page.getByText("Confirm Delete")).toBeVisible();
    await page.getByTestId("photo-lightbox").getByText("Cancel").click();

    // Confirm Delete should be gone, lightbox still open
    await expect(page.getByText("Confirm Delete")).not.toBeVisible();
    await expect(page.getByTestId("photo-lightbox")).toBeVisible();

    // Close lightbox
    await page.getByRole("button", { name: "Close" }).click();
    await expect(page.getByTestId("photo-lightbox")).not.toBeVisible();

    // Photo should still be visible
    await expect(page.locator("img").first()).toBeVisible();
  });
});
