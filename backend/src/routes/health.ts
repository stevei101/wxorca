import { Elysia } from "elysia";

export const healthRoutes = new Elysia({ prefix: "/api" })
  .get(
    "/health",
    () => ({
      status: "healthy",
      timestamp: new Date().toISOString(),
      version: "0.1.0",
    }),
    {
      detail: {
        tags: ["Health"],
        summary: "Health check",
        description: "Check if the API is running",
      },
    }
  )
  .get(
    "/ready",
    async () => {
      // Check if Rust agent is available
      const agentAvailable = await checkAgentAvailability();

      return {
        status: agentAvailable ? "ready" : "degraded",
        timestamp: new Date().toISOString(),
        checks: {
          agent: agentAvailable ? "available" : "unavailable",
        },
      };
    },
    {
      detail: {
        tags: ["Health"],
        summary: "Readiness check",
        description: "Check if the API is ready to serve requests",
      },
    }
  );

async function checkAgentAvailability(): Promise<boolean> {
  try {
    // Try to spawn the CLI to check if it's available
    const proc = Bun.spawn(["./target/release/wxorca-cli", "--help"], {
      cwd: process.cwd().replace("/backend", ""),
      stdout: "pipe",
      stderr: "pipe",
    });

    const exitCode = await proc.exited;
    return exitCode === 0;
  } catch {
    // If the binary doesn't exist, we're in development mode
    return true;
  }
}
