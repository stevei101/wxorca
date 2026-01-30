//! Documentation Helper Agent
//!
//! Helps users navigate and understand WatsonX Orchestrate documentation.

use super::{route_by_tools, AnalyzeQueryNode, ExecuteToolsNode};
use crate::state::AgentType;
use oxidizedgraph::prelude::*;
use std::sync::Arc;

/// Agent for helping users with documentation
pub struct DocsHelperAgent;

impl DocsHelperAgent {
    /// Build the agent graph for documentation help
    pub fn build_graph(tool_registry: Arc<ToolRegistry>) -> Result<CompiledGraph, GraphError> {
        let system_prompt = AgentType::DocsHelper.system_prompt().to_string();

        GraphBuilder::new()
            .name("docs_helper_agent")
            .description("Helps users navigate and understand WatsonX Orchestrate documentation")
            .add_node(AnalyzeQueryNode::new("analyze"))
            .add_node(DocsCategoryNode::new("categorize"))
            .add_node(DocsSearchNode::new("search_docs", system_prompt.clone()))
            .add_node(DocsResponseNode::new("respond", system_prompt))
            .add_node(ExecuteToolsNode::new("execute_tools", tool_registry))
            .set_entry_point("analyze")
            .add_edge("analyze", "categorize")
            .add_edge("categorize", "search_docs")
            .add_edge("search_docs", "respond")
            .add_conditional_edge("respond", route_by_tools)
            .add_edge("execute_tools", "respond")
            .compile()
    }
}

struct DocsCategoryNode {
    id: String,
}

impl DocsCategoryNode {
    fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for DocsCategoryNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Categorizes the documentation request")
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

        let category = categorize_docs_request(&query);

        {
            let mut guard = state
                .write()
                .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;
            guard.set_context("docs_category", serde_json::json!(category));
        }

        Ok(NodeOutput::cont())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct DocsCategory {
    primary: String,
    secondary: Option<String>,
    keywords: Vec<String>,
}

fn categorize_docs_request(query: &str) -> DocsCategory {
    let query_lower = query.to_lowercase();

    let (primary, secondary) = if query_lower.contains("api") || query_lower.contains("endpoint") {
        ("api", Some("reference"))
    } else if query_lower.contains("admin") || query_lower.contains("configure") {
        ("admin", Some("setup"))
    } else if query_lower.contains("start") || query_lower.contains("begin") {
        ("getting_started", None)
    } else if query_lower.contains("skill") {
        ("user", Some("skills"))
    } else if query_lower.contains("workflow") {
        ("user", Some("workflows"))
    } else if query_lower.contains("integration") {
        ("admin", Some("integrations"))
    } else if query_lower.contains("error") || query_lower.contains("troubleshoot") {
        ("troubleshooting", None)
    } else if query_lower.contains("release") || query_lower.contains("new") {
        ("release_notes", None)
    } else {
        ("user", None)
    };

    let keywords: Vec<String> = query_lower
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .map(|w| w.to_string())
        .take(5)
        .collect();

    DocsCategory {
        primary: primary.to_string(),
        secondary: secondary.map(|s| s.to_string()),
        keywords,
    }
}

struct DocsSearchNode {
    id: String,
    _system_prompt: String,
}

impl DocsSearchNode {
    fn new(id: impl Into<String>, system_prompt: String) -> Self {
        Self {
            id: id.into(),
            _system_prompt: system_prompt,
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for DocsSearchNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Searches documentation based on categorized request")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let (query, category) = {
            let guard = state
                .read()
                .map_err(|e| NodeError::Other(format!("Failed to read state: {}", e)))?;

            let query = guard
                .get_context::<String>("original_query")
                .cloned()
                .unwrap_or_default();

            let category = guard
                .get_context::<DocsCategory>("docs_category")
                .cloned()
                .unwrap_or_else(|| DocsCategory {
                    primary: "user".to_string(),
                    secondary: None,
                    keywords: vec![],
                });

            (query, category)
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
                    "category": category.primary,
                    "limit": 5
                }),
            };

