import "@testing-library/jest-dom";

// Global fetch mock
globalThis.fetch = vi.fn();

// Mock navigator.onLine
Object.defineProperty(navigator, "onLine", {
  writable: true,
  value: true,
});

// Mock localStorage
const localStorageStore: Record<string, string> = {};
const localStorageMock: Storage = {
  getItem: (key: string) => localStorageStore[key] ?? null,
  setItem: (key: string, value: string) => {
    localStorageStore[key] = value;
  },
  removeItem: (key: string) => {
    delete localStorageStore[key];
  },
  clear: () => {
    for (const key of Object.keys(localStorageStore)) {
      delete localStorageStore[key];
    }
  },
  get length() {
    return Object.keys(localStorageStore).length;
  },
  key: (index: number) => Object.keys(localStorageStore)[index] ?? null,
};
Object.defineProperty(window, "localStorage", { value: localStorageMock });

// Mock window.location for redirect tests
const locationMock = {
  href: "",
  origin: "http://localhost",
  pathname: "/",
  search: "",
  hash: "",
  assign: vi.fn(),
  replace: vi.fn(),
  reload: vi.fn(),
};
Object.defineProperty(window, "location", {
  writable: true,
  value: locationMock,
});

// Mock the svelte5-router's route action so components render in tests
vi.mock("@mateothegreat/svelte5-router", async (importOriginal) => {
  const actual = await importOriginal<Record<string, unknown>>();
  return {
    ...actual,
    route: () => ({ destroy() {} }),
    goto: vi.fn(),
  };
});

// Reset mocks between tests
beforeEach(() => {
  vi.clearAllMocks();
  localStorageMock.clear();
  locationMock.href = "";
  (navigator as any).onLine = true;
});
