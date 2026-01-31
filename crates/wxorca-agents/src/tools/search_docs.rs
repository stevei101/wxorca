//! Search WatsonX Orchestrate documentation tool

use async_trait::async_trait;
use oxidizedgraph::prelude::{NodeError, Tool};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};
use tracing;

/// Tool for searching WatsonX Orchestrate documentation
pub struct SearchDocsTool {
    db_host: String,
    db_port: u16,
    db_user: String,
    db_pass: String,
}

impl SearchDocsTool {
    pub fn new() -> Self {
        Self {
            db_host: std::env::var("SURREAL_HOST").unwrap_or_else(|_| "localhost".to_string()),
            db_port: std::env::var("SURREAL_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8000),
            db_user: std::env::var("SURREAL_USER").unwrap_or_else(|_| "root".to_string()),
            db_pass: std::env::var("SURREAL_PASS").unwrap_or_else(|_| "root".to_string()),
        }
    }

    async fn connect_db(&self) -> Result<Surreal<Client>, NodeError> {
        let url = format!("{}:{}", self.db_host, self.db_port);
        let client = Surreal::new::<Ws>(&url)
            .await
            .map_err(|e| NodeError::ToolError(format!("Failed to connect to SurrealDB: {}", e)))?;

        client
            .signin(Root {
                username: &self.db_user,
                password: &self.db_pass,
            })
            .await
            .map_err(|e| NodeError::ToolError(format!("Failed to authenticate: {}", e)))?;

        client
            .use_ns("wxorca")
            .use_db("main")
            .await
            .map_err(|e| NodeError::ToolError(format!("Failed to select database: {}", e)))?;

        Ok(client)
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

        // Try to query SurrealDB, fall back to mock data if connection fails
        let results = match self.query_surreal_db(&input).await {
            Ok(docs) if !docs.is_empty() => docs,
            Ok(_) => {
                // No results from DB, use mock data
                get_mock_docs(&input.query, input.limit, input.category.as_deref())
            }
            Err(e) => {
                tracing::warn!("SurrealDB query failed, using mock data: {}", e);
                get_mock_docs(&input.query, input.limit, input.category.as_deref())
            }
        };

        let response = serde_json::to_string_pretty(&results)
            .map_err(|e| NodeError::ToolError(format!("Failed to serialize results: {}", e)))?;

        Ok(response)
    }
}

impl SearchDocsTool {
    async fn query_surreal_db(&self, input: &SearchDocsInput) -> Result<Vec<DocResult>, NodeError> {
        let client = self.connect_db().await?;

        // Build query based on whether category filter is present
        let query_str = if input.category.is_some() {
            r#"
            SELECT title, content, url, category FROM wxo_docs
            WHERE category = $category
            LIMIT $limit
            "#
        } else {
            r#"
            SELECT title, content, url, category FROM wxo_docs
            LIMIT $limit
            "#
        };

        let mut query = client.query(query_str).bind(("limit", input.limit));

        if let Some(ref cat) = input.category {
            query = query.bind(("category", cat.clone()));
        }

        let mut result = query
            .await
            .map_err(|e| NodeError::ToolError(format!("Query failed: {}", e)))?;

        #[derive(Debug, Deserialize)]
        struct DbDoc {
            title: String,
            content: String,
            url: String,
            category: String,
        }

        let db_docs: Vec<DbDoc> = result.take(0).map_err(|e| {
            NodeError::ToolError(format!("Failed to parse results: {}", e))
        })?;

        // Convert to DocResult with relevance scoring
        let query_lower = input.query.to_lowercase();
        let results: Vec<DocResult> = db_docs
            .into_iter()
            .map(|doc| {
                // Simple relevance scoring based on query match
                let title_lower = doc.title.to_lowercase();
                let content_lower = doc.content.to_lowercase();
                let mut relevance = 0.5f32;

                if title_lower.contains(&query_lower) {
                    relevance += 0.3;
                }
                if content_lower.contains(&query_lower) {
                    relevance += 0.2;
                }
                for word in query_lower.split_whitespace() {
                    if title_lower.contains(word) {
                        relevance += 0.05;
                    }
                    if content_lower.contains(word) {
                        relevance += 0.03;
                    }
                }
                relevance = relevance.min(1.0);

                DocResult {
                    title: doc.title,
                    content: if doc.content.len() > 500 {
                        format!("{}...", &doc.content[..500])
                    } else {
                        doc.content
                    },
                    url: doc.url,
                    category: doc.category,
                    relevance,
                }
            })
            .collect();

        Ok(results)
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
