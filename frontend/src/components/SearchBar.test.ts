import { render, screen, fireEvent } from "@testing-library/svelte";
import SearchBar from "./SearchBar.svelte";

describe("SearchBar", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders an input with default placeholder", () => {
    render(SearchBar, { props: { onSearch: vi.fn() } });
    expect(screen.getByPlaceholderText("Search...")).toBeInTheDocument();
  });

  it("renders with a custom placeholder", () => {
    render(SearchBar, {
      props: { onSearch: vi.fn(), placeholder: "Find items..." },
    });
    expect(screen.getByPlaceholderText("Find items...")).toBeInTheDocument();
  });

  it("renders with an initial value", () => {
    render(SearchBar, {
      props: { onSearch: vi.fn(), value: "test query" },
    });
    const input = screen.getByPlaceholderText("Search...") as HTMLInputElement;
    expect(input.value).toBe("test query");
  });

  it("debounces onSearch calls by 300ms", async () => {
    const onSearch = vi.fn();
    render(SearchBar, { props: { onSearch } });

    const input = screen.getByPlaceholderText("Search...");
    await fireEvent.input(input, { target: { value: "hello" } });

    // Not called yet (debounce pending)
    expect(onSearch).not.toHaveBeenCalledWith("hello");

    // Advance past debounce
    vi.advanceTimersByTime(300);
    expect(onSearch).toHaveBeenCalledWith("hello");
  });

  it("shows a clear button when input has a value and clears on click", async () => {
    const onSearch = vi.fn();
    render(SearchBar, {
      props: { onSearch, value: "something" },
    });

    // Clear button should be visible
    const clearBtn = screen.getByRole("button");
    expect(clearBtn).toBeInTheDocument();

    await fireEvent.click(clearBtn);

    // onSearch should be called immediately with empty string (clear bypasses debounce)
    expect(onSearch).toHaveBeenCalledWith("");
  });

  it("does not show clear button when input is empty", () => {
    render(SearchBar, { props: { onSearch: vi.fn() } });
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});
