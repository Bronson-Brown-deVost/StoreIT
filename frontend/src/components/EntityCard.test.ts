import { render, screen } from "@testing-library/svelte";
import EntityCard from "./EntityCard.svelte";

// Mock the API module to prevent real fetches
vi.mock("~/api", () => ({
  getEntityPhotos: vi.fn().mockResolvedValue([]),
}));

// Mock PhotoThumbnail to avoid nested component complexity
vi.mock("./PhotoThumbnail.svelte", () => ({
  default: {
    $$render: () => "<div data-testid='photo-thumb'></div>",
  },
}));

describe("EntityCard", () => {
  it("renders the entity name", () => {
    render(EntityCard, {
      props: { href: "/locations/1", name: "Garage" },
    });
    expect(screen.getByText("Garage")).toBeInTheDocument();
  });

  it("renders as a link with the correct href", () => {
    render(EntityCard, {
      props: { href: "/locations/1", name: "Garage" },
    });
    const link = screen.getByRole("link");
    expect(link).toHaveAttribute("href", "/locations/1");
  });

  it("renders a description when provided", () => {
    render(EntityCard, {
      props: {
        href: "/locations/1",
        name: "Garage",
        description: "Main storage area",
      },
    });
    expect(screen.getByText("Main storage area")).toBeInTheDocument();
  });

  it("does not render a description when not provided", () => {
    const { container } = render(EntityCard, {
      props: { href: "/locations/1", name: "Garage" },
    });
    expect(container.querySelector("p.text-sm")).not.toBeInTheDocument();
  });

  it("renders a badge when provided", () => {
    render(EntityCard, {
      props: { href: "/locations/1", name: "Garage", badge: "3 items" },
    });
    expect(screen.getByText("3 items")).toBeInTheDocument();
  });

  it("does not render a badge when not provided", () => {
    const { container } = render(EntityCard, {
      props: { href: "/locations/1", name: "Garage" },
    });
    // Badge has specific classes
    expect(container.querySelector(".uppercase.tracking-wider")).not.toBeInTheDocument();
  });

  it("renders a colored placeholder when no photoId", () => {
    const { container } = render(EntityCard, {
      props: {
        href: "/locations/1",
        name: "Garage",
        color: "#ff0000",
      },
    });
    const placeholder = container.querySelector(
      "div[style*='background-color']"
    ) as HTMLElement;
    expect(placeholder).toBeInTheDocument();
    expect(placeholder.style.backgroundColor).toBe("rgb(255, 0, 0)");
  });

  it("uses default color #334155 when no color prop", () => {
    const { container } = render(EntityCard, {
      props: { href: "/locations/1", name: "Garage" },
    });
    const placeholder = container.querySelector(
      "div[style*='background-color']"
    ) as HTMLElement;
    expect(placeholder).toBeInTheDocument();
    expect(placeholder.style.backgroundColor).toBe("rgb(51, 65, 85)");
  });
});
