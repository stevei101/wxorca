//! Best Practices Coach Agent
//!
//! Provides optimization tips and best practices for WatsonX Orchestrate.

use super::{route_by_tools, AnalyzeQueryNode, ExecuteToolsNode};
use crate::state::AgentType;
use oxidizedgraph::prelude::*;
use std::sync::Arc;

/// Agent for providing best practices guidance
pub struct BestPracticesAgent;

impl BestPracticesAgent {
    /// Build the agent graph for best practices coaching
    pub fn build_graph(tool_registry: Arc<ToolRegistry>) -> Result<CompiledGraph, GraphError> {
        let system_prompt = AgentType::BestPractices.system_prompt().to_string();

        GraphBuilder::new()
            .name("best_practices_agent")
            .description("Provides optimization tips and best practices")
            .add_node(AnalyzeQueryNode::new("analyze"))
            .add_node(AssessmentNode::new("assess"))
            .add_node(BestPracticesSearchNode::new("search_docs", system_prompt.clone()))
            .add_node(BestPracticesResponseNode::new("respond", system_prompt))
            .add_node(ExecuteToolsNode::new("execute_tools", tool_registry))
            .set_entry_point("analyze")
            .add_edge("analyze", "assess")
            .add_edge("assess", "search_docs")
            .add_edge("search_docs", "respond")
            .add_conditional_edge("respond", route_by_tools)
            .add_edge("execute_tools", "respond")
            .compile()
    }
}

struct AssessmentNode {
    id: String,
}

impl AssessmentNode {
    fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for AssessmentNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Assesses the user's needs for best practices guidance")
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

        let topic = identify_best_practices_topic(&query);

        {
            let mut guard = state
                .write()
                .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;
            guard.set_context("bp_topic", serde_json::json!(topic));
        }

        Ok(NodeOutput::cont())
    }
}

fn identify_best_practices_topic(query: &str) -> &'static str {
    let query_lower = query.to_lowercase();

    if query_lower.contains("workflow") || query_lower.contains("automation") {
        "workflow_design"
    } else if query_lower.contains("performance") || query_lower.contains("speed") {
        "performance"
    } else if query_lower.contains("security") || query_lower.contains("permission") {
        "security"
    } else if query_lower.contains("skill") || query_lower.contains("catalog") {
        "skill_design"
    } else if query_lower.contains("team") || query_lower.contains("collaborate") {
        "collaboration"
    } else if query_lower.contains("error") || query_lower.contains("handle") {
        "error_handling"
    } else if query_lower.contains("test") || query_lower.contains("deploy") {
        "deployment"
    } else {
        "general"
    }
}

struct BestPracticesSearchNode {
    id: String,
    _system_prompt: String,
}

impl BestPracticesSearchNode {
    fn new(id: impl Into<String>, system_prompt: String) -> Self {
        Self {
            id: id.into(),
            _system_prompt: system_prompt,
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for BestPracticesSearchNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Searches for relevant best practices documentation")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let (query, topic) = {
            let guard = state
                .read()
                .map_err(|e| NodeError::Other(format!("Failed to read state: {}", e)))?;

            let query = guard
                .get_context::<String>("original_query")
                                .unwrap_or_default();

            let topic = guard
                .get_context::<String>("bp_topic")
                                .unwrap_or_else(|| "general".to_string());

            (query, topic)
        };

        if query.is_empty() {
            return Ok(NodeOutput::cont());
        }

        {
            let mut guard = state
                .write()
                .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;

            // Search for best practices examples
            let tool_call = ToolCall {
                id: uuid::Uuid::new_v4().to_string(),
                name: "fetch_wxo_examples".to_string(),
                arguments: serde_json::json!({
                    "topic": format!("{} best practices", topic.replace('_', " ")),
                    "limit": 3
                }),
            };

            guard.tool_calls.push(tool_call);
        }

        Ok(NodeOutput::cont())
    }
}

struct BestPracticesResponseNode {
    id: String,
    system_prompt: String,
}

impl BestPracticesResponseNode {
    fn new(id: impl Into<String>, system_prompt: String) -> Self {
        Self {
            id: id.into(),
            system_prompt,
        }
    }
}

