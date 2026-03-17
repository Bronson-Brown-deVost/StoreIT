import { render, screen } from "@testing-library/svelte";
import Breadcrumbs from "./Breadcrumbs.svelte";
import type { AncestryNode } from "~/api";

describe("Breadcrumbs", () => {
  it("renders a home link", () => {
    render(Breadcrumbs, { props: { items: [] } });
    const homeLink = screen.getAllByRole("link")[0];
    expect(homeLink).toHaveAttribute("href", "/");
  });

  it("renders ancestor items as links", () => {
    const items: AncestryNode[] = [
      { entity_type: "location", id: "loc-1", name: "Garage" },
      { entity_type: "container", id: "con-1", name: "Shelf A" },
    ];
    render(Breadcrumbs, { props: { items } });

    const garageLink = screen.getByText("Garage");
    expect(garageLink.closest("a")).toHaveAttribute("href", "/locations/loc-1");

    const shelfLink = screen.getByText("Shelf A");
    expect(shelfLink.closest("a")).toHaveAttribute("href", "/containers/con-1");
  });

  it("renders current label as non-link text", () => {
    render(Breadcrumbs, {
      props: {
        items: [{ entity_type: "location", id: "loc-1", name: "Garage" }],
        current: "Tool Box",
      },
    });
    const currentEl = screen.getByText("Tool Box");
    expect(currentEl.tagName).toBe("SPAN");
    expect(currentEl.closest("a")).toBeNull();
  });

  it("does not render current label when not provided", () => {
    render(Breadcrumbs, {
      props: {
        items: [{ entity_type: "location", id: "loc-1", name: "Garage" }],
      },
    });
    // Only home + one ancestor link
    const links = screen.getAllByRole("link");
    expect(links).toHaveLength(2);
  });

  it("renders with empty items and no current", () => {
    render(Breadcrumbs, { props: { items: [] } });
    const links = screen.getAllByRole("link");
    expect(links).toHaveLength(1); // Just the home link
  });

  it("renders separator chevrons between items", () => {
    const items: AncestryNode[] = [
      { entity_type: "location", id: "loc-1", name: "Garage" },
    ];
    const { container } = render(Breadcrumbs, {
      props: { items, current: "Box" },
    });
    // Each separator is a polyline with chevron points
    const chevrons = container.querySelectorAll("polyline");
    // One chevron before "Garage", one before "Box"
    expect(chevrons.length).toBe(2);
  });
});
