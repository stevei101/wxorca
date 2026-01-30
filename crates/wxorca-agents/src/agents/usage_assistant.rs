//! Usage Assistant Agent
//!
//! Helps users understand and effectively use WatsonX Orchestrate features.

use super::{route_by_tools, AnalyzeQueryNode, ExecuteToolsNode};
use crate::state::AgentType;
use oxidizedgraph::prelude::*;
use std::sync::Arc;

/// Agent for helping users with WatsonX Orchestrate features
pub struct UsageAssistantAgent;

impl UsageAssistantAgent {
    /// Build the agent graph for usage assistance
    pub fn build_graph(tool_registry: Arc<ToolRegistry>) -> Result<CompiledGraph, GraphError> {
        let system_prompt = AgentType::UsageAssistant.system_prompt().to_string();

        GraphBuilder::new()
            .name("usage_assistant_agent")
            .description("Helps users understand and use WatsonX Orchestrate features")
            .add_node(AnalyzeQueryNode::new("analyze"))
            .add_node(UsageSearchNode::new("search_docs", system_prompt.clone()))
            .add_node(ExampleFetchNode::new("fetch_examples"))
            .add_node(UsageResponseNode::new("respond", system_prompt))
            .add_node(ExecuteToolsNode::new("execute_tools", tool_registry))
            .set_entry_point("analyze")
            // Analyze -> conditional routing based on intent
            .add_conditional_edge("analyze", |state| {
                if let Some(intent) = state.get_context::<String>("user_intent") {
                    if intent == "example" {
                        return "fetch_examples".to_string();
                    }
                }
                "search_docs".to_string()
            })
            .add_edge("search_docs", "respond")
            .add_edge("fetch_examples", "respond")
            .add_conditional_edge("respond", route_by_tools)
            .add_edge("execute_tools", "respond")
            .compile()
    }
}

struct UsageSearchNode {
    id: String,
    _system_prompt: String,
}

impl UsageSearchNode {
    fn new(id: impl Into<String>, system_prompt: String) -> Self {
        Self {
            id: id.into(),
            _system_prompt: system_prompt,
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for UsageSearchNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Searches documentation for user-focused content")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let query = {
            let guard = state
                .read()
                .map_err(|e| NodeError::Other(format!("Failed to read state: {}", e)))?;
            guard
                .get_context::<String>("original_query")
                .cloned()
                .unwrap_or_default()
        };

        if query.is_empty() {
            return Ok(NodeOutput::cont());
        }

        {
            let mut guard = state
                .write()
                .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;

            let tool_call = ToolCall {
                id: uuid::Uuid::new_v4().to_string(),
                name: "search_wxo_docs".to_string(),
                arguments: serde_json::json!({
                    "query": query,
                    "category": "user",
                    "limit": 5
                }),
            };

            guard.tool_calls.push(tool_call);
        }

        Ok(NodeOutput::cont())
    }
}

struct ExampleFetchNode {
    id: String,
}

impl ExampleFetchNode {
    fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for ExampleFetchNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Fetches code examples for the user's query")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let query = {
            let guard = state
                .read()
                .map_err(|e| NodeError::Other(format!("Failed to read state: {}", e)))?;
            guard
                .get_context::<String>("original_query")
                .cloned()
                .unwrap_or_default()
        };

        if query.is_empty() {
            return Ok(NodeOutput::cont());
        }

        {
            let mut guard = state
                .write()
                .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;

            let tool_call = ToolCall {
                id: uuid::Uuid::new_v4().to_string(),
                name: "fetch_wxo_examples".to_string(),
                arguments: serde_json::json!({
                    "topic": query,
                    "limit": 3
                }),
            };

            guard.tool_calls.push(tool_call);
        }

        Ok(NodeOutput::cont())
    }
}

struct UsageResponseNode {
    id: String,
    system_prompt: String,
}