            guard.tool_calls.push(tool_call);
        }

        Ok(NodeOutput::cont())
    }
}

struct DocsResponseNode {
    id: String,
    system_prompt: String,
}

impl DocsResponseNode {
    fn new(id: impl Into<String>, system_prompt: String) -> Self {
        Self {
            id: id.into(),
            system_prompt,
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for DocsResponseNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Generates documentation navigation response")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let mut guard = state
            .write()
            .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;

        let query = guard
            .get_context::<String>("original_query")
            .cloned()
            .unwrap_or_default();

        let category = guard
            .get_context::<DocsCategory>("docs_category")
            .cloned()
            .unwrap_or_else(|| DocsCategory {
                primary: "user".to_string(),
                secondary: None,
                keywords: vec![],
            });

        let tool_results: Vec<String> = guard
            .messages
            .iter()
            .filter(|m| m.role == MessageRole::Tool)
            .map(|m| m.content.clone())
            .collect();

        let response =
            generate_docs_response(&query, &category, &tool_results, &self.system_prompt);

        guard.add_assistant_message(&response);
        guard.mark_complete();

        Ok(NodeOutput::Finish)
    }
}

fn generate_docs_response(
    _query: &str,
    category: &DocsCategory,
    tool_results: &[String],
    _system_prompt: &str,
) -> String {
    let mut response = String::new();

    response.push_str("## 📚 Documentation Guide\n\n");

    // Add category-specific documentation overview
    match category.primary.as_str() {
        "api" => {
            response.push_str("### API Documentation\n\n");
            response.push_str("The WatsonX Orchestrate API documentation covers:\n\n");
            response.push_str("- **Authentication**: How to obtain and use API tokens\n");
            response.push_str("- **Skills API**: Create, manage, and execute skills\n");
            response.push_str("- **Workflows API**: Manage workflow definitions\n");
            response.push_str("- **Users API**: User and team management\n\n");
            response.push_str("**Quick Links:**\n");
            response.push_str("- [API Reference](https://www.ibm.com/docs/watsonx-orchestrate/api)\n");
            response.push_str("- [Authentication Guide](https://www.ibm.com/docs/watsonx-orchestrate/api/auth)\n");
        }
        "admin" => {
            response.push_str("### Administration Documentation\n\n");
            response.push_str("Admin documentation helps you:\n\n");
            response.push_str("- **Set up** your WXO environment\n");
            response.push_str("- **Configure** security and access control\n");
            response.push_str("- **Manage** users, teams, and permissions\n");
            response.push_str("- **Integrate** with external services\n\n");
            response.push_str("**Quick Links:**\n");
            response.push_str("- [Admin Guide](https://www.ibm.com/docs/watsonx-orchestrate/admin)\n");
            response.push_str("- [Security Configuration](https://www.ibm.com/docs/watsonx-orchestrate/security)\n");
        }
        "getting_started" => {
            response.push_str("### Getting Started\n\n");
            response.push_str("Welcome to WatsonX Orchestrate! Here's how to begin:\n\n");
            response.push_str("1. **First Steps**: Log in and explore the interface\n");
            response.push_str("2. **Try a Skill**: Use a pre-built skill from the catalog\n");
            response.push_str("3. **Create Your Own**: Build a simple custom skill\n");
            response.push_str("4. **Automate**: Combine skills into workflows\n\n");
            response.push_str("**Quick Links:**\n");
            response.push_str("- [Quick Start Guide](https://www.ibm.com/docs/watsonx-orchestrate/quickstart)\n");
            response.push_str("- [Tutorial Videos](https://www.ibm.com/docs/watsonx-orchestrate/tutorials)\n");
        }
        "troubleshooting" => {
            response.push_str("### Troubleshooting Documentation\n\n");
            response.push_str("Find solutions for common issues:\n\n");
            response.push_str("- **Authentication Issues**: Login and access problems\n");
            response.push_str("- **Skill Errors**: Execution failures and debugging\n");
            response.push_str("- **Integration Problems**: Connection and sync issues\n");
            response.push_str("- **Performance**: Slow operations and timeouts\n\n");
            response.push_str("**Quick Links:**\n");
            response.push_str("- [Troubleshooting Guide](https://www.ibm.com/docs/watsonx-orchestrate/troubleshooting)\n");
            response.push_str("- [Known Issues](https://www.ibm.com/docs/watsonx-orchestrate/known-issues)\n");
        }
        "release_notes" => {
            response.push_str("### Release Notes\n\n");
            response.push_str("Stay up to date with WatsonX Orchestrate:\n\n");
            response.push_str("- **New Features**: Latest capabilities added\n");
            response.push_str("- **Improvements**: Enhancements to existing features\n");
            response.push_str("- **Bug Fixes**: Issues that have been resolved\n");
            response.push_str("- **Breaking Changes**: Updates that may require action\n\n");
            response.push_str("**Quick Links:**\n");
            response.push_str("- [Latest Release Notes](https://www.ibm.com/docs/watsonx-orchestrate/release-notes)\n");
            response.push_str("- [Roadmap](https://www.ibm.com/docs/watsonx-orchestrate/roadmap)\n");
        }
        _ => {
            response.push_str("### User Documentation\n\n");
            response.push_str("User documentation helps you work effectively:\n\n");
            response.push_str("- **Skills**: Create and use automation skills\n");
            response.push_str("- **Workflows**: Build multi-step automations\n");
            response.push_str("- **Catalog**: Find pre-built integrations\n");
            response.push_str("- **AI Features**: Natural language interaction\n\n");
            response.push_str("**Quick Links:**\n");
            response.push_str("- [User Guide](https://www.ibm.com/docs/watsonx-orchestrate/user)\n");
            response.push_str("- [Skill Catalog](https://www.ibm.com/docs/watsonx-orchestrate/catalog)\n");
        }
    }

    // Include search results if available
    if !tool_results.is_empty() {
        response.push_str("\n---\n\n### 🔍 Relevant Documentation Found\n\n");
        response.push_str("Based on your query, here are the most relevant docs:\n\n");

        // Parse and format tool results
        for result in tool_results {
            if let Ok(docs) = serde_json::from_str::<Vec<serde_json::Value>>(result) {
                for doc in docs.iter().take(3) {
                    if let (Some(title), Some(url)) = (
                        doc.get("title").and_then(|t| t.as_str()),
                        doc.get("url").and_then(|u| u.as_str()),
                    ) {
                        response.push_str(&format!("- **[{}]({})**", title, url));
                        if let Some(content) = doc.get("content").and_then(|c| c.as_str()) {
                            let excerpt = if content.len() > 100 {
                                format!("{}...", &content[..100])
                            } else {
                                content.to_string()
                            };
                            response.push_str(&format!("\n  _{}_", excerpt));
                        }
                        response.push_str("\n\n");
                    }
                }
            }
        }
    }

    response.push_str("\n---\n\n");
    response.push_str("**Can't find what you need?** Try asking a more specific question or ");
    response.push_str("let me know which documentation category you're interested in.");

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::create_tool_registry;

    #[test]
    fn test_build_docs_graph() {
        let registry = Arc::new(create_tool_registry());
        let graph = DocsHelperAgent::build_graph(registry);
        assert!(graph.is_ok());
    }

    #[test]
    fn test_categorize_api() {
        let category = categorize_docs_request("How do I use the API to create a skill?");
        assert_eq!(category.primary, "api");
    }

    #[test]
    fn test_categorize_admin() {
        let category = categorize_docs_request("How do I configure SSO?");
        assert_eq!(category.primary, "admin");
    }
}