#[async_trait::async_trait]
impl NodeExecutor for BestPracticesResponseNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> Option<&str> {
        Some("Generates best practices recommendations")
    }

    async fn execute(&self, state: SharedState) -> Result<NodeOutput, NodeError> {
        let mut guard = state
            .write()
            .map_err(|e| NodeError::Other(format!("Failed to write state: {}", e)))?;

        let query = guard
            .get_context::<String>("original_query")
                        .unwrap_or_default();

        let topic = guard
            .get_context::<String>("bp_topic")
                        .unwrap_or_else(|| "general".to_string());

        let response = generate_best_practices_response(&query, &topic, &self.system_prompt);

        guard.add_assistant_message(&response);
        guard.mark_complete();

        Ok(NodeOutput::finish())
    }
}

fn generate_best_practices_response(
    _query: &str,
    topic: &str,
    _system_prompt: &str,
) -> String {
    let mut response = String::new();

    response.push_str(&format!(
        "## 🏆 Best Practices: {}\n\n",
        topic.replace('_', " ").to_uppercase()
    ));

    match topic {
        "workflow_design" => {
            response.push_str("### Workflow Design Principles\n\n");
            response.push_str("**1. Keep it Modular**\n");
            response.push_str("- Break complex workflows into reusable sub-workflows\n");
            response.push_str("- Each workflow should do one thing well\n");
            response.push_str("- Use consistent naming conventions\n\n");

            response.push_str("**2. Plan for Failure**\n");
            response.push_str("- Add error handling at each critical step\n");
            response.push_str("- Use retries with exponential backoff\n");
            response.push_str("- Log failures for debugging\n\n");

            response.push_str("**3. Document Everything**\n");
            response.push_str("- Add descriptions to workflows and steps\n");
            response.push_str("- Document expected inputs and outputs\n");
            response.push_str("- Maintain a changelog\n\n");

            response.push_str("**4. Test Thoroughly**\n");
            response.push_str("- Test with edge cases\n");
            response.push_str("- Use staging environments\n");
            response.push_str("- Validate before production deployment\n");
        }
        "performance" => {
            response.push_str("### Performance Optimization\n\n");
            response.push_str("**1. Minimize External Calls**\n");
            response.push_str("- Batch operations when possible\n");
            response.push_str("- Cache frequently accessed data\n");
            response.push_str("- Use efficient queries\n\n");

            response.push_str("**2. Optimize Workflow Design**\n");
            response.push_str("- Run independent steps in parallel\n");
            response.push_str("- Avoid unnecessary data transformations\n");
            response.push_str("- Set appropriate timeouts\n\n");

            response.push_str("**3. Monitor and Measure**\n");
            response.push_str("- Track execution times\n");
            response.push_str("- Identify bottlenecks\n");
            response.push_str("- Set up alerts for slow operations\n\n");

            response.push_str("**4. Resource Management**\n");
            response.push_str("- Be mindful of API rate limits\n");
            response.push_str("- Schedule heavy operations off-peak\n");
            response.push_str("- Clean up unused resources\n");
        }
        "security" => {
            response.push_str("### Security Best Practices\n\n");
            response.push_str("**1. Access Control**\n");
            response.push_str("- Follow the principle of least privilege\n");
            response.push_str("- Review permissions regularly\n");
            response.push_str("- Use role-based access control (RBAC)\n\n");

            response.push_str("**2. Credential Management**\n");
            response.push_str("- Never hardcode credentials\n");
            response.push_str("- Use secure secret storage\n");
            response.push_str("- Rotate credentials regularly\n\n");

            response.push_str("**3. Data Protection**\n");
            response.push_str("- Encrypt sensitive data in transit and at rest\n");
            response.push_str("- Minimize data retention\n");
            response.push_str("- Audit data access\n\n");

            response.push_str("**4. Monitoring & Compliance**\n");
            response.push_str("- Enable audit logging\n");
            response.push_str("- Set up security alerts\n");
            response.push_str("- Conduct regular security reviews\n");
        }
        "skill_design" => {
            response.push_str("### Skill Design Best Practices\n\n");
            response.push_str("**1. Clear Interface**\n");
            response.push_str("- Define explicit input/output schemas\n");
            response.push_str("- Use descriptive parameter names\n");
            response.push_str("- Provide helpful descriptions\n\n");

            response.push_str("**2. Validation**\n");
            response.push_str("- Validate inputs early\n");
            response.push_str("- Return clear error messages\n");
            response.push_str("- Handle edge cases gracefully\n\n");

            response.push_str("**3. Discoverability**\n");
            response.push_str("- Use meaningful skill names\n");
            response.push_str("- Add relevant tags\n");
            response.push_str("- Include usage examples\n\n");

            response.push_str("**4. Maintainability**\n");
            response.push_str("- Version your skills\n");
            response.push_str("- Plan for backward compatibility\n");
            response.push_str("- Document changes\n");
        }
        "error_handling" => {
            response.push_str("### Error Handling Best Practices\n\n");
            response.push_str("**1. Anticipate Failures**\n");
            response.push_str("- External services can fail\n");
            response.push_str("- Data may be invalid\n");
            response.push_str("- Networks can be unreliable\n\n");

            response.push_str("**2. Handle Gracefully**\n");
            response.push_str("- Catch specific exceptions\n");
            response.push_str("- Provide meaningful error messages\n");
            response.push_str("- Offer recovery options when possible\n\n");

            response.push_str("**3. Retry Strategy**\n");
            response.push_str("- Use exponential backoff\n");
            response.push_str("- Set maximum retry limits\n");
            response.push_str("- Know when to give up\n\n");

            response.push_str("**4. Logging & Alerting**\n");
            response.push_str("- Log errors with context\n");
            response.push_str("- Set up alerts for critical failures\n");
            response.push_str("- Track error patterns\n");
        }
        "deployment" => {
            response.push_str("### Deployment Best Practices\n\n");
            response.push_str("**1. Environment Strategy**\n");
            response.push_str("- Use separate dev/staging/prod environments\n");
            response.push_str("- Test in staging first\n");
            response.push_str("- Use consistent configurations\n\n");

            response.push_str("**2. Testing**\n");
            response.push_str("- Write automated tests\n");
            response.push_str("- Test with realistic data\n");
            response.push_str("- Perform load testing\n\n");

            response.push_str("**3. Rollout Strategy**\n");
            response.push_str("- Use gradual rollouts\n");
            response.push_str("- Monitor after deployment\n");
            response.push_str("- Have a rollback plan\n\n");

            response.push_str("**4. Documentation**\n");
            response.push_str("- Document deployment steps\n");
            response.push_str("- Maintain runbooks\n");
            response.push_str("- Keep change logs\n");
        }
        "collaboration" => {
            response.push_str("### Team Collaboration Best Practices\n\n");
            response.push_str("**1. Organization**\n");
            response.push_str("- Use consistent naming conventions\n");
            response.push_str("- Organize skills and workflows logically\n");
            response.push_str("- Use tags for easy discovery\n\n");

            response.push_str("**2. Sharing**\n");
            response.push_str("- Share reusable components\n");
            response.push_str("- Document shared resources\n");
            response.push_str("- Set appropriate permissions\n\n");

            response.push_str("**3. Communication**\n");
            response.push_str("- Document changes clearly\n");
            response.push_str("- Notify teams of updates\n");
            response.push_str("- Establish review processes\n\n");

            response.push_str("**4. Standards**\n");
            response.push_str("- Define coding standards\n");
            response.push_str("- Create templates\n");
            response.push_str("- Review and improve regularly\n");
        }
        _ => {
            response.push_str("### General Best Practices\n\n");
            response.push_str("**Start Simple**\n");
            response.push_str("- Begin with basic implementations\n");
            response.push_str("- Add complexity as needed\n");
            response.push_str("- Iterate based on feedback\n\n");

            response.push_str("**Document Everything**\n");
            response.push_str("- Write clear descriptions\n");
            response.push_str("- Maintain up-to-date documentation\n");
            response.push_str("- Include examples\n\n");

            response.push_str("**Test Thoroughly**\n");
            response.push_str("- Test before deploying\n");
            response.push_str("- Use realistic scenarios\n");
            response.push_str("- Monitor in production\n\n");

            response.push_str("What specific area would you like guidance on?\n");
            response.push_str("- Workflow design\n");
            response.push_str("- Performance optimization\n");
            response.push_str("- Security\n");
            response.push_str("- Skill development\n");
        }
    }

    response.push_str("\n\n---\n\n");
    response.push_str("**💡 Need more specific advice?** Tell me about your use case and I can provide tailored recommendations.");

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::create_tool_registry;

    #[test]
    fn test_build_best_practices_graph() {
        let registry = Arc::new(create_tool_registry());
        let graph = BestPracticesAgent::build_graph(registry);
        assert!(graph.is_ok());
    }

    #[test]
    fn test_identify_topic() {
        assert_eq!(
            identify_best_practices_topic("How should I design my workflow?"),
            "workflow_design"
        );
        assert_eq!(
            identify_best_practices_topic("How can I improve performance?"),
            "performance"
        );
        assert_eq!(
            identify_best_practices_topic("What are security best practices?"),
            "security"
        );
    }
}
