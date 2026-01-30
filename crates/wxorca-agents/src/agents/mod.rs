//! WXOrca Agents - Specialized AI agents for WatsonX Orchestrate guidance
//!
//! Each agent provides specialized assistance for different aspects of WXO:
//! - AdminSetupAgent: Configuration and administration
//! - UsageAssistantAgent: Feature usage and workflows
//! - TroubleshootAgent: Problem diagnosis and resolution
//! - BestPracticesAgent: Optimization and best practices
//! - DocsHelperAgent: Documentation navigation

mod admin_setup;
mod best_practices;
mod docs_helper;
mod troubleshoot;
mod usage_assistant;

pub use admin_setup::AdminSetupAgent;
pub use best_practices::BestPracticesAgent;
pub use docs_helper::DocsHelperAgent;
pub use troubleshoot::TroubleshootAgent;
pub use usage_assistant::UsageAssistantAgent;

use crate::state::{AgentType, WxorcaState};
use crate::tools::create_tool_registry;
use oxidizedgraph::prelude::*;
use std::sync::Arc;

/// Build the agent graph for the specified agent type
pub fn build_agent_graph(agent_type: AgentType) -> Result<CompiledGraph, GraphError> {
    let tool_registry = Arc::new(create_tool_registry());

    match agent_type {
        AgentType::AdminSetup => AdminSetupAgent::build_graph(tool_registry),
        AgentType::UsageAssistant => UsageAssistantAgent::build_graph(tool_registry),
        AgentType::Troubleshoot => TroubleshootAgent::build_graph(tool_registry),
        AgentType::BestPractices => BestPracticesAgent::build_graph(tool_registry),
        AgentType::DocsHelper => DocsHelperAgent::build_graph(tool_registry),
    }
}

/// Common node for analyzing user queries
pub struct AnalyzeQueryNode {
    id: String,
}

impl AnalyzeQueryNode {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for AnalyzeQueryNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Analyzes the user's query to extract intent and key information")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let mut guard = state
            .write()
            .map_err(|e| NodeError::Other(format!("Failed to acquire state lock: {}", e)))?;

        // Extract the last user message
        if let Some(last_msg) = guard.last_user_message() {
            let content = last_msg.content.clone();

            // Simple keyword-based intent detection
            let intent = detect_intent(&content);
            guard.set_context("user_intent", serde_json::json!(intent));
            guard.set_context("original_query", serde_json::json!(content));

            // Check if this needs tool usage
            let needs_tools = intent == "search" || intent == "validate" || intent == "example";
            guard.set_context("needs_tools", serde_json::json!(needs_tools));
        }

        Ok(NodeOutput::cont())
    }
}

fn detect_intent(query: &str) -> &'static str {
    let query_lower = query.to_lowercase();

    if query_lower.contains("how do i")
        || query_lower.contains("how to")
        || query_lower.contains("show me")
    {
        return "howto";
    }

    if query_lower.contains("error")
        || query_lower.contains("failed")
        || query_lower.contains("not working")
        || query_lower.contains("problem")
    {
        return "troubleshoot";
    }

    if query_lower.contains("documentation")
        || query_lower.contains("docs")
        || query_lower.contains("where can i find")
    {
        return "search";
    }

    if query_lower.contains("example")
        || query_lower.contains("sample")
        || query_lower.contains("show me code")
    {
        return "example";
    }

    if query_lower.contains("validate")
        || query_lower.contains("check")
        || query_lower.contains("is this correct")
    {
        return "validate";
    }

    if query_lower.contains("best practice")
        || query_lower.contains("recommend")
        || query_lower.contains("should i")
    {
        return "advice";
    }

    "general"
}

/// Common node for executing tools based on context
pub struct ExecuteToolsNode {
    id: String,
    tool_registry: Arc<ToolRegistry>,
}

impl ExecuteToolsNode {
    pub fn new(id: impl Into<String>, tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            id: id.into(),
            tool_registry,
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for ExecuteToolsNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Executes pending tool calls")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let pending_calls = {
            let guard = state
                .read()
                .map_err(|e| NodeError::Other(format!("Failed to read state: {}", e)))?;
            guard.tool_calls.clone()
        };

        for call in pending_calls {
            let result = self.tool_registry.execute(&call).await;

            let mut guard = state
                .write()
                .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;

            // ToolResult has content (success) or error fields
            guard.add_tool_result(&call.id, result.as_str());
        }

        // Clear tool calls after execution
        {
            let mut guard = state
                .write()
                .map_err(|e| NodeError::Other(format!("Failed to clear tool calls: {}", e)))?;
            guard.clear_tool_calls();
        }

        Ok(NodeOutput::cont())
    }
}

/// Router function for deciding whether to use tools or respond directly
pub fn route_by_tools(state: &AgentState) -> String {
    if state.has_pending_tool_calls() {
        "execute_tools".to_string()
    } else {
        transitions::END.to_string()
    }
}

/// Router function based on user intent
pub fn route_by_intent(state: &AgentState) -> String {
    if let Some(needs_tools) = state.get_context::<bool>("needs_tools") {
        if *needs_tools {
            return "search_docs".to_string();
        }
    }
    "respond".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_intent() {
        assert_eq!(detect_intent("How do I create a skill?"), "howto");
        assert_eq!(detect_intent("I'm getting an error"), "troubleshoot");
        assert_eq!(detect_intent("Show me an example"), "example");
        assert_eq!(detect_intent("Is this config correct?"), "validate");
        assert_eq!(detect_intent("What's the best practice for this?"), "advice");
    }

    #[test]
    fn test_build_agent_graphs() {
        // Test that all agent graphs can be built
        for agent_type in AgentType::all() {
            let result = build_agent_graph(*agent_type);
            assert!(
                result.is_ok(),
                "Failed to build graph for {:?}: {:?}",
                agent_type,
                result.err()
            );
        }
    }
}
