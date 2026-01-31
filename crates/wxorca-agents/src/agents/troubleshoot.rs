//! Troubleshooting Bot Agent
//!
//! Helps users diagnose and resolve issues with WatsonX Orchestrate.

use super::{route_by_tools, AnalyzeQueryNode, ExecuteToolsNode};
use crate::state::AgentType;
use oxidizedgraph::prelude::*;
use std::sync::Arc;

/// Agent for troubleshooting WatsonX Orchestrate issues
pub struct TroubleshootAgent;

impl TroubleshootAgent {
    /// Build the agent graph for troubleshooting
    pub fn build_graph(tool_registry: Arc<ToolRegistry>) -> Result<CompiledGraph, GraphError> {
        let system_prompt = AgentType::Troubleshoot.system_prompt().to_string();

        GraphBuilder::new()
            .name("troubleshoot_agent")
            .description("Diagnoses and resolves WatsonX Orchestrate issues")
            .add_node(AnalyzeQueryNode::new("analyze"))
            .add_node(DiagnoseNode::new("diagnose"))
            .add_node(TroubleshootSearchNode::new("search_docs", system_prompt.clone()))
            .add_node(TroubleshootResponseNode::new("respond", system_prompt))
            .add_node(ExecuteToolsNode::new("execute_tools", tool_registry))
            .set_entry_point("analyze")
            .add_edge("analyze", "diagnose")
            .add_edge("diagnose", "search_docs")
            .add_edge("search_docs", "respond")
            .add_conditional_edge("respond", route_by_tools)
            .add_edge("execute_tools", "respond")
            .compile()
    }
}

struct DiagnoseNode {
    id: String,
}

impl DiagnoseNode {
    fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for DiagnoseNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Diagnoses the issue based on user description")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let query = {
            let guard = state
                .read()
                .map_err(|e| NodeError::Other(format!("Failed to read state: {}", e)))?;
            guard
                .get_context::<String>("original_query")
                                .unwrap_or_default()
        };

        let diagnosis = diagnose_issue(&query);

        {
            let mut guard = state
                .write()
                .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;
            guard.set_context("diagnosis", serde_json::json!(diagnosis));
        }

        Ok(NodeOutput::cont())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Diagnosis {
    category: String,
    severity: String,
    likely_causes: Vec<String>,
    suggested_checks: Vec<String>,
}

fn diagnose_issue(query: &str) -> Diagnosis {
    let query_lower = query.to_lowercase();

    if query_lower.contains("authentication")
        || query_lower.contains("login")
        || query_lower.contains("access denied")
        || query_lower.contains("401")
    {
        Diagnosis {
            category: "authentication".to_string(),
            severity: "high".to_string(),
            likely_causes: vec![
                "Expired credentials or tokens".to_string(),
                "Incorrect SSO configuration".to_string(),
                "User permissions not set correctly".to_string(),
                "API key revoked or expired".to_string(),
            ],
            suggested_checks: vec![
                "Verify credentials are correct".to_string(),
                "Check token expiration".to_string(),
                "Review user permissions".to_string(),
                "Test SSO configuration".to_string(),
            ],
        }
    } else if query_lower.contains("timeout")
        || query_lower.contains("slow")
        || query_lower.contains("performance")
    {
        Diagnosis {
            category: "performance".to_string(),
            severity: "medium".to_string(),
            likely_causes: vec![
                "High system load".to_string(),
                "Network latency".to_string(),
                "Large data volumes".to_string(),
                "Resource constraints".to_string(),
            ],
            suggested_checks: vec![
                "Check system status page".to_string(),
                "Monitor network connectivity".to_string(),
                "Review workflow complexity".to_string(),
                "Check concurrent user count".to_string(),
            ],
        }
    } else if query_lower.contains("integration")
        || query_lower.contains("connection")
        || query_lower.contains("api")
    {
        Diagnosis {
            category: "integration".to_string(),
            severity: "medium".to_string(),
            likely_causes: vec![
                "External service unavailable".to_string(),
                "Credentials expired".to_string(),
                "API rate limit exceeded".to_string(),
                "Configuration mismatch".to_string(),
            ],
            suggested_checks: vec![
                "Verify external service status".to_string(),
                "Check integration credentials".to_string(),
                "Review API rate limits".to_string(),
                "Test connection settings".to_string(),
            ],
        }
    } else if query_lower.contains("skill")
        || query_lower.contains("workflow")
        || query_lower.contains("failed")
    {
        Diagnosis {
            category: "execution".to_string(),
            severity: "medium".to_string(),
            likely_causes: vec![
                "Invalid input data".to_string(),
                "Missing required parameters".to_string(),
                "Skill configuration error".to_string(),
                "Dependency failure".to_string(),
            ],
            suggested_checks: vec![
                "Review input data format".to_string(),
                "Check required parameters".to_string(),
                "Validate skill configuration".to_string(),
                "Check execution logs".to_string(),
            ],
        }
    } else {
        Diagnosis {
            category: "general".to_string(),
            severity: "low".to_string(),
            likely_causes: vec![
                "Configuration issue".to_string(),
                "User error".to_string(),
                "Temporary system issue".to_string(),
            ],
            suggested_checks: vec![
                "Describe the issue in more detail".to_string(),
                "Check system status".to_string(),
                "Review recent changes".to_string(),
            ],
        }
    }
}

struct TroubleshootSearchNode {
    id: String,
    _system_prompt: String,
}

