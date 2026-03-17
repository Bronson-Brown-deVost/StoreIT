import { render, screen, fireEvent } from "@testing-library/svelte";
import PrintLabel from "./PrintLabel.svelte";

// Mock qrcode-generator as a factory that returns an object with the expected methods
vi.mock("qrcode-generator", () => {
  const factory = () => ({
    addData: vi.fn(),
    make: vi.fn(),
    createSvgTag: () => '<svg data-testid="qr-mock"></svg>',
  });
  return { default: factory };
});

describe("PrintLabel", () => {
  const baseProps = {
    name: "Tool Box",
    entityType: "container",
    entityId: "abc-123",
    onClose: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the entity name", () => {
    render(PrintLabel, { props: baseProps });
    expect(screen.getByText("Tool Box")).toBeInTheDocument();
  });

  it("renders the entity type", () => {
    render(PrintLabel, { props: baseProps });
    expect(screen.getByText("container")).toBeInTheDocument();
  });

  it("renders the dialog title", () => {
    render(PrintLabel, { props: baseProps });
    expect(screen.getByText("Print Label")).toBeInTheDocument();
  });

  it("renders Cancel and Print buttons", () => {
    render(PrintLabel, { props: baseProps });
    expect(screen.getByText("Cancel")).toBeInTheDocument();
    expect(screen.getByText("Print")).toBeInTheDocument();
  });

  it("calls onClose when Cancel is clicked", async () => {
    const onClose = vi.fn();
    render(PrintLabel, { props: { ...baseProps, onClose } });
    await fireEvent.click(screen.getByText("Cancel"));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("renders description when provided", () => {
    render(PrintLabel, {
      props: { ...baseProps, description: "Red metal box" },
    });
    expect(screen.getByText("Red metal box")).toBeInTheDocument();
  });

  it("renders location path when provided", () => {
    render(PrintLabel, {
      props: { ...baseProps, locationPath: ["Garage", "Shelf A"] },
    });
    expect(screen.getByText("Garage > Shelf A")).toBeInTheDocument();
  });

  it("opens a print window when Print is clicked", async () => {
    const mockPrintWindow = {
      document: {
        write: vi.fn(),
        close: vi.fn(),
      },
    };
    vi.spyOn(window, "open").mockReturnValue(mockPrintWindow as any);

    render(PrintLabel, { props: baseProps });
    await fireEvent.click(screen.getByText("Print"));

    expect(window.open).toHaveBeenCalledWith("", "_blank", "width=400,height=400");
    expect(mockPrintWindow.document.write).toHaveBeenCalled();
    expect(mockPrintWindow.document.close).toHaveBeenCalled();

    // Verify the written HTML contains entity name
    const html = mockPrintWindow.document.write.mock.calls[0][0];
    expect(html).toContain("Tool Box");
    expect(html).toContain("container");
  });

  it("calls onClose when clicking the backdrop", async () => {
    const onClose = vi.fn();
    const { container } = render(PrintLabel, {
      props: { ...baseProps, onClose },
    });
    // The backdrop is the outermost fixed div
    const backdrop = container.querySelector(".fixed.inset-0") as HTMLElement;
    // Simulate clicking the backdrop itself (target === currentTarget)
    await fireEvent.click(backdrop);
    expect(onClose).toHaveBeenCalled();
  });
});
