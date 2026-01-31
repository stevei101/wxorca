//! Search WatsonX Orchestrate documentation tool

use async_trait::async_trait;
use oxidizedgraph::prelude::{NodeError, Tool};
use serde::{Deserialize, Serialize};

/// Tool for searching WatsonX Orchestrate documentation
pub struct SearchDocsTool {
    // In a real implementation, this would hold a database connection
    // For now, we'll use mock data
}

impl SearchDocsTool {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for SearchDocsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct SearchDocsInput {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    category: Option<String>,
}

fn default_limit() -> usize {
    5
}

#[derive(Debug, Serialize, Deserialize)]
struct DocResult {
    title: String,
    content: String,
    url: String,
    category: String,
    relevance: f32,
}

#[async_trait]
impl Tool for SearchDocsTool {
    fn name(&self) -> &str {
        "search_wxo_docs"
    }

    fn description(&self) -> &str {
        "Search IBM WatsonX Orchestrate documentation for relevant information. \
         Use this to find setup guides, feature documentation, API references, \
         and troubleshooting information."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query for documentation"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default: 5)",
                    "default": 5
                },
                "category": {
                    "type": "string",
                    "description": "Optional category filter (e.g., 'admin', 'user', 'api', 'troubleshooting')"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String, NodeError> {
        let input: SearchDocsInput = serde_json::from_value(arguments)
            .map_err(|e| NodeError::ToolError(format!("Invalid arguments: {}", e)))?;

        // Mock documentation results
        // In a real implementation, this would query SurrealDB
        let results = get_mock_docs(&input.query, input.limit, input.category.as_deref());

        let response = serde_json::to_string_pretty(&results)
            .map_err(|e| NodeError::ToolError(format!("Failed to serialize results: {}", e)))?;

        Ok(response)
    }
}

fn get_mock_docs(query: &str, limit: usize, category: Option<&str>) -> Vec<DocResult> {
    // Mock documentation database
    let all_docs = vec![
        DocResult {
            title: "Getting Started with WatsonX Orchestrate".to_string(),
            content: "WatsonX Orchestrate is an AI-powered automation platform that helps you \
                     work more efficiently by automating repetitive tasks and providing \
                     intelligent assistance.".to_string(),
            url: "https://www.ibm.com/docs/watsonx-orchestrate/getting-started".to_string(),
            category: "user".to_string(),
            relevance: 0.95,
        },
        DocResult {
            title: "Admin Setup Guide".to_string(),
            content: "This guide walks administrators through the initial setup of WatsonX \
                     Orchestrate, including user management, security configuration, and \
                     integration setup.".to_string(),
            url: "https://www.ibm.com/docs/watsonx-orchestrate/admin-guide".to_string(),
            category: "admin".to_string(),
            relevance: 0.92,
        },
        DocResult {
            title: "Creating Custom Skills".to_string(),
            content: "Learn how to create custom skills in WatsonX Orchestrate. Skills are \
                     reusable automation components that can be combined into workflows.".to_string(),
            url: "https://www.ibm.com/docs/watsonx-orchestrate/skills".to_string(),
            category: "user".to_string(),
            relevance: 0.88,
        },
        DocResult {
            title: "API Reference".to_string(),
            content: "Complete API reference for WatsonX Orchestrate, including authentication, \
                     skill management, and workflow execution endpoints.".to_string(),
            url: "https://www.ibm.com/docs/watsonx-orchestrate/api".to_string(),
            category: "api".to_string(),
            relevance: 0.85,
        },
        DocResult {
            title: "Troubleshooting Common Issues".to_string(),
            content: "Solutions for common issues including authentication failures, skill \
                     execution errors, and integration problems.".to_string(),
            url: "https://www.ibm.com/docs/watsonx-orchestrate/troubleshooting".to_string(),
            category: "troubleshooting".to_string(),
            relevance: 0.82,
        },
        DocResult {
            title: "Integration with Salesforce".to_string(),
            content: "Step-by-step guide for integrating WatsonX Orchestrate with Salesforce, \
                     enabling CRM automation and data synchronization.".to_string(),
            url: "https://www.ibm.com/docs/watsonx-orchestrate/integrations/salesforce".to_string(),
            category: "admin".to_string(),
            relevance: 0.78,
        },
        DocResult {
            title: "Security Best Practices".to_string(),
            content: "Security recommendations for WatsonX Orchestrate deployments, including \
                     authentication, access control, and data protection.".to_string(),
            url: "https://www.ibm.com/docs/watsonx-orchestrate/security".to_string(),
            category: "admin".to_string(),
            relevance: 0.75,
        },
        DocResult {
            title: "Workflow Automation Patterns".to_string(),
            content: "Common workflow patterns and best practices for building efficient \
                     automations in WatsonX Orchestrate.".to_string(),
            url: "https://www.ibm.com/docs/watsonx-orchestrate/workflows".to_string(),
            category: "user".to_string(),
            relevance: 0.72,
        },
    ];

    let query_lower = query.to_lowercase();

    let mut filtered: Vec<DocResult> = all_docs
        .into_iter()
        .filter(|doc| {
            // Filter by category if specified
            if let Some(cat) = category {
                if doc.category != cat {
                    return false;
                }
            }

            // Simple relevance matching
            let title_lower = doc.title.to_lowercase();
            let content_lower = doc.content.to_lowercase();

            title_lower.contains(&query_lower)
                || content_lower.contains(&query_lower)
                || query_lower.split_whitespace().any(|word| {
                    title_lower.contains(word) || content_lower.contains(word)
                })
        })
        .collect();

    // Sort by relevance
    filtered.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());

    // Limit results
    filtered.truncate(limit);

    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_docs() {
        let tool = SearchDocsTool::new();

        let result = tool
            .execute(serde_json::json!({
                "query": "setup",
                "limit": 3
            }))
            .await
            .unwrap();

        let docs: Vec<DocResult> = serde_json::from_str(&result).unwrap();
        assert!(!docs.is_empty());
        assert!(docs.len() <= 3);
    }

    #[tokio::test]
    async fn test_search_docs_with_category() {
        let tool = SearchDocsTool::new();

        let result = tool
            .execute(serde_json::json!({
                "query": "guide",
                "category": "admin"
            }))
            .await
            .unwrap();

        let docs: Vec<DocResult> = serde_json::from_str(&result).unwrap();
        for doc in docs {
            assert_eq!(doc.category, "admin");
        }
    }
}
