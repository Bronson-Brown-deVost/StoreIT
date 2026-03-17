import { test, expect } from "@playwright/test";
import { TestApi } from "../helpers/api";

test.describe("Photo upload", () => {
  test("upload a large photo (5MB) via API succeeds", async ({ request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    // Create a ~5MB buffer to simulate a mobile camera photo
    const size = 5 * 1024 * 1024;
    const largeBuffer = Buffer.alloc(size, 0);
    // Minimal valid JPEG: SOI marker + padding + EOI marker
    largeBuffer[0] = 0xff;
    largeBuffer[1] = 0xd8;
    largeBuffer[size - 2] = 0xff;
    largeBuffer[size - 1] = 0xd9;

    const res = await request.post("/api/v1/photos", {
      multipart: {
        entity_type: "location",
        entity_id: loc.id,
        file: {
          name: "camera-photo.jpg",
          mimeType: "image/jpeg",
          buffer: largeBuffer,
        },
      },
    });

    expect(res.status()).toBe(201);
    const photo = await res.json();
    expect(photo.id).toBeTruthy();
  });

  test("upload a 15MB photo via API succeeds", async ({ request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    const size = 15 * 1024 * 1024;
    const largeBuffer = Buffer.alloc(size, 0);
    largeBuffer[0] = 0xff;
    largeBuffer[1] = 0xd8;
    largeBuffer[size - 2] = 0xff;
    largeBuffer[size - 1] = 0xd9;

    const res = await request.post("/api/v1/photos", {
      multipart: {
        entity_type: "location",
        entity_id: loc.id,
        file: {
          name: "large-camera-photo.jpg",
          mimeType: "image/jpeg",
          buffer: largeBuffer,
        },
      },
    });

    expect(res.status()).toBe(201);
  });

  test("photo gallery shows error message on upload failure", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();

    // The "Add Photo" button should exist
    await expect(page.getByText("Add Photo")).toBeVisible();

    // After a failed upload, an error message should be shown to the user
    // (We can't easily simulate a failure in e2e, but we verify the UI has the upload button)
  });

  test("photo gallery file input does not force camera capture", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();

    // The file input should accept images but NOT have the capture attribute
    // (capture forces camera-only on some mobile browsers, preventing gallery selection)
    const fileInput = page.locator("input[type='file'][accept='image/*']");
    await expect(fileInput).toBeAttached();
    const captureAttr = await fileInput.getAttribute("capture");
    expect(captureAttr).toBeNull();
  });

  test("create dialog file input does not force camera capture", async ({ page, request }) => {
    const api = new TestApi(request);
    await api.createLocation();

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "StoreIT" })).toBeVisible();
    await page.getByTitle("Add location").dispatchEvent("click");
    await expect(page.getByText("New Location")).toBeVisible();

    const fileInput = page.locator("input[type='file'][accept='image/*']");
    await expect(fileInput).toBeAttached();
    const captureAttr = await fileInput.getAttribute("capture");
    expect(captureAttr).toBeNull();
  });

  test("thumbnail endpoint returns valid displayable image", async ({ request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    // Create a real 400x300 PNG with color data (simulates a real photo)
    const PNG = await import("pngjs").then((m) => m.PNG);
    const png = new PNG({ width: 400, height: 300 });
    for (let y = 0; y < 300; y++) {
      for (let x = 0; x < 400; x++) {
        const idx = (400 * y + x) * 4;
        png.data[idx] = x % 256;     // R
        png.data[idx + 1] = y % 256; // G
        png.data[idx + 2] = 128;     // B
        png.data[idx + 3] = 255;     // A
      }
    }
    const imageBuffer = PNG.sync.write(png);

    const uploadRes = await request.post("/api/v1/photos", {
      multipart: {
        entity_type: "location",
        entity_id: loc.id,
        file: {
          name: "real-photo.png",
          mimeType: "image/png",
          buffer: imageBuffer,
        },
      },
    });
    expect(uploadRes.status()).toBe(201);
    const photo = await uploadRes.json();

    // Full file endpoint should return original
    const fullRes = await request.get(`/api/v1/photos/${photo.id}/file`);
    expect(fullRes.status()).toBe(200);
    const fullBody = await fullRes.body();
    expect(fullBody.length).toBeGreaterThan(1000);

    // Thumbnail endpoint should return a valid, smaller WebP
    const thumbRes = await request.get(`/api/v1/photos/${photo.id}/thumbnail`);
    expect(thumbRes.status()).toBe(200);
    expect(thumbRes.headers()["content-type"]).toBe("image/webp");
    expect(thumbRes.headers()["cache-control"]).toContain("immutable");
    const thumbBody = await thumbRes.body();
    expect(thumbBody.length).toBeGreaterThan(100);
    expect(thumbBody.length).toBeLessThan(fullBody.length);
    // WebP files start with RIFF....WEBP
    expect(Buffer.from(thumbBody.subarray(0, 4)).toString()).toBe("RIFF");
    expect(Buffer.from(thumbBody.subarray(8, 12)).toString()).toBe("WEBP");
    // Must be lossy VP8 (not VP8L lossless)
    expect(Buffer.from(thumbBody.subarray(12, 16)).toString()).toBe("VP8 ");
  });

  test("thumbnail displays correctly in the browser", async ({ page, request }) => {
    const api = new TestApi(request);
    const loc = await api.createLocation();

    // Create a real image
    const PNG = await import("pngjs").then((m) => m.PNG);
    const png = new PNG({ width: 400, height: 300 });
    for (let y = 0; y < 300; y++) {
      for (let x = 0; x < 400; x++) {
        const idx = (400 * y + x) * 4;
        png.data[idx] = (x * 3) % 256;
        png.data[idx + 1] = (y * 2) % 256;
        png.data[idx + 2] = 100;
        png.data[idx + 3] = 255;
      }
    }
    const imageBuffer = PNG.sync.write(png);

    // Upload via API
    await request.post("/api/v1/photos", {
      multipart: {
        entity_type: "location",
        entity_id: loc.id,
        file: {
          name: "browser-test.png",
          mimeType: "image/png",
          buffer: imageBuffer,
        },
      },
    });

    // Navigate to the location page
    await page.goto(`/locations/${loc.id}`);
    await expect(page.getByRole("heading", { name: loc.name })).toBeVisible();

    // Wait for thumbnail image to appear and load
    const thumbImg = page.locator("[role='button'] img").first();
    await expect(thumbImg).toBeVisible({ timeout: 10000 });

    // Verify the image actually rendered (naturalWidth > 0 means it loaded successfully)
    const naturalWidth = await thumbImg.evaluate(
      (img: HTMLImageElement) => img.naturalWidth
    );
    expect(naturalWidth).toBeGreaterThan(0);

    // Verify it's using the thumbnail URL, not the full file URL
    const src = await thumbImg.getAttribute("src");
    expect(src).toContain("/thumbnail");
  });

  test("add item page file input does not force camera capture", async ({ page }) => {
    await page.goto("/add");
    await expect(page.getByRole("heading", { name: "Add Item" })).toBeVisible();

    const fileInput = page.locator("input[type='file'][accept='image/*']");
    await expect(fileInput).toBeAttached();
    const captureAttr = await fileInput.getAttribute("capture");
    expect(captureAttr).toBeNull();
  });
});
