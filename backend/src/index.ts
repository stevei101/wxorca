import { Elysia } from "elysia";
import { cors } from "@elysiajs/cors";
import { swagger } from "@elysiajs/swagger";
import { agentRoutes } from "./routes/agents";
import { healthRoutes } from "./routes/health";

const app = new Elysia()
  .use(
    cors({
      origin: ["http://localhost:5173", "http://localhost:3000"],
      methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"],
      allowedHeaders: ["Content-Type", "Authorization"],
    })
  )
  .use(
    swagger({
      documentation: {
        info: {
          title: "WXOrca API",
          version: "0.1.0",
          description:
            "API for WXOrca - AI-powered guide for IBM WatsonX Orchestrate",
        },
        tags: [
          { name: "Agents", description: "Agent interaction endpoints" },
          { name: "Health", description: "Health check endpoints" },
        ],
      },
    })
  )
  .use(healthRoutes)
  .use(agentRoutes)
  .listen(3000);

console.log(
  `🐳 WXOrca API is running at ${app.server?.hostname}:${app.server?.port}`
);
console.log(`📚 Swagger docs available at http://localhost:3000/swagger`);

export type App = typeof app;
