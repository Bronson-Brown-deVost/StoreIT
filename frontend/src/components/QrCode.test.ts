import { render } from "@testing-library/svelte";
import QrCode from "./QrCode.svelte";

describe("QrCode", () => {
  it("renders an SVG from data string", () => {
    const { container } = render(QrCode, {
      props: { data: "https://example.com" },
    });
    const svg = container.querySelector("svg");
    expect(svg).toBeInTheDocument();
  });

  it("uses default size of 128px", () => {
    const { container } = render(QrCode, {
      props: { data: "test-data" },
    });
    const wrapper = container.firstElementChild as HTMLElement;
    expect(wrapper.style.width).toBe("128px");
    expect(wrapper.style.height).toBe("128px");
  });

  it("accepts a custom size", () => {
    const { container } = render(QrCode, {
      props: { data: "test-data", size: 256 },
    });
    const wrapper = container.firstElementChild as HTMLElement;
    expect(wrapper.style.width).toBe("256px");
    expect(wrapper.style.height).toBe("256px");
  });

  it("accepts a custom class", () => {
    const { container } = render(QrCode, {
      props: { data: "test-data", class: "my-qr" },
    });
    const wrapper = container.firstElementChild as HTMLElement;
    expect(wrapper.className).toContain("my-qr");
  });

  it("generates different SVGs for different data", () => {
    const { container: c1 } = render(QrCode, {
      props: { data: "data-one" },
    });
    const { container: c2 } = render(QrCode, {
      props: { data: "data-two" },
    });
    const svg1 = c1.querySelector("svg")?.outerHTML;
    const svg2 = c2.querySelector("svg")?.outerHTML;
    expect(svg1).not.toEqual(svg2);
  });
});
