import { get, post, put, del, postMultipart, fileUrl, ApiClientError } from "./client";

// Mock the offline queue module
vi.mock("~/lib/offlineQueue", () => ({
  queueMutation: vi.fn(),
}));

import { queueMutation } from "~/lib/offlineQueue";

function mockFetchResponse(status: number, body?: unknown) {
  const response = {
    ok: status >= 200 && status < 300,
    status,
    json: vi.fn().mockResolvedValue(body),
    text: vi.fn().mockResolvedValue(JSON.stringify(body)),
  } as unknown as Response;
  (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(response);
  return response;
}

describe("client", () => {
  describe("get", () => {
    it("fetches from /api/v1 + path and returns JSON", async () => {
      mockFetchResponse(200, { id: "1", name: "Test" });
      const result = await get<{ id: string; name: string }>("/locations");
      expect(fetch).toHaveBeenCalledWith("/api/v1/locations", {
        credentials: "same-origin",
      });
      expect(result).toEqual({ id: "1", name: "Test" });
    });

    it("throws ApiClientError on non-ok response", async () => {
      mockFetchResponse(404, {
        error: { code: "not_found", message: "Not found" },
      });
      await expect(get("/locations/999")).rejects.toThrow(ApiClientError);
      await expect(get("/locations/999")).rejects.toMatchObject({
        code: "not_found",
        status: 404,
      });
    });

    it("dispatches unauthenticated event on 401", async () => {
      mockFetchResponse(401);
      const handler = vi.fn();
      window.addEventListener("storeit:unauthenticated", handler);
      await expect(get("/auth/me")).rejects.toThrow(ApiClientError);
      expect(handler).toHaveBeenCalled();
      window.removeEventListener("storeit:unauthenticated", handler);
    });
  });

  describe("post", () => {
    it("sends POST with JSON body", async () => {
      mockFetchResponse(201, { id: "new-1" });
      const result = await post("/locations", { name: "Garage" });
      expect(fetch).toHaveBeenCalledWith("/api/v1/locations", {
        credentials: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: "Garage" }),
      });
      expect(result).toEqual({ id: "new-1" });
    });

    it("queues mutation when offline and fetch throws", async () => {
      (fetch as ReturnType<typeof vi.fn>).mockRejectedValue(
        new TypeError("Failed to fetch")
      );
      (navigator as any).onLine = false;

      await expect(
        post("/locations", { name: "Test" })
      ).rejects.toThrow(ApiClientError);

      expect(queueMutation).toHaveBeenCalledWith({
        url: "/api/v1/locations",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: "Test" }),
      });
    });

    it("rethrows when online and fetch fails", async () => {
      (fetch as ReturnType<typeof vi.fn>).mockRejectedValue(
        new TypeError("Failed to fetch")
      );
      (navigator as any).onLine = true;

      await expect(post("/locations", { name: "Test" })).rejects.toThrow(
        "Failed to fetch"
      );
      expect(queueMutation).not.toHaveBeenCalled();
    });
  });

  describe("put", () => {
    it("sends PUT with JSON body", async () => {
      mockFetchResponse(200, { id: "1", name: "Updated" });
      const result = await put("/locations/1", { name: "Updated" });
      expect(fetch).toHaveBeenCalledWith("/api/v1/locations/1", {
        credentials: "same-origin",
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: "Updated" }),
      });
      expect(result).toEqual({ id: "1", name: "Updated" });
    });

    it("queues mutation when offline", async () => {
      (fetch as ReturnType<typeof vi.fn>).mockRejectedValue(
        new TypeError("Failed to fetch")
      );
      (navigator as any).onLine = false;

      await expect(
        put("/locations/1", { name: "Updated" })
      ).rejects.toThrow(ApiClientError);

      expect(queueMutation).toHaveBeenCalledWith({
        url: "/api/v1/locations/1",
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: "Updated" }),
      });
    });
  });

  describe("del", () => {
    it("sends DELETE request", async () => {
      mockFetchResponse(204);
      await del("/locations/1");
      expect(fetch).toHaveBeenCalledWith("/api/v1/locations/1", {
        credentials: "same-origin",
        method: "DELETE",
      });
    });

    it("returns undefined for 204 response", async () => {
      mockFetchResponse(204);
      const result = await del("/locations/1");
      expect(result).toBeUndefined();
    });

    it("queues mutation when offline", async () => {
      (fetch as ReturnType<typeof vi.fn>).mockRejectedValue(
        new TypeError("Failed to fetch")
      );
      (navigator as any).onLine = false;

      await expect(del("/locations/1")).rejects.toThrow(ApiClientError);

      expect(queueMutation).toHaveBeenCalledWith({
        url: "/api/v1/locations/1",
        method: "DELETE",
        headers: {},
      });
    });
  });

  describe("postMultipart", () => {
    it("sends POST with FormData (no Content-Type header)", async () => {
      mockFetchResponse(201, { id: "photo-1" });
      const fd = new FormData();
      fd.append("file", new Blob(["test"]), "test.jpg");

      const result = await postMultipart("/photos", fd);
      expect(fetch).toHaveBeenCalledWith("/api/v1/photos", {
        credentials: "same-origin",
        method: "POST",
        body: fd,
      });
      expect(result).toEqual({ id: "photo-1" });
    });
  });

  describe("fileUrl", () => {
    it("returns /api/v1 + path", () => {
      expect(fileUrl("/photos/1/file")).toBe("/api/v1/photos/1/file");
    });
  });

  describe("handleResponse error parsing", () => {
    it("falls back to HTTP status message when JSON parsing fails", async () => {
      const response = {
        ok: false,
        status: 500,
        json: vi.fn().mockRejectedValue(new Error("Invalid JSON")),
      } as unknown as Response;
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(response);

      await expect(get("/fail")).rejects.toMatchObject({
        code: "unknown",
        message: "HTTP 500",
        status: 500,
      });
    });
  });
});
