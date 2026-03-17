import { execSync, spawn } from "child_process";
import { mkdtempSync, writeFileSync } from "fs";
import { tmpdir } from "os";
import { join } from "path";

const BASE_URL = "http://127.0.0.1:3100";
const ADMIN_BASE_URL = "http://127.0.0.1:3101";
const PROJECT_ROOT = join(__dirname, "../..");

async function globalSetup() {
  // 1. Build frontend
  console.log("[e2e] Building frontend...");
  execSync("npm run build", { cwd: join(PROJECT_ROOT, "frontend"), stdio: "inherit" });

  // 2. Build backend (use nix develop if available, otherwise bare cargo)
  console.log("[e2e] Building backend...");
  try {
    execSync("nix develop --command cargo build -p storeit-server", {
      cwd: PROJECT_ROOT,
      stdio: "inherit",
    });
  } catch {
    execSync("cargo build -p storeit-server", { cwd: PROJECT_ROOT, stdio: "inherit" });
  }

  const serverBin = join(PROJECT_ROOT, "target/debug/storeit-server");

  // 3. Start main server (local auth, port 3100) — used by non-admin tests
  console.log("[e2e] Starting main server (local auth, port 3100)...");
  const tempDir = mkdtempSync(join(tmpdir(), "storeit-e2e-"));
  const dbPath = join(tempDir, "e2e-test.db");
  const imagePath = join(tempDir, "images");

  const server = spawn(serverBin, [], {
    env: {
      ...process.env,
      STOREIT_BIND: "127.0.0.1:3100",
      DATABASE_URL: `sqlite:${dbPath}?mode=rwc`,
      STOREIT_IMAGE_PATH: imagePath,
      STOREIT_ADMIN_USERNAME: "testuser",
      STOREIT_ADMIN_PASSWORD: "testpass123",
      STOREIT_ADMIN_EMAIL: "testuser@test.com",
      STOREIT_ADMIN_DISPLAY_NAME: "Test User",
      STOREIT_SESSION_SECRET: "e2e-test-secret-must-be-at-least-32-chars-long!",
    },
    stdio: "inherit",
  });

  writeFileSync(join(__dirname, ".server-pid"), String(server.pid));
  writeFileSync(join(__dirname, ".temp-dir"), tempDir);

  // 4. Start admin server (local auth, port 3101) — used by admin tests
  console.log("[e2e] Starting admin server (local auth, port 3101)...");
  const adminTempDir = mkdtempSync(join(tmpdir(), "storeit-e2e-admin-"));
  const adminDbPath = join(adminTempDir, "e2e-admin.db");
  const adminImagePath = join(adminTempDir, "images");

  const adminServer = spawn(serverBin, [], {
    env: {
      ...process.env,
      STOREIT_BIND: "127.0.0.1:3101",
      DATABASE_URL: `sqlite:${adminDbPath}?mode=rwc`,
      STOREIT_IMAGE_PATH: adminImagePath,
      STOREIT_ADMIN_USERNAME: "admin",
      STOREIT_ADMIN_PASSWORD: "testpass123",
      STOREIT_ADMIN_EMAIL: "admin@test.com",
      STOREIT_ADMIN_DISPLAY_NAME: "Test Admin",
      STOREIT_SESSION_SECRET: "e2e-admin-secret-must-be-at-least-32-chars-long",
    },
    stdio: "inherit",
  });

  writeFileSync(join(__dirname, ".admin-server-pid"), String(adminServer.pid));
  writeFileSync(join(__dirname, ".admin-temp-dir"), adminTempDir);

  // 5. Wait for both servers to be ready
  console.log("[e2e] Waiting for servers...");
  await Promise.all([
    waitForServer(BASE_URL, 30_000),
    waitForServer(ADMIN_BASE_URL, 30_000),
  ]);
  console.log("[e2e] Both servers are ready");

  // 6. Login to main server as test user
  console.log("[e2e] Authenticating test user...");
  const sessionCookie = await loginLocal(BASE_URL, "testuser", "testpass123");
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
  writeFileSync(join(__dirname, ".auth-state.json"), JSON.stringify(storageState, null, 2));
  console.log("[e2e] Test user authenticated");

  // 7. Login to admin server as admin
  console.log("[e2e] Authenticating admin user...");
  const adminSessionCookie = await loginLocal(ADMIN_BASE_URL, "admin", "testpass123");
  const adminStorageState = {
    cookies: [
      {
        name: "storeit_session",
        value: adminSessionCookie,
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
  writeFileSync(join(__dirname, ".admin-auth-state.json"), JSON.stringify(adminStorageState, null, 2));
  console.log("[e2e] Admin authenticated");
}

async function waitForServer(baseUrl: string, timeoutMs: number) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(`${baseUrl}/api/v1/auth/mode`);
      if (res.ok) return;
    } catch {
      // Server not ready yet
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error(`Server at ${baseUrl} did not start within timeout`);
}

async function loginLocal(baseUrl: string, username: string, password: string): Promise<string> {
  const res = await fetch(`${baseUrl}/api/v1/auth/local/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password }),
    redirect: "manual",
  });

  const setCookies = res.headers.getSetCookie();
  for (const sc of setCookies) {
    const match = sc.match(/^storeit_session=([^;]+)/);
    if (match) return match[1];
  }

  throw new Error(`Login failed for ${username} at ${baseUrl}: ${res.status}`);
}

export default globalSetup;
