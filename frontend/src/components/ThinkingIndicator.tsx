interface ThinkingIndicatorProps {
  agentIcon: string;
}

export default function ThinkingIndicator({
  agentIcon,
}: ThinkingIndicatorProps) {
  return (
    <div className="flex gap-3">
      {/* Avatar */}
      <div className="flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center text-sm bg-ibm-gray-20">
        {agentIcon}
      </div>

      {/* Thinking animation */}
      <div className="bg-ibm-gray-10 rounded-lg px-4 py-3">
        <div className="flex items-center gap-2">
          <div className="flex gap-1">
            <span
              className="w-2 h-2 bg-ibm-blue-60 rounded-full animate-bounce"
              style={{ animationDelay: "0ms" }}
            />
            <span
              className="w-2 h-2 bg-ibm-blue-60 rounded-full animate-bounce"
              style={{ animationDelay: "150ms" }}
            />
            <span
              className="w-2 h-2 bg-ibm-blue-60 rounded-full animate-bounce"
              style={{ animationDelay: "300ms" }}
            />
          </div>
          <span className="text-sm text-ibm-gray-60">Thinking...</span>
        </div>
      </div>
    </div>
  );
}
