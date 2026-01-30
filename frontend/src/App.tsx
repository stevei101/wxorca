import { useState, useEffect } from "react";
import AgentSelector from "./components/AgentSelector";
import AgentChat from "./components/AgentChat";
import { fetchAgentTypes, AgentType } from "./services/api";

function App() {
  const [agentTypes, setAgentTypes] = useState<AgentType[]>([]);
  const [selectedAgent, setSelectedAgent] = useState<AgentType | null>(null);
  const [sessionId, setSessionId] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadAgentTypes();
    // Generate a unique session ID
    setSessionId(generateSessionId());
  }, []);

  async function loadAgentTypes() {
    try {
      const types = await fetchAgentTypes();
      setAgentTypes(types);
      if (types.length > 0) {
        setSelectedAgent(types[0]);
      }
    } catch (err) {
      setError("Failed to load agent types. Is the backend running?");
      console.error(err);
    } finally {
      setLoading(false);
    }
  }

  function handleAgentChange(agent: AgentType) {
    setSelectedAgent(agent);
    // Generate new session for new agent
    setSessionId(generateSessionId());
  }

  function generateSessionId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-ibm-gray-10 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin w-12 h-12 border-4 border-ibm-blue-60 border-t-transparent rounded-full mx-auto mb-4"></div>
          <p className="text-ibm-gray-70">Loading WXOrca...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-ibm-gray-10 flex items-center justify-center">
        <div className="bg-white p-8 rounded-lg shadow-lg max-w-md text-center">
          <div className="text-ibm-red-60 text-4xl mb-4">⚠️</div>
          <h2 className="text-xl font-semibold text-ibm-gray-100 mb-2">
            Connection Error
          </h2>
          <p className="text-ibm-gray-70 mb-4">{error}</p>
          <button
            onClick={() => {
              setError(null);
              setLoading(true);
              loadAgentTypes();
            }}
            className="px-4 py-2 bg-ibm-blue-60 text-white rounded hover:bg-ibm-blue-70 transition-colors"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-ibm-gray-10 flex flex-col">
      {/* Header */}
      <header className="bg-ibm-gray-100 text-white py-4 px-6">
        <div className="max-w-6xl mx-auto flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-2xl">🐳</span>
            <div>
              <h1 className="text-xl font-semibold">WXOrca</h1>
              <p className="text-ibm-gray-30 text-sm">
                AI-powered guide for IBM WatsonX Orchestrate
              </p>
            </div>
          </div>
          <div className="flex items-center gap-4">
            <a
              href="https://www.ibm.com/products/watsonx-orchestrate"
              target="_blank"
              rel="noopener noreferrer"
              className="text-ibm-blue-40 hover:text-ibm-blue-30 text-sm"
            >
              WatsonX Orchestrate →
            </a>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="flex-1 flex flex-col max-w-6xl mx-auto w-full p-6 gap-6">
        {/* Agent selector */}
        <AgentSelector
          agents={agentTypes}
          selectedAgent={selectedAgent}
          onSelect={handleAgentChange}
        />

        {/* Chat interface */}
        {selectedAgent && (
          <AgentChat
            agent={selectedAgent}
            sessionId={sessionId}
            onNewSession={() => setSessionId(generateSessionId())}
          />
        )}
      </main>

      {/* Footer */}
      <footer className="bg-ibm-gray-100 text-ibm-gray-40 py-4 px-6 text-center text-sm">
        <p>
          Built with{" "}
          <a
            href="https://github.com/oxidizedgraph"
            className="text-ibm-blue-40 hover:underline"
          >
            oxidizedgraph
          </a>{" "}
          + SurrealDB + React
        </p>
      </footer>
    </div>
  );
}

export default App;
