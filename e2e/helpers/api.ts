import { APIRequestContext } from "@playwright/test";

let counter = 0;
function uniquePrefix(): string {
  return `e2e_${Date.now()}_${++counter}`;
}

export class TestApi {
  constructor(private request: APIRequestContext) {}

  async createLocation(name?: string): Promise<{ id: string; name: string }> {
    const locName = name ?? `${uniquePrefix()}_location`;
    const res = await this.request.post("/api/v1/locations", {
      data: { name: locName },
    });
    if (!res.ok()) throw new Error(`createLocation failed: ${res.status()} ${await res.text()}`);
    return res.json();
  }

  async createContainer(
    name: string | undefined,
    parentType: "location" | "container",
    parentId: string,
    opts?: { color?: string; description?: string }
  ): Promise<{ id: string; name: string; color?: string }> {
    const cName = name ?? `${uniquePrefix()}_container`;
    const data: Record<string, unknown> = {
      name: cName,
      parent_type: parentType,
      parent_id: parentId,
    };
    if (opts?.color) data.color = opts.color;
    if (opts?.description) data.description = opts.description;

    const res = await this.request.post("/api/v1/containers", { data });
    if (!res.ok()) throw new Error(`createContainer failed: ${res.status()} ${await res.text()}`);
    return res.json();
  }

  async createItem(
    name: string | undefined,
    parentType: "location" | "container",
    parentId: string,
    opts?: { description?: string; category?: string }
  ): Promise<{ id: string; name: string }> {
    const iName = name ?? `${uniquePrefix()}_item`;
    const data: Record<string, unknown> = {
      name: iName,
      parent_type: parentType,
      parent_id: parentId,
    };
    if (opts?.description) data.description = opts.description;
    if (opts?.category) data.category = opts.category;

    const res = await this.request.post("/api/v1/items", { data });
    if (!res.ok()) throw new Error(`createItem failed: ${res.status()} ${await res.text()}`);
    return res.json();
  }

  async createNfcTag(tagUri: string): Promise<{ id: string; tag_uri: string }> {
    const res = await this.request.post("/api/v1/nfc-tags", {
      data: { tag_uri: tagUri },
    });
    if (!res.ok()) throw new Error(`createNfcTag failed: ${res.status()} ${await res.text()}`);
    return res.json();
  }

  async assignNfcTag(tagId: string, entityType: string, entityId: string): Promise<void> {
    const res = await this.request.put(`/api/v1/nfc-tags/${tagId}/assign`, {
      data: { entity_type: entityType, entity_id: entityId },
    });
    if (!res.ok()) throw new Error(`assignNfcTag failed: ${res.status()} ${await res.text()}`);
  }

  async moveItem(itemId: string, targetType: "location" | "container", targetId: string): Promise<void> {
    const res = await this.request.post(`/api/v1/items/${itemId}/move`, {
      data: { target_type: targetType, target_id: targetId },
    });
    if (!res.ok()) throw new Error(`moveItem failed: ${res.status()} ${await res.text()}`);
  }

  async moveContainer(containerId: string, targetType: "location" | "container", targetId: string): Promise<void> {
    const res = await this.request.post(`/api/v1/containers/${containerId}/move`, {
      data: { target_type: targetType, target_id: targetId },
    });
    if (!res.ok()) throw new Error(`moveContainer failed: ${res.status()} ${await res.text()}`);
  }

  async deleteItem(itemId: string): Promise<void> {
    const res = await this.request.delete(`/api/v1/items/${itemId}`);
    if (!res.ok()) throw new Error(`deleteItem failed: ${res.status()} ${await res.text()}`);
  }

  async deleteContainer(containerId: string): Promise<void> {
    const res = await this.request.delete(`/api/v1/containers/${containerId}`);
    if (!res.ok()) throw new Error(`deleteContainer failed: ${res.status()} ${await res.text()}`);
  }

  async deleteLocation(locationId: string): Promise<void> {
    const res = await this.request.delete(`/api/v1/locations/${locationId}`);
    if (!res.ok()) throw new Error(`deleteLocation failed: ${res.status()} ${await res.text()}`);
  }

  async updateItem(itemId: string, data: Record<string, unknown>): Promise<{ id: string; name: string }> {
    const res = await this.request.put(`/api/v1/items/${itemId}`, { data });
    if (!res.ok()) throw new Error(`updateItem failed: ${res.status()} ${await res.text()}`);
    return res.json();
  }

  async updateContainer(containerId: string, data: Record<string, unknown>): Promise<{ id: string; name: string }> {
    const res = await this.request.put(`/api/v1/containers/${containerId}`, { data });
    if (!res.ok()) throw new Error(`updateContainer failed: ${res.status()} ${await res.text()}`);
    return res.json();
  }

  async updateLocation(locationId: string, data: Record<string, unknown>): Promise<{ id: string; name: string }> {
    const res = await this.request.put(`/api/v1/locations/${locationId}`, { data });
    if (!res.ok()) throw new Error(`updateLocation failed: ${res.status()} ${await res.text()}`);
    return res.json();
  }

  async uploadPhoto(entityType: string, entityId: string, filePath: string): Promise<{ id: string }> {
    const res = await this.request.post("/api/v1/photos", {
      multipart: {
        entity_type: entityType,
        entity_id: entityId,
        file: {
          name: "test.png",
          mimeType: "image/png",
          buffer: Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==", "base64"),
        },
      },
    });
    if (!res.ok()) throw new Error(`uploadPhoto failed: ${res.status()} ${await res.text()}`);
    return res.json();
  }

  async batchCreateItems(
    parentType: "location" | "container",
    parentId: string,
    items: Array<{ name: string; category?: string; material?: string; color?: string; quantity?: number }>
  ): Promise<{ id: string; name: string }[]> {
    const data = items.map((item) => ({
      parent_type: parentType,
      parent_id: parentId,
      name: item.name,
      category: item.category ?? null,
      material: item.material ?? null,
      color: item.color ?? null,
      quantity: item.quantity ?? 1,
    }));
    const res = await this.request.post("/api/v1/items/batch", { data });
    if (!res.ok()) throw new Error(`batchCreateItems failed: ${res.status()} ${await res.text()}`);
    return res.json();
  }

  async deleteNfcTag(tagId: string): Promise<void> {
    const res = await this.request.delete(`/api/v1/nfc-tags/${tagId}`);
    if (!res.ok()) throw new Error(`deleteNfcTag failed: ${res.status()} ${await res.text()}`);
  }

  async getItem(itemId: string): Promise<Record<string, unknown>> {
    const res = await this.request.get(`/api/v1/items/${itemId}`);
    if (!res.ok()) throw new Error(`getItem failed: ${res.status()} ${await res.text()}`);
    return res.json();
  }

  async search(query: string): Promise<{ results: Array<{ entity_type: string; entity_id: string }> }> {
    const res = await this.request.get(`/api/v1/search?q=${encodeURIComponent(query)}`);
    if (!res.ok()) throw new Error(`search failed: ${res.status()} ${await res.text()}`);
    return res.json();
  }
}