impl TroubleshootSearchNode {
    fn new(id: impl Into<String>, system_prompt: String) -> Self {
        Self {
            id: id.into(),
            _system_prompt: system_prompt,
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for TroubleshootSearchNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Searches troubleshooting documentation")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let (query, diagnosis_category) = {
            let guard = state
                .read()
                .map_err(|e| NodeError::Other(format!("Failed to read state: {}", e)))?;

            let query = guard
                .get_context::<String>("original_query")
                                .unwrap_or_default();

            let category = guard
                .get_context::<Diagnosis>("diagnosis")
                .map(|d| d.category.clone())
                .unwrap_or_else(|| "general".to_string());

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
                    "query": format!("{} {}", diagnosis_category, query),
                    "category": "troubleshooting",
                    "limit": 5
                }),
            };

            guard.tool_calls.push(tool_call);
        }

        Ok(NodeOutput::cont())
    }
}

struct TroubleshootResponseNode {
    id: String,
    system_prompt: String,
}

impl TroubleshootResponseNode {
    fn new(id: impl Into<String>, system_prompt: String) -> Self {
        Self {
            id: id.into(),
            system_prompt,
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for TroubleshootResponseNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Generates troubleshooting guidance")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let mut guard = state
            .write()
            .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;

        let query = guard
            .get_context::<String>("original_query")
                        .unwrap_or_default();

        let diagnosis = guard
            .get_context::<Diagnosis>("diagnosis")
                        .unwrap_or_else(|| Diagnosis {
                category: "general".to_string(),
                severity: "low".to_string(),
                likely_causes: vec![],
                suggested_checks: vec![],
            });

        let response = generate_troubleshoot_response(&query, &diagnosis, &self.system_prompt);

        guard.add_assistant_message(&response);
        guard.mark_complete();

        Ok(NodeOutput::finish())
    }
}

fn generate_troubleshoot_response(
    _query: &str,
    diagnosis: &Diagnosis,
    _system_prompt: &str,
) -> String {
    let mut response = String::new();

    response.push_str(&format!(
        "## 🔍 Issue Analysis: {}\n\n",
        diagnosis.category.to_uppercase()
    ));

    response.push_str(&format!(
        "**Severity**: {}\n\n",
        match diagnosis.severity.as_str() {
            "high" => "🔴 High",
            "medium" => "🟡 Medium",
            _ => "🟢 Low",
        }
    ));

    response.push_str("### Likely Causes\n");
    for cause in &diagnosis.likely_causes {
        response.push_str(&format!("- {}\n", cause));
    }
    response.push('\n');

    response.push_str("### Troubleshooting Steps\n\n");
    for (i, check) in diagnosis.suggested_checks.iter().enumerate() {
        response.push_str(&format!("{}. {}\n", i + 1, check));
    }
    response.push('\n');

    // Add category-specific advice
    match diagnosis.category.as_str() {
        "authentication" => {
            response.push_str("### Quick Fix Attempts\n");
            response.push_str("1. Clear browser cache and cookies\n");
            response.push_str("2. Try logging out and back in\n");
            response.push_str("3. Check if your session has expired\n");
            response.push_str("4. Verify your account is active\n\n");
            response.push_str("**⚠️ If issues persist**, contact your administrator to verify your account permissions.");
        }
        "performance" => {
            response.push_str("### Quick Fix Attempts\n");
            response.push_str("1. Refresh the page\n");
            response.push_str("2. Check your internet connection\n");
            response.push_str("3. Try a different browser\n");
            response.push_str("4. Check the WXO status page for outages\n\n");
            response.push_str("**💡 Tip**: If working with large datasets, try processing in smaller batches.");
        }
        "integration" => {
            response.push_str("### Quick Fix Attempts\n");
            response.push_str("1. Test the external service directly\n");
            response.push_str("2. Re-authenticate the integration\n");
            response.push_str("3. Check for API version changes\n");
            response.push_str("4. Review integration logs\n\n");
            response.push_str("**⚠️ Note**: External service issues are outside WXO control.");
        }
        "execution" => {
            response.push_str("### Quick Fix Attempts\n");
            response.push_str("1. Verify input data format\n");
            response.push_str("2. Check for required fields\n");
            response.push_str("3. Review skill/workflow logs\n");
            response.push_str("4. Test with simpler inputs\n\n");
            response.push_str("**💡 Tip**: Use the validation tool to check your configuration.");
        }
        _ => {
            response.push_str("### Need More Information\n");
            response.push_str("Could you provide more details about:\n");
            response.push_str("- What exactly happened?\n");
            response.push_str("- Any error messages shown?\n");
            response.push_str("- When did this start?\n");
            response.push_str("- Any recent changes?\n");
        }
    }

    response.push_str("\n\n---\n\n");
    response.push_str("**Still having issues?** I can help you escalate to IBM Support if needed.");

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::create_tool_registry;

    #[test]
    fn test_build_troubleshoot_graph() {
        let registry = Arc::new(create_tool_registry());
        let graph = TroubleshootAgent::build_graph(registry);
        assert!(graph.is_ok());
    }

    #[test]
    fn test_diagnose_authentication() {
        let diagnosis = diagnose_issue("I can't login, getting access denied");
        assert_eq!(diagnosis.category, "authentication");
        assert_eq!(diagnosis.severity, "high");
    }

    #[test]
    fn test_diagnose_performance() {
        let diagnosis = diagnose_issue("The workflow is running very slow");
        assert_eq!(diagnosis.category, "performance");
    }
}
