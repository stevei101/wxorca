import { Elysia, t } from "elysia";
import { invokeRustAgent, AgentResponse } from "../services/rust-bridge";

// Agent types available
const AGENT_TYPES = [
  {
    id: "admin-setup",
    name: "Admin Setup Guide",
    description:
      "Guides administrators through WatsonX Orchestrate setup and configuration. Helps with user management, security settings, and integrations.",
    icon: "⚙️",
  },
  {
    id: "usage",
    name: "Usage Assistant",
    description:
      "Helps you understand how to use WatsonX Orchestrate effectively. Ask about creating skills, building automations, or using the catalog.",
    icon: "💡",
  },
  {
    id: "troubleshoot",
    name: "Troubleshooting Bot",
    description:
      "Diagnoses and resolves issues with WatsonX Orchestrate. Describe your problem and I'll help you find a solution.",
    icon: "🔧",
  },
  {
    id: "best-practices",
    name: "Best Practices Coach",
    description:
      "Provides optimization tips and best practices. I can help you design better workflows and improve performance.",
    icon: "🏆",
  },
  {
    id: "docs",
    name: "Documentation Helper",
    description:
      "Navigates and explains WatsonX Orchestrate documentation. Ask me about any feature and I'll find the relevant docs.",
    icon: "📚",
  },
] as const;

// Conversation history storage (in-memory for now)
const conversations = new Map<
  string,
  {
    agentType: string;
    messages: Array<{ role: "user" | "assistant"; content: string }>;
    createdAt: Date;
    updatedAt: Date;
  }
>();

export const agentRoutes = new Elysia({ prefix: "/api/agents" })
  // List all agent types
  .get(
    "/types",
    () => AGENT_TYPES,
    {
      detail: {
        tags: ["Agents"],
        summary: "List agent types",
        description: "Get all available agent types with their descriptions",
      },
    }
  )

  // Get a specific agent type
  .get(
    "/types/:id",
    ({ params, error }) => {
      const agent = AGENT_TYPES.find((a) => a.id === params.id);
      if (!agent) {
        return error(404, { message: "Agent type not found" });
      }
      return agent;
    },
    {
      params: t.Object({
        id: t.String(),
      }),
      detail: {
        tags: ["Agents"],
        summary: "Get agent type",
        description: "Get details about a specific agent type",
      },
    }
  )

  // Chat with an agent
  .post(
    "/chat",
    async ({ body, error }) => {
      const { sessionId, agentType, message } = body;

      // Validate agent type
      const validAgentTypes = AGENT_TYPES.map((a) => a.id);
      if (!validAgentTypes.includes(agentType as any)) {
        return error(400, { message: "Invalid agent type" });
      }

      // Get or create conversation
      let conversation = conversations.get(sessionId);
      if (!conversation) {
        conversation = {
          agentType,
          messages: [],
          createdAt: new Date(),
          updatedAt: new Date(),
        };
        conversations.set(sessionId, conversation);
      }

      // Add user message
      conversation.messages.push({ role: "user", content: message });
      conversation.updatedAt = new Date();

      try {
        // Call the Rust agent
        const response = await invokeRustAgent(agentType, sessionId, message);

        if (response.error) {
          return error(500, { message: response.error });
        }

        // Add assistant message
        conversation.messages.push({
          role: "assistant",
          content: response.response,
        });

        return {
          sessionId: response.session_id,
          agentType: response.agent_type,
          message: response.response,
          timestamp: new Date().toISOString(),
        };
      } catch (err) {
        console.error("Agent invocation failed:", err);
        return error(500, {
          message: "Failed to invoke agent",
          details: err instanceof Error ? err.message : String(err),
        });
      }
    },
    {
      body: t.Object({
        sessionId: t.String({ minLength: 1 }),
        agentType: t.String({ minLength: 1 }),
        message: t.String({ minLength: 1 }),
      }),
      detail: {
        tags: ["Agents"],
        summary: "Chat with agent",
        description: "Send a message to an agent and get a response",
      },
    }
  )

  // Get conversation history
  .get(
    "/conversations/:sessionId",
    ({ params, error }) => {
      const conversation = conversations.get(params.sessionId);
      if (!conversation) {
        return error(404, { message: "Conversation not found" });
      }

      return {
        sessionId: params.sessionId,
        agentType: conversation.agentType,
        messages: conversation.messages,
        createdAt: conversation.createdAt.toISOString(),
        updatedAt: conversation.updatedAt.toISOString(),
      };
    },
    {
      params: t.Object({
        sessionId: t.String(),
      }),
      detail: {
        tags: ["Agents"],
        summary: "Get conversation",
        description: "Get the conversation history for a session",
      },
    }
  )

  // Delete conversation
  .delete(
    "/conversations/:sessionId",
    ({ params }) => {
      const deleted = conversations.delete(params.sessionId);
      return {
        success: deleted,
        message: deleted ? "Conversation deleted" : "Conversation not found",
      };
    },
    {
      params: t.Object({
        sessionId: t.String(),
      }),
      detail: {
        tags: ["Agents"],
        summary: "Delete conversation",
        description: "Delete a conversation and its history",
      },
    }
  )

  // Submit feedback
  .post(
    "/feedback",
    ({ body }) => {
      // In a real implementation, this would store feedback in SurrealDB
      console.log("Feedback received:", body);

      return {
        success: true,
        message: "Thank you for your feedback!",
      };
    },
    {
      body: t.Object({
        sessionId: t.String(),
        messageId: t.Optional(t.String()),
        rating: t.Number({ minimum: 1, maximum: 5 }),
        comment: t.Optional(t.String()),
      }),
      detail: {
        tags: ["Agents"],
        summary: "Submit feedback",
        description: "Submit feedback about an agent interaction",
      },
    }
  );
