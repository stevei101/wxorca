//! Fetch code examples tool for WatsonX Orchestrate

use async_trait::async_trait;
use oxidizedgraph::prelude::{NodeError, Tool};
use serde::{Deserialize, Serialize};

/// Tool for fetching code examples for WatsonX Orchestrate
pub struct FetchExamplesTool;

impl FetchExamplesTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FetchExamplesTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct FetchExamplesInput {
    topic: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    3
}

#[derive(Debug, Serialize)]
struct CodeExample {
    title: String,
    description: String,
    language: String,
    code: String,
    tags: Vec<String>,
}

#[async_trait]
impl Tool for FetchExamplesTool {
    fn name(&self) -> &str {
        "fetch_wxo_examples"
    }

    fn description(&self) -> &str {
        "Fetch code examples and sample configurations for WatsonX Orchestrate. \
         Returns relevant examples for skills, workflows, integrations, and API usage."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "topic": {
                    "type": "string",
                    "description": "Topic to find examples for (e.g., 'skill creation', 'salesforce integration', 'api authentication')"
                },
                "language": {
                    "type": "string",
                    "description": "Programming language filter (e.g., 'python', 'javascript', 'json')"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of examples to return (default: 3)",
                    "default": 3
                }
            },
            "required": ["topic"]
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String, NodeError> {
        let input: FetchExamplesInput = serde_json::from_value(arguments)
            .map_err(|e| NodeError::ToolError(format!("Invalid arguments: {}", e)))?;

        let examples = get_mock_examples(&input.topic, input.language.as_deref(), input.limit);

        serde_json::to_string_pretty(&examples)
            .map_err(|e| NodeError::ToolError(format!("Failed to serialize examples: {}", e)))
    }
}

