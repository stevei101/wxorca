import { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { ChatMessage } from "../services/api";

interface MessageBubbleProps {
  message: ChatMessage;
  agentIcon: string;
  onFeedback?: (rating: number) => void;
}

export default function MessageBubble({
  message,
  agentIcon,
  onFeedback,
}: MessageBubbleProps) {
  const [feedbackGiven, setFeedbackGiven] = useState<number | null>(null);

  const isUser = message.role === "user";

  function handleFeedback(rating: number) {
    setFeedbackGiven(rating);
    onFeedback?.(rating);
  }

  return (
    <div className={`flex gap-3 ${isUser ? "flex-row-reverse" : ""}`}>
      {/* Avatar */}
      <div
        className={`
          flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center text-sm
          ${isUser ? "bg-ibm-blue-60 text-white" : "bg-ibm-gray-20"}
        `}
      >
        {isUser ? "👤" : agentIcon}
      </div>

      {/* Message content */}
      <div
        className={`
          flex-1 max-w-[80%]
          ${isUser ? "text-right" : ""}
        `}
      >
        <div
          className={`
            inline-block rounded-lg px-4 py-3 text-left
            ${
              isUser
                ? "bg-ibm-blue-60 text-white"
                : "bg-ibm-gray-10 text-ibm-gray-100"
            }
          `}
        >
          {isUser ? (
            <p className="whitespace-pre-wrap">{message.content}</p>
          ) : (
            <div className="markdown-content prose prose-sm max-w-none">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {message.content}
              </ReactMarkdown>
            </div>
          )}
        </div>

        {/* Feedback buttons for assistant messages */}
        {!isUser && onFeedback && (
          <div className="mt-2 flex items-center gap-2">
            {feedbackGiven === null ? (
              <>
                <span className="text-xs text-ibm-gray-50">Was this helpful?</span>
                <button
                  onClick={() => handleFeedback(5)}
                  className="text-ibm-gray-40 hover:text-ibm-green-50 transition-colors"
                  title="Yes, this was helpful"
                >
                  👍
                </button>
                <button
                  onClick={() => handleFeedback(1)}
                  className="text-ibm-gray-40 hover:text-ibm-red-50 transition-colors"
                  title="No, this wasn't helpful"
                >
                  👎
                </button>
              </>
            ) : (
              <span className="text-xs text-ibm-gray-50">
                Thanks for your feedback!
              </span>
            )}
          </div>
        )}

        {/* Timestamp */}
        {message.timestamp && (
          <p className="text-xs text-ibm-gray-40 mt-1">
            {formatTimestamp(message.timestamp)}
          </p>
        )}
      </div>
    </div>
  );
}

function formatTimestamp(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return "";
  }
}
