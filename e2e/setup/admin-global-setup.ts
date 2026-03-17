import { spawn } from "child_process";
import { mkdtempSync, writeFileSync } from "fs";
import { tmpdir } from "os";
import { join } from "path";

const BASE_URL = "http://127.0.0.1:3101";
const PROJECT_ROOT = join(__dirname, "../..");

async function adminGlobalSetup() {
  // Create temp directory for test DB + images
  const tempDir = mkdtempSync(join(tmpdir(), "storeit-e2e-admin-"));
  const dbPath = join(tempDir, "e2e-admin.db");
  const imagePath = join(tempDir, "images");

  // Start the backend server in local auth mode
  console.log("[e2e-admin] Starting backend server (local auth)...");
  const serverBin = join(PROJECT_ROOT, "target/debug/storeit-server");
  const server = spawn(serverBin, [], {
    env: {
      ...process.env,
      STOREIT_BIND: "127.0.0.1:3101",
      DATABASE_URL: `sqlite:${dbPath}?mode=rwc`,
      STOREIT_IMAGE_PATH: imagePath,
      // No STOREIT_AUTH_ISSUER → local auth mode
      STOREIT_ADMIN_USERNAME: "admin",
      STOREIT_ADMIN_PASSWORD: "testpass123",
      STOREIT_ADMIN_EMAIL: "admin@test.com",
      STOREIT_ADMIN_DISPLAY_NAME: "Test Admin",
      STOREIT_SESSION_SECRET: "e2e-admin-secret-must-be-at-least-32-chars-long",
    },
    stdio: "inherit",
  });

  writeFileSync(join(__dirname, ".admin-server-pid"), String(server.pid));
  writeFileSync(join(__dirname, ".admin-temp-dir"), tempDir);

  // Wait for server to be ready
  console.log("[e2e-admin] Waiting for server...");
  await waitForServer(30_000);
  console.log("[e2e-admin] Server is ready");

  // Login as admin via local auth
  console.log("[e2e-admin] Authenticating as admin...");
  const sessionCookie = await loginAsAdmin();
  console.log("[e2e-admin] Authenticated successfully");

  // Save auth state
  const storageState = {
    cookies: [
      {
        name: "storeit_session",
        value: sessionCookie,
        domain: "127.0.0.1",
        path: "/",
        httpOnly: true,
        secure: false,
        sameSite: "Lax" as const,
        expires: -1,
      },
    ],
    origins: [],
  };
  writeFileSync(join(__dirname, ".admin-auth-state.json"), JSON.stringify(storageState, null, 2));
}

async function waitForServer(timeoutMs: number) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/auth/mode`);
      if (res.ok) return;
    } catch {
      // Server not ready yet
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error("Admin server did not start within timeout");
}

async function loginAsAdmin(): Promise<string> {
  const res = await fetch(`${BASE_URL}/api/v1/auth/local/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username: "admin", password: "testpass123" }),
    redirect: "manual",
  });

  // The local login endpoint sets a storeit_session cookie
  const setCookies = res.headers.getSetCookie();
  for (const sc of setCookies) {
    const match = sc.match(/^storeit_session=([^;]+)/);
    if (match) return match[1];
  }

  // If no set-cookie, the session might be in the response body
  if (res.ok) {
    const body = await res.json() as Record<string, unknown>;
    // Check if response has session info - try to extract cookie from response
    if (body.session_id) return body.session_id as string;
  }

  throw new Error(`Admin login failed: ${res.status} ${await res.text()}`);
}

export default adminGlobalSetup;
