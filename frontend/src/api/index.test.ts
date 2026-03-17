import {
  getLocations,
  getLocation,
  createLocation,
  updateLocation,
  getLocationChildren,
  getLocationContainers,
  getLocationItems,
  getContainer,
  createContainer,
  updateContainer,
  deleteContainer,
  moveContainer,
  getContainerAncestry,
  getItem,
  createItem,
  updateItem,
  deleteItem,
  moveItem,
  batchCreateItems,
  uploadPhoto,
  getEntityPhotos,
  photoFileUrl,
  photoThumbnailUrl,
  search,
  identifyPhoto,
  getAuthMode,
  getMe,
  localLogin,
  logout,
  listNfcTags,
  createNfcTag,
  resolveNfcTag,
  assignNfcTag,
  deleteNfcTag,
  getLocationTree,
  listContainers,
  listItems,
  switchGroup,
} from "./index";

// Mock the client module
vi.mock("./client", () => ({
  get: vi.fn(),
  post: vi.fn(),
  put: vi.fn(),
  del: vi.fn(),
  postMultipart: vi.fn(),
  fileUrl: vi.fn((path: string) => `/api/v1${path}`),
  ApiClientError: class extends Error {
    constructor(public code: string, message: string, public status: number) {
      super(message);
    }
  },
}));

import { get, post, put, del, postMultipart } from "./client";

