//! Validate WatsonX Orchestrate configuration tool

use async_trait::async_trait;
use oxidizedgraph::prelude::{NodeError, Tool};
use serde::{Deserialize, Serialize};

/// Tool for validating WatsonX Orchestrate configurations
pub struct ValidateConfigTool;

impl ValidateConfigTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ValidateConfigTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct ValidateConfigInput {
    config_type: ConfigType,
    config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ConfigType {
    Skill,
    Workflow,
    Integration,
    Authentication,
}

#[derive(Debug, Serialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<ValidationError>,
    warnings: Vec<ValidationWarning>,
    suggestions: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ValidationError {
    field: String,
    message: String,
    code: String,
}

#[derive(Debug, Serialize)]
struct ValidationWarning {
    field: String,
    message: String,
}

#[async_trait]
impl Tool for ValidateConfigTool {
    fn name(&self) -> &str {
        "validate_wxo_config"
    }

    fn description(&self) -> &str {
        "Validate WatsonX Orchestrate configuration objects like skills, workflows, \
         integrations, and authentication settings. Returns validation errors, \
         warnings, and suggestions for improvement."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "config_type": {
                    "type": "string",
                    "enum": ["skill", "workflow", "integration", "authentication"],
                    "description": "Type of configuration to validate"
                },
                "config": {
                    "type": "object",
                    "description": "The configuration object to validate"
                }
            },
            "required": ["config_type", "config"]
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String, NodeError> {
        let input: ValidateConfigInput = serde_json::from_value(arguments)
            .map_err(|e| NodeError::ToolError(format!("Invalid arguments: {}", e)))?;

        let result = match input.config_type {
            ConfigType::Skill => validate_skill_config(&input.config),
            ConfigType::Workflow => validate_workflow_config(&input.config),
            ConfigType::Integration => validate_integration_config(&input.config),
            ConfigType::Authentication => validate_auth_config(&input.config),
        };

        serde_json::to_string_pretty(&result)
            .map_err(|e| NodeError::ToolError(format!("Failed to serialize result: {}", e)))
    }
}

fn validate_skill_config(config: &serde_json::Value) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    // Check required fields
    if config.get("name").is_none() {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Skill name is required".to_string(),
            code: "MISSING_REQUIRED_FIELD".to_string(),
        });
    }

    if config.get("description").is_none() {
        warnings.push(ValidationWarning {
            field: "description".to_string(),
            message: "Adding a description helps users understand what this skill does".to_string(),
        });
    }

    if config.get("input_schema").is_none() {
        warnings.push(ValidationWarning {
            field: "input_schema".to_string(),
            message: "Defining an input schema improves validation and user experience".to_string(),
        });
    }

    // Check for common issues
    if let Some(name) = config.get("name").and_then(|n| n.as_str()) {
        if name.contains(' ') {
            errors.push(ValidationError {
                field: "name".to_string(),
                message: "Skill name should not contain spaces. Use underscores or hyphens."
                    .to_string(),
                code: "INVALID_NAME_FORMAT".to_string(),
            });
        }
        if name.len() > 64 {
            errors.push(ValidationError {
                field: "name".to_string(),
                message: "Skill name must be 64 characters or less".to_string(),
                code: "NAME_TOO_LONG".to_string(),
            });
        }
    }

    suggestions.push("Consider adding example inputs to help users understand expected values".to_string());
    suggestions.push("Add tags to make the skill easier to find in the catalog".to_string());

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
        suggestions,
    }
}

fn validate_workflow_config(config: &serde_json::Value) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    // Check required fields
    if config.get("name").is_none() {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Workflow name is required".to_string(),
            code: "MISSING_REQUIRED_FIELD".to_string(),
        });
    }

    if config.get("steps").is_none() {
        errors.push(ValidationError {
            field: "steps".to_string(),
            message: "Workflow must have at least one step".to_string(),
            code: "MISSING_REQUIRED_FIELD".to_string(),
        });
    } else if let Some(steps) = config.get("steps").and_then(|s| s.as_array()) {
        if steps.is_empty() {
            errors.push(ValidationError {
                field: "steps".to_string(),
                message: "Workflow must have at least one step".to_string(),
                code: "EMPTY_STEPS".to_string(),
            });
        }

        // Check each step
        for (i, step) in steps.iter().enumerate() {
            if step.get("skill_id").is_none() && step.get("action").is_none() {
                errors.push(ValidationError {
                    field: format!("steps[{}]", i),
                    message: "Each step must have either a skill_id or action".to_string(),
                    code: "INVALID_STEP".to_string(),
                });
            }
        }
    }

    if config.get("error_handling").is_none() {
        warnings.push(ValidationWarning {
            field: "error_handling".to_string(),
            message: "Consider adding error handling to make the workflow more robust".to_string(),
        });
    }

    suggestions.push("Add a timeout to prevent workflows from running indefinitely".to_string());
    suggestions.push("Consider adding conditional logic for different scenarios".to_string());

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
        suggestions,
    }
}

