import { readFileSync, rmSync, existsSync } from "fs";
import { join } from "path";

async function adminGlobalTeardown() {
  const pidFile = join(__dirname, ".admin-server-pid");
  if (existsSync(pidFile)) {
    const pid = parseInt(readFileSync(pidFile, "utf-8").trim(), 10);
    try {
      process.kill(pid, "SIGTERM");
      console.log(`[e2e-admin] Killed server process ${pid}`);
    } catch {
      // Process may already be dead
    }
    rmSync(pidFile, { force: true });
  }

  const tempDirFile = join(__dirname, ".admin-temp-dir");
  if (existsSync(tempDirFile)) {
    const tempDir = readFileSync(tempDirFile, "utf-8").trim();
    rmSync(tempDir, { recursive: true, force: true });
    rmSync(tempDirFile, { force: true });
    console.log(`[e2e-admin] Cleaned up temp dir: ${tempDir}`);
  }

  const authFile = join(__dirname, ".admin-auth-state.json");
  rmSync(authFile, { force: true });
}

export default adminGlobalTeardown;
