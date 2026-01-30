import { AgentType } from "../services/api";

interface AgentSelectorProps {
  agents: AgentType[];
  selectedAgent: AgentType | null;
  onSelect: (agent: AgentType) => void;
}

export default function AgentSelector({
  agents,
  selectedAgent,
  onSelect,
}: AgentSelectorProps) {
  return (
    <div className="bg-white rounded-lg shadow-sm p-4">
      <h2 className="text-sm font-medium text-ibm-gray-70 mb-3">
        Select an Agent
      </h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3">
        {agents.map((agent) => (
          <button
            key={agent.id}
            onClick={() => onSelect(agent)}
            className={`
              p-4 rounded-lg border-2 text-left transition-all
              ${
                selectedAgent?.id === agent.id
                  ? "border-ibm-blue-60 bg-ibm-blue-10"
                  : "border-ibm-gray-20 hover:border-ibm-gray-40 bg-white"
              }
            `}
          >
            <div className="text-2xl mb-2">{agent.icon}</div>
            <h3
              className={`font-medium text-sm ${
                selectedAgent?.id === agent.id
                  ? "text-ibm-blue-60"
                  : "text-ibm-gray-100"
              }`}
            >
              {agent.name}
            </h3>
            <p className="text-xs text-ibm-gray-60 mt-1 line-clamp-2">
              {agent.description}
            </p>
          </button>
        ))}
      </div>
    </div>
  );
}
