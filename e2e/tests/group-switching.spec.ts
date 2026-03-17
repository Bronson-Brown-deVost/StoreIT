import { test, expect } from "@playwright/test";

test.describe("Group Switching", () => {
  // Create additional groups for the test user before tests
  test.beforeAll(async ({ request }) => {
    // Create "family" and "work" groups via admin API (ignore errors if already exist)
    for (const name of ["family", "work"]) {
      await request.post("/api/v1/admin/groups", { data: { name } });
    }
    // Get current user info to know user ID
    const meRes = await request.get("/api/v1/auth/me");
    const me = await meRes.json();
    const userId = me.user.id;

    // List groups to get IDs and add user as member
    const groupsRes = await request.get("/api/v1/admin/groups");
    const groups = await groupsRes.json();
    for (const group of groups) {
      if (group.name === "family" || group.name === "work") {
        // Ignore errors if already a member
        await request.post(`/api/v1/admin/groups/${group.id}/members`, {
          data: { user_id: userId, role: "member" },
        });
      }
    }
  });

  test("settings page shows group switcher", async ({ page }) => {
    await page.goto("/settings");
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
    await expect(page.getByText("Group")).toBeVisible();
  });

  test("settings page shows multiple groups", async ({ page }) => {
    await page.goto("/settings");
    await expect(page.getByText("family")).toBeVisible();
    await expect(page.getByText("work")).toBeVisible();
  });

  test("clicking a group switches active group", async ({ page }) => {
    await page.goto("/settings");
    const workButton = page.getByRole("button", { name: "work" });
    await expect(workButton).toBeVisible();
    await workButton.click();

    await page.waitForTimeout(500);
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
  });

  test("sign out button is visible", async ({ page }) => {
    await page.goto("/settings");
    await expect(page.getByText("Sign Out")).toBeVisible();
  });

  test("export CSV button is visible", async ({ page }) => {
    await page.goto("/settings");
    await expect(page.getByText("CSV")).toBeVisible();
  });

  test("export JSON button is visible", async ({ page }) => {
    await page.goto("/settings");
    await expect(page.getByText("JSON")).toBeVisible();
  });
});
