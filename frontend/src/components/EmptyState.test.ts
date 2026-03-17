import { render, screen, fireEvent } from "@testing-library/svelte";
import EmptyState from "./EmptyState.svelte";

describe("EmptyState", () => {
  it("renders the message text", () => {
    render(EmptyState, { props: { message: "Nothing here yet" } });
    expect(screen.getByText("Nothing here yet")).toBeInTheDocument();
  });

  it("renders an SVG icon", () => {
    const { container } = render(EmptyState, {
      props: { message: "Empty" },
    });
    expect(container.querySelector("svg")).toBeInTheDocument();
  });

  it("does not render a button when no action props are provided", () => {
    render(EmptyState, { props: { message: "No items" } });
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("renders an action button when actionLabel and onAction are provided", () => {
    const onAction = vi.fn();
    render(EmptyState, {
      props: {
        message: "No items found",
        actionLabel: "Add Item",
        onAction,
      },
    });
    const button = screen.getByRole("button", { name: "Add Item" });
    expect(button).toBeInTheDocument();
  });

  it("calls onAction when the button is clicked", async () => {
    const onAction = vi.fn();
    render(EmptyState, {
      props: {
        message: "No items found",
        actionLabel: "Add Item",
        onAction,
      },
    });
    const button = screen.getByRole("button", { name: "Add Item" });
    await fireEvent.click(button);
    expect(onAction).toHaveBeenCalledOnce();
  });

  it("does not render button when only actionLabel is provided (no onAction)", () => {
    render(EmptyState, {
      props: { message: "Empty", actionLabel: "Do something" },
    });
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});