fn get_mock_examples(topic: &str, language: Option<&str>, limit: usize) -> Vec<CodeExample> {
    let all_examples = vec![
        // Skill examples
        CodeExample {
            title: "Basic Skill Definition".to_string(),
            description: "A simple skill that processes text input".to_string(),
            language: "json".to_string(),
            code: r#"{
  "name": "text_processor",
  "description": "Processes and transforms text",
  "input_schema": {
    "type": "object",
    "properties": {
      "text": { "type": "string" },
      "operation": { "type": "string", "enum": ["uppercase", "lowercase", "reverse"] }
    },
    "required": ["text", "operation"]
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "result": { "type": "string" }
    }
  }
}"#
            .to_string(),
            tags: vec!["skill".to_string(), "basic".to_string()],
        },
        CodeExample {
            title: "Python Skill Implementation".to_string(),
            description: "Python code for a custom WXO skill".to_string(),
            language: "python".to_string(),
            code: r#"from wxo_sdk import Skill, SkillInput, SkillOutput

class DataValidatorSkill(Skill):
    """Validates input data against defined rules."""

    def execute(self, input: SkillInput) -> SkillOutput:
        data = input.get("data")
        rules = input.get("rules", [])

        errors = []
        for rule in rules:
            if not self._validate_rule(data, rule):
                errors.append(f"Validation failed: {rule['name']}")

        return SkillOutput(
            success=len(errors) == 0,
            errors=errors,
            validated_data=data if not errors else None
        )

    def _validate_rule(self, data, rule):
        # Rule validation logic here
        return True"#
                .to_string(),
            tags: vec!["skill".to_string(), "python".to_string(), "validation".to_string()],
        },
        // Workflow examples
        CodeExample {
            title: "Sequential Workflow".to_string(),
            description: "A workflow that executes skills in sequence".to_string(),
            language: "json".to_string(),
            code: r#"{
  "name": "customer_onboarding",
  "description": "Automated customer onboarding workflow",
  "steps": [
    {
      "id": "validate_input",
      "skill_id": "data_validator",
      "input": { "data": "{{trigger.customer_data}}" }
    },
    {
      "id": "create_account",
      "skill_id": "crm_create_account",
      "input": { "customer": "{{steps.validate_input.output}}" }
    },
    {
      "id": "send_welcome",
      "skill_id": "email_sender",
      "input": {
        "to": "{{trigger.customer_data.email}}",
        "template": "welcome_email"
      }
    }
  ],
  "error_handling": {
    "on_failure": "notify_admin"
  }
}"#
            .to_string(),
            tags: vec!["workflow".to_string(), "onboarding".to_string()],
        },
        // API examples
        CodeExample {
            title: "API Authentication".to_string(),
            description: "Authenticating with the WXO API".to_string(),
            language: "python".to_string(),
            code: r#"import requests

def get_wxo_token(api_key: str, instance_url: str) -> str:
    """Obtain an access token for WatsonX Orchestrate API."""

    response = requests.post(
        f"{instance_url}/api/v1/auth/token",
        headers={
            "Content-Type": "application/json",
            "X-API-Key": api_key
        }
    )
    response.raise_for_status()
    return response.json()["access_token"]

# Usage
token = get_wxo_token(
    api_key="your-api-key",
    instance_url="https://your-instance.watsonx-orchestrate.ibm.com"
)"#
            .to_string(),
            tags: vec!["api".to_string(), "authentication".to_string(), "python".to_string()],
        },
        CodeExample {
            title: "JavaScript API Client".to_string(),
            description: "Using the WXO API from JavaScript".to_string(),
            language: "javascript".to_string(),
            code: r#"class WXOClient {
  constructor(instanceUrl, apiKey) {
    this.instanceUrl = instanceUrl;
    this.apiKey = apiKey;
    this.token = null;
  }

  async authenticate() {
    const response = await fetch(`${this.instanceUrl}/api/v1/auth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-API-Key': this.apiKey
      }
    });
    const data = await response.json();
    this.token = data.access_token;
    return this.token;
  }

  async executeSkill(skillId, input) {
    if (!this.token) await this.authenticate();

    const response = await fetch(`${this.instanceUrl}/api/v1/skills/${skillId}/execute`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ input })
    });
    return response.json();
  }
}"#
            .to_string(),
            tags: vec!["api".to_string(), "javascript".to_string(), "client".to_string()],
        },
        // Integration examples
        CodeExample {
            title: "Salesforce Integration Config".to_string(),
            description: "Configuration for Salesforce integration".to_string(),
            language: "json".to_string(),
            code: r#"{
  "type": "salesforce",
  "name": "production_salesforce",
  "credentials": {
    "auth_type": "oauth2",
    "client_id": "{{secrets.SF_CLIENT_ID}}",
    "client_secret": "{{secrets.SF_CLIENT_SECRET}}",
    "instance_url": "https://your-org.salesforce.com"
  },
  "settings": {
    "api_version": "v58.0",
    "rate_limit": {
      "requests_per_minute": 100
    },
    "retry": {
      "max_attempts": 3,
      "backoff_ms": 1000
    }
  },
  "sync": {
    "objects": ["Account", "Contact", "Opportunity"],
    "direction": "bidirectional",
    "frequency": "real-time"
  }
}"#
            .to_string(),
            tags: vec!["integration".to_string(), "salesforce".to_string()],
        },
        CodeExample {
            title: "Error Handling Pattern".to_string(),
            description: "Best practice for error handling in workflows".to_string(),
            language: "json".to_string(),
            code: r#"{
  "name": "robust_workflow",
  "steps": [
    {
      "id": "main_task",
      "skill_id": "important_operation",
      "retry": {
        "max_attempts": 3,
        "backoff": "exponential",
        "initial_delay_ms": 1000
      },
      "timeout_ms": 30000
    }
  ],
  "error_handling": {
    "on_failure": {
      "steps": [
        {
          "id": "log_error",
          "skill_id": "error_logger",
          "input": { "error": "{{error}}" }
        },
        {
          "id": "notify_team",
          "skill_id": "slack_notifier",
          "input": {
            "channel": "#alerts",
            "message": "Workflow failed: {{workflow.name}}"
          }
        }
      ]
    },
    "on_timeout": {
      "action": "cancel_and_notify"
    }
  }
}"#
            .to_string(),
            tags: vec!["workflow".to_string(), "error-handling".to_string(), "best-practices".to_string()],
        },
    ];

    let topic_lower = topic.to_lowercase();

    let mut filtered: Vec<CodeExample> = all_examples
        .into_iter()
        .filter(|example| {
            // Filter by language if specified
            if let Some(lang) = language {
                if example.language.to_lowercase() != lang.to_lowercase() {
                    return false;
                }
            }

            // Match by topic
            let title_lower = example.title.to_lowercase();
            let desc_lower = example.description.to_lowercase();
            let tags_match = example
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&topic_lower));

            title_lower.contains(&topic_lower)
                || desc_lower.contains(&topic_lower)
                || tags_match
                || topic_lower.split_whitespace().any(|word| {
                    title_lower.contains(word)
                        || desc_lower.contains(word)
                        || example.tags.iter().any(|t| t.to_lowercase().contains(word))
                })
        })
        .collect();

    filtered.truncate(limit);
    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_examples() {
        let tool = FetchExamplesTool::new();

        let result = tool
            .execute(serde_json::json!({
                "topic": "skill",
                "limit": 2
            }))
            .await
            .unwrap();

        let examples: Vec<CodeExample> = serde_json::from_str(&result).unwrap();
        assert!(!examples.is_empty());
        assert!(examples.len() <= 2);
    }

    #[tokio::test]
    async fn test_fetch_examples_with_language() {
        let tool = FetchExamplesTool::new();

        let result = tool
            .execute(serde_json::json!({
                "topic": "api",
                "language": "python"
            }))
            .await
            .unwrap();

        let examples: Vec<CodeExample> = serde_json::from_str(&result).unwrap();
        for example in examples {
            assert_eq!(example.language.to_lowercase(), "python");
        }
    }
}
