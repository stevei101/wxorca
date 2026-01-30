const API_BASE = "/api";

export interface AgentType {
  id: string;
  name: string;
  description: string;
  icon: string;
}

export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  timestamp?: string;
}

export interface ChatResponse {
  sessionId: string;
  agentType: string;
  message: string;
  timestamp: string;
}

export interface Conversation {
  sessionId: string;
  agentType: string;
  messages: ChatMessage[];
  createdAt: string;
  updatedAt: string;
}

export interface FeedbackPayload {
  sessionId: string;
  messageId?: string;
  rating: number;
  comment?: string;
}

/**
 * Fetch all available agent types
 */
export async function fetchAgentTypes(): Promise<AgentType[]> {
  const response = await fetch(`${API_BASE}/agents/types`);
  if (!response.ok) {
    throw new Error(`Failed to fetch agent types: ${response.statusText}`);
  }
  return response.json();
}

/**
 * Send a message to an agent and get a response
 */
export async function sendMessage(
  sessionId: string,
  agentType: string,
  message: string
): Promise<ChatResponse> {
  const response = await fetch(`${API_BASE}/agents/chat`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      sessionId,
      agentType,
      message,
    }),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({}));
    throw new Error(error.message || `Failed to send message: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Get conversation history
 */
export async function getConversation(sessionId: string): Promise<Conversation | null> {
  const response = await fetch(`${API_BASE}/agents/conversations/${sessionId}`);

  if (response.status === 404) {
    return null;
  }

  if (!response.ok) {
    throw new Error(`Failed to get conversation: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Delete a conversation
 */
export async function deleteConversation(sessionId: string): Promise<void> {
  const response = await fetch(`${API_BASE}/agents/conversations/${sessionId}`, {
    method: "DELETE",
  });

  if (!response.ok) {
    throw new Error(`Failed to delete conversation: ${response.statusText}`);
  }
}

/**
 * Submit feedback
 */
export async function submitFeedback(feedback: FeedbackPayload): Promise<void> {
  const response = await fetch(`${API_BASE}/agents/feedback`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(feedback),
  });

  if (!response.ok) {
    throw new Error(`Failed to submit feedback: ${response.statusText}`);
  }
}

/**
 * Check API health
 */
export async function checkHealth(): Promise<{ status: string; timestamp: string }> {
  const response = await fetch(`${API_BASE}/health`);
  if (!response.ok) {
    throw new Error(`Health check failed: ${response.statusText}`);
  }
  return response.json();
}