impl UsageResponseNode {
    fn new(id: impl Into<String>, system_prompt: String) -> Self {
        Self {
            id: id.into(),
            system_prompt,
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for UsageResponseNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Generates user-friendly responses about WXO features")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let mut guard = state
            .write()
            .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;

        let query = guard
            .get_context::<String>("original_query")
            .cloned()
            .unwrap_or_default();

        let tool_results: Vec<String> = guard
            .messages
            .iter()
            .filter(|m| m.role == MessageRole::Tool)
            .map(|m| m.content.clone())
            .collect();

        let response = generate_usage_response(&query, &tool_results, &self.system_prompt);

        guard.add_assistant_message(&response);
        guard.mark_complete();

        Ok(NodeOutput::Finish)
    }
}

fn generate_usage_response(query: &str, tool_results: &[String], _system_prompt: &str) -> String {
    let query_lower = query.to_lowercase();
    let mut response = String::new();

    if query_lower.contains("skill") {
        response.push_str("## Working with Skills\n\n");
        response.push_str("Skills are the building blocks of WatsonX Orchestrate. Here's how to work with them:\n\n");
        response.push_str("### Creating a Skill\n");
        response.push_str("1. Click **+ New Skill** in the skill catalog\n");
        response.push_str("2. Choose a skill type (API, Custom, Pre-built)\n");
        response.push_str("3. Define inputs and outputs\n");
        response.push_str("4. Test your skill before publishing\n\n");
        response.push_str("### Using Skills\n");
        response.push_str("- Type naturally: \"Send an email to John about the meeting\"\n");
        response.push_str("- WXO will find and execute the right skill\n");
        response.push_str("- Review and confirm before execution\n");
    } else if query_lower.contains("workflow") || query_lower.contains("automation") {
        response.push_str("## Building Workflows\n\n");
        response.push_str("Workflows let you chain skills together for complex automations:\n\n");
        response.push_str("1. **Design**: Map out the steps in your process\n");
        response.push_str("2. **Build**: Add skills to your workflow canvas\n");
        response.push_str("3. **Connect**: Define data flow between steps\n");
        response.push_str("4. **Test**: Run the workflow with test data\n");
        response.push_str("5. **Deploy**: Publish for your team to use\n\n");
        response.push_str("**💡 Pro Tip**: Start simple and add complexity gradually.");
    } else if query_lower.contains("catalog") {
        response.push_str("## Skill Catalog\n\n");
        response.push_str("The catalog contains all available skills:\n\n");
        response.push_str("- **Pre-built Skills**: Ready-to-use integrations (Salesforce, Slack, etc.)\n");
        response.push_str("- **Custom Skills**: Created by your organization\n");
        response.push_str("- **Personal Skills**: Your private skills\n\n");
        response.push_str("Browse by category or search by keyword to find what you need.");
    } else if query_lower.contains("ai") || query_lower.contains("assistant") {
        response.push_str("## AI Assistant Features\n\n");
        response.push_str("WatsonX Orchestrate's AI understands natural language:\n\n");
        response.push_str("- **Ask Questions**: \"What can you do?\"\n");
        response.push_str("- **Execute Tasks**: \"Create a new support ticket\"\n");
        response.push_str("- **Get Help**: \"How do I use the Salesforce integration?\"\n\n");
        response.push_str("The AI learns from your usage patterns to provide better suggestions over time.");
    } else {
        response.push_str("## Getting Started with WatsonX Orchestrate\n\n");
        response.push_str("Welcome! I can help you with:\n\n");
        response.push_str("- **Skills**: Creating and using automation skills\n");
        response.push_str("- **Workflows**: Building multi-step automations\n");
        response.push_str("- **Catalog**: Finding pre-built integrations\n");
        response.push_str("- **AI Features**: Natural language interaction\n\n");
        response.push_str("What would you like to learn about?");
    }

    if !tool_results.is_empty() {
        response.push_str("\n\n---\n\n**📋 Additional Resources:**\n");
        response.push_str("I found some relevant information. Check the details above.");
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::create_tool_registry;

    #[test]
    fn test_build_usage_graph() {
        let registry = Arc::new(create_tool_registry());
        let graph = UsageAssistantAgent::build_graph(registry);
        assert!(graph.is_ok());
    }
}