fn validate_integration_config(config: &serde_json::Value) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    // Check required fields
    if config.get("type").is_none() {
        errors.push(ValidationError {
            field: "type".to_string(),
            message: "Integration type is required".to_string(),
            code: "MISSING_REQUIRED_FIELD".to_string(),
        });
    }

    if config.get("credentials").is_none() {
        errors.push(ValidationError {
            field: "credentials".to_string(),
            message: "Integration credentials are required".to_string(),
            code: "MISSING_REQUIRED_FIELD".to_string(),
        });
    }

    // Check for security issues
    if let Some(creds) = config.get("credentials") {
        if creds.get("password").is_some() {
            warnings.push(ValidationWarning {
                field: "credentials.password".to_string(),
                message: "Consider using API keys or OAuth instead of passwords".to_string(),
            });
        }
    }

    if config.get("rate_limit").is_none() {
        warnings.push(ValidationWarning {
            field: "rate_limit".to_string(),
            message: "Setting a rate limit prevents overloading the external service".to_string(),
        });
    }

    suggestions.push("Test the integration in a sandbox environment first".to_string());
    suggestions.push("Set up monitoring for integration failures".to_string());

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
        suggestions,
    }
}

fn validate_auth_config(config: &serde_json::Value) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    // Check authentication method
    if config.get("method").is_none() {
        errors.push(ValidationError {
            field: "method".to_string(),
            message: "Authentication method is required".to_string(),
            code: "MISSING_REQUIRED_FIELD".to_string(),
        });
    }

    // Check for security best practices
    if let Some(method) = config.get("method").and_then(|m| m.as_str()) {
        match method {
            "basic" => {
                warnings.push(ValidationWarning {
                    field: "method".to_string(),
                    message: "Basic authentication is less secure. Consider using OAuth or API keys"
                        .to_string(),
                });
            }
            "oauth" => {
                if config.get("token_refresh").is_none() {
                    warnings.push(ValidationWarning {
                        field: "token_refresh".to_string(),
                        message: "Configure token refresh to prevent authentication failures"
                            .to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    // Check session settings
    if let Some(session) = config.get("session") {
        if let Some(timeout) = session.get("timeout").and_then(|t| t.as_i64()) {
            if timeout > 86400 {
                warnings.push(ValidationWarning {
                    field: "session.timeout".to_string(),
                    message: "Session timeout longer than 24 hours may be a security risk"
                        .to_string(),
                });
            }
        }
    }

    suggestions.push("Enable multi-factor authentication for admin accounts".to_string());
    suggestions.push("Set up audit logging for authentication events".to_string());
    suggestions.push("Regularly rotate API keys and tokens".to_string());

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
        suggestions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_skill_valid() {
        let tool = ValidateConfigTool::new();

        let result = tool
            .execute(serde_json::json!({
                "config_type": "skill",
                "config": {
                    "name": "my_skill",
                    "description": "A test skill",
                    "input_schema": {}
                }
            }))
            .await
            .unwrap();

        let validation: ValidationResult = serde_json::from_str(&result).unwrap();
        assert!(validation.valid);
        assert!(validation.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_skill_invalid() {
        let tool = ValidateConfigTool::new();

        let result = tool
            .execute(serde_json::json!({
                "config_type": "skill",
                "config": {
                    "name": "my invalid skill"
                }
            }))
            .await
            .unwrap();

        let validation: ValidationResult = serde_json::from_str(&result).unwrap();
        assert!(!validation.valid);
        assert!(!validation.errors.is_empty());
    }
}
