import { readFileSync, rmSync, existsSync } from "fs";
import { join } from "path";

async function globalTeardown() {
  // Kill the OIDC server process
  killServer(join(__dirname, ".server-pid"), "OIDC");

  // Kill the admin server process
  killServer(join(__dirname, ".admin-server-pid"), "admin");

  // Clean up OIDC temp directory
  cleanTempDir(join(__dirname, ".temp-dir"), "OIDC");

  // Clean up admin temp directory
  cleanTempDir(join(__dirname, ".admin-temp-dir"), "admin");

  // Clean up auth state files
  rmSync(join(__dirname, ".auth-state.json"), { force: true });
  rmSync(join(__dirname, ".admin-auth-state.json"), { force: true });
}

function killServer(pidFile: string, label: string) {
  if (existsSync(pidFile)) {
    const pid = parseInt(readFileSync(pidFile, "utf-8").trim(), 10);
    try {
      process.kill(pid, "SIGTERM");
      console.log(`[e2e] Killed ${label} server process ${pid}`);
    } catch {
      // Process may already be dead
    }
    rmSync(pidFile, { force: true });
  }
}

function cleanTempDir(tempDirFile: string, label: string) {
  if (existsSync(tempDirFile)) {
    const tempDir = readFileSync(tempDirFile, "utf-8").trim();
    rmSync(tempDir, { recursive: true, force: true });
    rmSync(tempDirFile, { force: true });
    console.log(`[e2e] Cleaned up ${label} temp dir: ${tempDir}`);
  }
}

export default globalTeardown;
