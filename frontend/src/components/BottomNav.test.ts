import { render, screen } from "@testing-library/svelte";
import BottomNav from "./BottomNav.svelte";

// Mock the auth module
vi.mock("~/lib/auth.svelte", () => ({
  auth: {
    user: { is_admin: false },
    loading: false,
    data: undefined,
    groups: [],
    activeGroupId: undefined,
    authMode: "local",
  },
}));

describe("BottomNav", () => {
  it("renders a nav element", () => {
    render(BottomNav);
    expect(screen.getByRole("navigation")).toBeInTheDocument();
  });

  it("renders Browse link", () => {
    render(BottomNav);
    expect(screen.getByText("Browse")).toBeInTheDocument();
    const link = screen.getByText("Browse").closest("a");
    expect(link).toHaveAttribute("href", "/");
  });

  it("renders Search link", () => {
    render(BottomNav);
    expect(screen.getByText("Search")).toBeInTheDocument();
    const link = screen.getByText("Search").closest("a");
    expect(link).toHaveAttribute("href", "/search");
  });

  it("renders Add link", () => {
    render(BottomNav);
    expect(screen.getByText("Add")).toBeInTheDocument();
    const link = screen.getByText("Add").closest("a");
    expect(link).toHaveAttribute("href", "/add");
  });

  it("renders Settings link", () => {
    render(BottomNav);
    expect(screen.getByText("Settings")).toBeInTheDocument();
    const link = screen.getByText("Settings").closest("a");
    expect(link).toHaveAttribute("href", "/settings");
  });

  it("does not render Admin link when user is not admin", () => {
    render(BottomNav);
    expect(screen.queryByText("Admin")).not.toBeInTheDocument();
  });
});

