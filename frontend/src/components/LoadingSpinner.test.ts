import { render } from "@testing-library/svelte";
import LoadingSpinner from "./LoadingSpinner.svelte";

describe("LoadingSpinner", () => {
  it("renders a spinner element", () => {
    const { container } = render(LoadingSpinner);
    const spinner = container.querySelector(".animate-spin");
    expect(spinner).toBeInTheDocument();
  });

  it("applies default flex centering classes", () => {
    const { container } = render(LoadingSpinner);
    const wrapper = container.firstElementChild as HTMLElement;
    expect(wrapper.className).toContain("flex");
    expect(wrapper.className).toContain("items-center");
    expect(wrapper.className).toContain("justify-center");
  });

  it("accepts a custom class prop", () => {
    const { container } = render(LoadingSpinner, {
      props: { class: "mt-8 h-64" },
    });
    const wrapper = container.firstElementChild as HTMLElement;
    expect(wrapper.className).toContain("mt-8 h-64");
  });

  it("renders without a class prop (empty string default)", () => {
    const { container } = render(LoadingSpinner);
    const wrapper = container.firstElementChild as HTMLElement;
    // Should still have base classes without trailing garbage
    expect(wrapper.className).toMatch(/flex/);
  });
});