describe("API wrapper functions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // -- Auth --
  describe("auth", () => {
    it("getAuthMode calls get with correct path", async () => {
      await getAuthMode();
      expect(get).toHaveBeenCalledWith("/auth/mode");
    });

    it("getMe calls get with correct path", async () => {
      await getMe();
      expect(get).toHaveBeenCalledWith("/auth/me");
    });

    it("localLogin posts credentials", async () => {
      await localLogin({ username: "admin", password: "pass" });
      expect(post).toHaveBeenCalledWith("/auth/local/login", {
        username: "admin",
        password: "pass",
      });
    });

    it("switchGroup puts active group", async () => {
      await switchGroup({ group_id: "g-1" });
      expect(put).toHaveBeenCalledWith("/auth/me/active-group", {
        group_id: "g-1",
      });
    });

    it("logout posts to logout", async () => {
      await logout();
      expect(post).toHaveBeenCalledWith("/auth/logout");
    });
  });

  // -- Locations --
  describe("locations", () => {
    it("getLocations", async () => {
      await getLocations();
      expect(get).toHaveBeenCalledWith("/locations");
    });

    it("getLocationTree", async () => {
      await getLocationTree();
      expect(get).toHaveBeenCalledWith("/locations/tree");
    });

    it("getLocation", async () => {
      await getLocation("loc-1");
      expect(get).toHaveBeenCalledWith("/locations/loc-1");
    });

    it("createLocation", async () => {
      await createLocation({ name: "Garage" });
      expect(post).toHaveBeenCalledWith("/locations", { name: "Garage" });
    });

    it("updateLocation", async () => {
      await updateLocation("loc-1", { name: "Updated Garage" });
      expect(put).toHaveBeenCalledWith("/locations/loc-1", {
        name: "Updated Garage",
      });
    });

    it("getLocationChildren", async () => {
      await getLocationChildren("loc-1");
      expect(get).toHaveBeenCalledWith("/locations/loc-1/children");
    });

    it("getLocationContainers", async () => {
      await getLocationContainers("loc-1");
      expect(get).toHaveBeenCalledWith("/locations/loc-1/containers");
    });

    it("getLocationItems", async () => {
      await getLocationItems("loc-1");
      expect(get).toHaveBeenCalledWith("/locations/loc-1/items");
    });
  });

  // -- Containers --
  describe("containers", () => {
    it("listContainers", async () => {
      await listContainers();
      expect(get).toHaveBeenCalledWith("/containers");
    });

    it("getContainer", async () => {
      await getContainer("con-1");
      expect(get).toHaveBeenCalledWith("/containers/con-1");
    });

    it("createContainer", async () => {
      const req = {
        parent_type: "location",
        parent_id: "loc-1",
        name: "Shelf",
      };
      await createContainer(req);
      expect(post).toHaveBeenCalledWith("/containers", req);
    });

    it("updateContainer", async () => {
      await updateContainer("con-1", { name: "Updated Shelf" });
      expect(put).toHaveBeenCalledWith("/containers/con-1", {
        name: "Updated Shelf",
      });
    });

    it("deleteContainer", async () => {
      await deleteContainer("con-1");
      expect(del).toHaveBeenCalledWith("/containers/con-1");
    });

    it("moveContainer", async () => {
      await moveContainer("con-1", {
        target_type: "location",
        target_id: "loc-2",
      });
      expect(post).toHaveBeenCalledWith("/containers/con-1/move", {
        target_type: "location",
        target_id: "loc-2",
      });
    });

    it("getContainerAncestry", async () => {
      await getContainerAncestry("con-1");
      expect(get).toHaveBeenCalledWith("/containers/con-1/ancestry");
    });
  });

  // -- Items --
  describe("items", () => {
    it("listItems", async () => {
      await listItems();
      expect(get).toHaveBeenCalledWith("/items");
    });

    it("getItem", async () => {
      await getItem("item-1");
      expect(get).toHaveBeenCalledWith("/items/item-1");
    });

    it("createItem", async () => {
      const req = {
        parent_type: "container",
        parent_id: "con-1",
        name: "Hammer",
      };
      await createItem(req);
      expect(post).toHaveBeenCalledWith("/items", req);
    });

    it("updateItem", async () => {
      await updateItem("item-1", { name: "Updated Hammer" });
      expect(put).toHaveBeenCalledWith("/items/item-1", {
        name: "Updated Hammer",
      });
    });

    it("deleteItem", async () => {
      await deleteItem("item-1");
      expect(del).toHaveBeenCalledWith("/items/item-1");
    });

    it("batchCreateItems", async () => {
      const reqs = [
        { parent_type: "container", parent_id: "c-1", name: "Item A" },
        { parent_type: "container", parent_id: "c-1", name: "Item B" },
      ];
      await batchCreateItems(reqs);
      expect(post).toHaveBeenCalledWith("/items/batch", reqs);
    });

    it("moveItem", async () => {
      await moveItem("item-1", {
        target_type: "container",
        target_id: "con-2",
      });
      expect(post).toHaveBeenCalledWith("/items/item-1/move", {
        target_type: "container",
        target_id: "con-2",
      });
    });
  });

  // -- Photos --
  describe("photos", () => {
    it("uploadPhoto sends multipart form data", async () => {
      const file = new File(["img"], "test.jpg", { type: "image/jpeg" });
      await uploadPhoto("container", "con-1", file);
      expect(postMultipart).toHaveBeenCalledWith(
        "/photos",
        expect.any(FormData)
      );
    });

    it("getEntityPhotos", async () => {
      await getEntityPhotos("container", "con-1");
      expect(get).toHaveBeenCalledWith(
        "/photos/by-entity?entity_type=container&entity_id=con-1"
      );
    });

    it("photoFileUrl returns correct URL", () => {
      expect(photoFileUrl("p-1")).toBe("/api/v1/photos/p-1/file");
    });

    it("photoThumbnailUrl returns correct URL", () => {
      expect(photoThumbnailUrl("p-1")).toBe("/api/v1/photos/p-1/thumbnail");
    });
  });

  // -- Search --
  describe("search", () => {
    it("search with query only", async () => {
      await search("hammer");
      expect(get).toHaveBeenCalledWith("/search?q=hammer");
    });

    it("search with query and limit", async () => {
      await search("hammer", 10);
      expect(get).toHaveBeenCalledWith("/search?q=hammer&limit=10");
    });
  });

  // -- Identify --
  describe("identify", () => {
    it("identifyPhoto sends file as multipart", async () => {
      const file = new File(["img"], "photo.jpg", { type: "image/jpeg" });
      await identifyPhoto(file);
      expect(postMultipart).toHaveBeenCalledWith(
        "/identify",
        expect.any(FormData)
      );
    });
  });

  // -- NFC Tags --
  describe("nfc tags", () => {
    it("listNfcTags", async () => {
      await listNfcTags();
      expect(get).toHaveBeenCalledWith("/nfc-tags");
    });

    it("createNfcTag", async () => {
      await createNfcTag({ tag_uri: "nfc://tag1" });
      expect(post).toHaveBeenCalledWith("/nfc-tags", { tag_uri: "nfc://tag1" });
    });

    it("resolveNfcTag encodes URI", async () => {
      await resolveNfcTag("nfc://tag 1");
      expect(get).toHaveBeenCalledWith(
        `/nfc-tags/resolve/${encodeURIComponent("nfc://tag 1")}`
      );
    });

    it("assignNfcTag", async () => {
      await assignNfcTag("t-1", {
        entity_type: "container",
        entity_id: "con-1",
      });
      expect(put).toHaveBeenCalledWith("/nfc-tags/t-1/assign", {
        entity_type: "container",
        entity_id: "con-1",
      });
    });

    it("deleteNfcTag", async () => {
      await deleteNfcTag("t-1");
      expect(del).toHaveBeenCalledWith("/nfc-tags/t-1");
    });
  });
});
