import { useState, useRef, useEffect } from "react";
import { AgentType, ChatMessage, sendMessage, submitFeedback } from "../services/api";
import MessageBubble from "./MessageBubble";
import ThinkingIndicator from "./ThinkingIndicator";

interface AgentChatProps {
  agent: AgentType;
  sessionId: string;
  onNewSession: () => void;
}

export default function AgentChat({
  agent,
  sessionId,
  onNewSession,
}: AgentChatProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Reset messages when agent changes
  useEffect(() => {
    setMessages([]);
    setError(null);
    inputRef.current?.focus();
  }, [agent.id, sessionId]);

  // Scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Auto-resize textarea
  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.style.height = "auto";
      inputRef.current.style.height = `${Math.min(inputRef.current.scrollHeight, 200)}px`;
    }
  }, [input]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const userMessage = input.trim();
    setInput("");
    setError(null);

    // Add user message immediately
    const newUserMessage: ChatMessage = {
      role: "user",
      content: userMessage,
      timestamp: new Date().toISOString(),
    };
    setMessages((prev) => [...prev, newUserMessage]);

    setIsLoading(true);

    try {
      const response = await sendMessage(sessionId, agent.id, userMessage);

      const assistantMessage: ChatMessage = {
        role: "assistant",
        content: response.message,
        timestamp: response.timestamp,
      };
      setMessages((prev) => [...prev, assistantMessage]);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to get response");
      // Remove the user message if we couldn't get a response
      setMessages((prev) => prev.slice(0, -1));
    } finally {
      setIsLoading(false);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  }

  async function handleFeedback(messageIndex: number, rating: number) {
    try {
      await submitFeedback({
        sessionId,
        messageId: `msg_${messageIndex}`,
        rating,
      });
    } catch (err) {
      console.error("Failed to submit feedback:", err);
    }
  }

  function handleClearChat() {
    setMessages([]);
    onNewSession();
  }

  return (
    <div className="flex-1 bg-white rounded-lg shadow-sm flex flex-col min-h-[500px]">
      {/* Chat header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-ibm-gray-20">
        <div className="flex items-center gap-3">
          <span className="text-2xl">{agent.icon}</span>
          <div>
            <h2 className="font-semibold text-ibm-gray-100">{agent.name}</h2>
            <p className="text-sm text-ibm-gray-60 max-w-md truncate">
              {agent.description}
            </p>
          </div>
        </div>
        <button
          onClick={handleClearChat}
          className="text-sm text-ibm-gray-60 hover:text-ibm-gray-100 px-3 py-1.5 rounded hover:bg-ibm-gray-10 transition-colors"
        >
          Clear chat
        </button>
      </div>

      {/* Messages area */}
      <div className="flex-1 overflow-y-auto p-6 space-y-4">
        {messages.length === 0 && !isLoading && (
          <div className="text-center py-12">
            <span className="text-4xl mb-4 block">{agent.icon}</span>
            <h3 className="text-lg font-medium text-ibm-gray-100 mb-2">
              Start a conversation
            </h3>
            <p className="text-ibm-gray-60 max-w-md mx-auto">
              {agent.description}
            </p>
            <div className="mt-6 flex flex-wrap gap-2 justify-center">
              {getSuggestedQuestions(agent.id).map((question, i) => (
                <button
                  key={i}
                  onClick={() => setInput(question)}
                  className="text-sm px-3 py-1.5 bg-ibm-gray-10 hover:bg-ibm-gray-20 rounded-full text-ibm-gray-70 transition-colors"
                >
                  {question}
                </button>
              ))}
            </div>
          </div>
        )}

        {messages.map((message, index) => (
          <MessageBubble
            key={index}
            message={message}
            agentIcon={agent.icon}
            onFeedback={
              message.role === "assistant"
                ? (rating) => handleFeedback(index, rating)
                : undefined
            }
          />
        ))}

        {isLoading && <ThinkingIndicator agentIcon={agent.icon} />}

        {error && (
          <div className="bg-ibm-red-50/10 border border-ibm-red-50 text-ibm-red-60 px-4 py-3 rounded-lg">
            <p className="font-medium">Error</p>
            <p className="text-sm">{error}</p>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input area */}
      <form
        onSubmit={handleSubmit}
        className="border-t border-ibm-gray-20 p-4"
      >
        <div className="flex gap-3">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={`Ask ${agent.name}...`}
            className="flex-1 resize-none border border-ibm-gray-30 rounded-lg px-4 py-3 focus:outline-none focus:ring-2 focus:ring-ibm-blue-60 focus:border-transparent min-h-[48px] max-h-[200px]"
            rows={1}
            disabled={isLoading}
          />
          <button
            type="submit"
            disabled={!input.trim() || isLoading}
            className="px-6 py-3 bg-ibm-blue-60 text-white rounded-lg hover:bg-ibm-blue-70 disabled:bg-ibm-gray-30 disabled:cursor-not-allowed transition-colors font-medium"
          >
            Send
          </button>
        </div>
        <p className="text-xs text-ibm-gray-50 mt-2">
          Press Enter to send, Shift+Enter for new line
        </p>
      </form>
    </div>
  );
}

function getSuggestedQuestions(agentId: string): string[] {
  const suggestions: Record<string, string[]> = {
    "admin-setup": [
      "How do I set up WXO?",
      "Configure SSO",
      "Manage user permissions",
    ],
    usage: [
      "How do I create a skill?",
      "Build a workflow",
      "Use the catalog",
    ],
    troubleshoot: [
      "I can't log in",
      "Skill is failing",
      "Integration not working",
    ],
    "best-practices": [
      "Workflow design tips",
      "Security best practices",
      "Performance optimization",
    ],
    docs: [
      "Getting started guide",
      "API documentation",
      "Find integration docs",
    ],
  };

  return suggestions[agentId] || ["How can you help me?"];
}
