//! Admin Setup Guide Agent
//!
//! Helps administrators set up and configure WatsonX Orchestrate.

use super::{route_by_tools, AnalyzeQueryNode, ExecuteToolsNode};
use crate::state::AgentType;
use oxidizedgraph::prelude::*;
use std::sync::Arc;

/// Agent for guiding administrators through WatsonX Orchestrate setup
pub struct AdminSetupAgent;

impl AdminSetupAgent {
    /// Build the agent graph for admin setup guidance
    pub fn build_graph(tool_registry: Arc<ToolRegistry>) -> Result<CompiledGraph, GraphError> {
        let system_prompt = AgentType::AdminSetup.system_prompt().to_string();

        GraphBuilder::new()
            .name("admin_setup_agent")
            .description("Guides administrators through WatsonX Orchestrate setup and configuration")
            // Analyze the user's query
            .add_node(AnalyzeQueryNode::new("analyze"))
            // Search documentation for relevant info
            .add_node(AdminSearchNode::new("search_docs", system_prompt.clone()))
            // Generate response with admin-specific guidance
            .add_node(AdminResponseNode::new("respond", system_prompt))
            // Execute any tool calls
            .add_node(ExecuteToolsNode::new("execute_tools", tool_registry))
            // Set entry point
            .set_entry_point("analyze")
            // Flow: analyze -> search_docs -> respond
            .add_edge("analyze", "search_docs")
            .add_edge("search_docs", "respond")
            // Conditional: if tools needed, execute them
            .add_conditional_edge("respond", route_by_tools)
            // After tools, loop back to respond
            .add_edge("execute_tools", "respond")
            .compile()
    }
}

/// Node that searches documentation with admin-focused context
struct AdminSearchNode {
    id: String,
    _system_prompt: String,
}

impl AdminSearchNode {
    fn new(id: impl Into<String>, system_prompt: String) -> Self {
        Self {
            id: id.into(),
            _system_prompt: system_prompt,
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for AdminSearchNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Searches documentation with admin-focused context")
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

        // Add a tool call to search admin documentation
        {
            let mut guard = state
                .write()
                .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;

            let tool_call = ToolCall {
                id: uuid::Uuid::new_v4().to_string(),
                name: "search_wxo_docs".to_string(),
                arguments: serde_json::json!({
                    "query": query,
                    "category": "admin",
                    "limit": 5
                }),
            };

            guard.tool_calls.push(tool_call);
        }

        Ok(NodeOutput::cont())
    }
}

/// Node that generates admin-focused responses
struct AdminResponseNode {
    id: String,
    system_prompt: String,
}

impl AdminResponseNode {
    fn new(id: impl Into<String>, system_prompt: String) -> Self {
        Self {
            id: id.into(),
            system_prompt,
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for AdminResponseNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Generates admin-focused responses")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let mut guard = state
            .write()
            .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;

        // Get the original query and any tool results
        let query = guard
            .get_context::<String>("original_query")
            .cloned()
            .unwrap_or_default();

        // Get tool results if any
        let tool_results: Vec<String> = guard
            .messages
            .iter()
            .filter(|m| m.role == MessageRole::Tool)
            .map(|m| m.content.clone())
            .collect();

        // Generate response (in a real implementation, this would call an LLM)
        let response = generate_admin_response(&query, &tool_results, &self.system_prompt);

        guard.add_assistant_message(&response);
        guard.mark_complete();

        Ok(NodeOutput::Finish)
    }
}

fn generate_admin_response(query: &str, tool_results: &[String], _system_prompt: &str) -> String {
    // In a real implementation, this would call an LLM
    // For now, generate a helpful template response

    let query_lower = query.to_lowercase();
    let has_docs = !tool_results.is_empty();

    let mut response = String::new();

    if query_lower.contains("setup") || query_lower.contains("install") {
        response.push_str("## WatsonX Orchestrate Setup Guide\n\n");
        response.push_str("Here's how to set up WatsonX Orchestrate:\n\n");
        response.push_str("1. **Access the Admin Console**: Navigate to your WXO instance and log in with admin credentials.\n\n");
        response.push_str("2. **Configure Identity Provider**: Set up SSO or local authentication under Settings > Security.\n\n");
        response.push_str("3. **Create User Groups**: Define roles and permissions in Settings > Users & Teams.\n\n");
        response.push_str("4. **Set Up Integrations**: Connect external services in Settings > Integrations.\n\n");
    } else if query_lower.contains("user") || query_lower.contains("permission") {
        response.push_str("## User Management\n\n");
        response.push_str("To manage users in WatsonX Orchestrate:\n\n");
        response.push_str("1. Go to **Settings > Users & Teams**\n");
        response.push_str("2. Click **Add User** to invite new users\n");
        response.push_str("3. Assign appropriate roles (Admin, Developer, User)\n");
        response.push_str("4. Configure team memberships for collaboration\n\n");
        response.push_str("**Tip**: Use groups to manage permissions at scale.\n");
    } else if query_lower.contains("security") || query_lower.contains("authentication") {
        response.push_str("## Security Configuration\n\n");
        response.push_str("Security best practices for WatsonX Orchestrate:\n\n");
        response.push_str("- Enable **Multi-Factor Authentication** (MFA) for all admin accounts\n");
        response.push_str("- Configure **Session Timeouts** appropriately\n");
        response.push_str("- Set up **Audit Logging** to track changes\n");
        response.push_str("- Review **API Key** permissions regularly\n");
        response.push_str("- Use **Least Privilege** principle for user roles\n");
    } else if query_lower.contains("integration") {
        response.push_str("## Integration Setup\n\n");
        response.push_str("To configure integrations:\n\n");
        response.push_str("1. Navigate to **Settings > Integrations**\n");
        response.push_str("2. Select the integration type (Salesforce, ServiceNow, etc.)\n");
        response.push_str("3. Provide the required credentials\n");
        response.push_str("4. Configure sync settings and permissions\n");
        response.push_str("5. Test the connection before enabling\n");
    } else {
        response.push_str("I'm here to help you with WatsonX Orchestrate administration.\n\n");
        response.push_str("I can assist with:\n");
        response.push_str("- Initial setup and configuration\n");
        response.push_str("- User and team management\n");
        response.push_str("- Security settings\n");
        response.push_str("- Integration configuration\n");
        response.push_str("- API key management\n\n");
        response.push_str("What would you like help with?");
    }

    if has_docs {
        response.push_str("\n\n---\n\n**📚 Related Documentation:**\n");
        response.push_str("I found some relevant documentation that might help. ");
        response.push_str("Check the search results above for more details.");
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::create_tool_registry;

    #[test]
    fn test_build_admin_graph() {
        let registry = Arc::new(create_tool_registry());
        let graph = AdminSetupAgent::build_graph(registry);
        assert!(graph.is_ok());
    }
}
