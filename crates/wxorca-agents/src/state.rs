//! State definitions for WXOrca agents
//!
//! Defines the state that flows through agent graphs, including
//! conversation history, user context, and WatsonX Orchestrate-specific data.

use chrono::{DateTime, Utc};
use oxidizedgraph::prelude::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The type of agent handling the conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AgentType {
    /// Guides administrators through WXO setup and configuration
    #[default]
    AdminSetup,
    /// Helps users understand how to use WXO features
    UsageAssistant,
    /// Diagnoses and resolves common WXO issues
    Troubleshoot,
    /// Provides optimization tips and best practices
    BestPractices,
    /// Navigates and explains WXO documentation
    DocsHelper,
}

impl AgentType {
    /// Get the display name for this agent type
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::AdminSetup => "Admin Setup Guide",
            AgentType::UsageAssistant => "Usage Assistant",
            AgentType::Troubleshoot => "Troubleshooting Bot",
            AgentType::BestPractices => "Best Practices Coach",
            AgentType::DocsHelper => "Documentation Helper",
        }
    }

    /// Get a description of what this agent does
    pub fn description(&self) -> &'static str {
        match self {
            AgentType::AdminSetup => {
                "I help administrators set up and configure IBM WatsonX Orchestrate. \
                 I can guide you through initial setup, user management, integrations, \
                 and security configuration."
            }
            AgentType::UsageAssistant => {
                "I help you understand how to use WatsonX Orchestrate effectively. \
                 Ask me about creating skills, building automations, or using the catalog."
            }
            AgentType::Troubleshoot => {
                "I help diagnose and resolve issues with WatsonX Orchestrate. \
                 Describe your problem and I'll help you find a solution."
            }
            AgentType::BestPractices => {
                "I provide optimization tips and best practices for WatsonX Orchestrate. \
                 I can help you design better workflows and improve performance."
            }
            AgentType::DocsHelper => {
                "I help you navigate and understand WatsonX Orchestrate documentation. \
                 Ask me about any feature and I'll find the relevant docs."
            }
        }
    }

    /// Get the system prompt for this agent type
    pub fn system_prompt(&self) -> &'static str {
        match self {
            AgentType::AdminSetup => include_str!("prompts/admin_setup.txt"),
            AgentType::UsageAssistant => include_str!("prompts/usage_assistant.txt"),
            AgentType::Troubleshoot => include_str!("prompts/troubleshoot.txt"),
            AgentType::BestPractices => include_str!("prompts/best_practices.txt"),
            AgentType::DocsHelper => include_str!("prompts/docs_helper.txt"),
        }
    }

    /// Get all agent types
    pub fn all() -> &'static [AgentType] {
        &[
            AgentType::AdminSetup,
            AgentType::UsageAssistant,
            AgentType::Troubleshoot,
            AgentType::BestPractices,
            AgentType::DocsHelper,
        ]
    }
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin-setup" | "admin_setup" | "adminsetup" => Ok(AgentType::AdminSetup),
            "usage" | "usage-assistant" | "usage_assistant" => Ok(AgentType::UsageAssistant),
            "troubleshoot" | "troubleshooting" => Ok(AgentType::Troubleshoot),
            "best-practices" | "best_practices" | "bestpractices" => Ok(AgentType::BestPractices),
            "docs" | "docs-helper" | "docs_helper" | "documentation" => Ok(AgentType::DocsHelper),
            _ => Err(format!("Unknown agent type: {}", s)),
        }
    }
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    pub id: Uuid,
    /// Role of the message sender
    pub role: MessageRole,
    /// Content of the message
    pub content: String,
    /// When the message was created
    pub timestamp: DateTime<Utc>,
    /// Optional tool call ID (for tool responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Optional tool name (for tool calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
}

impl Message {
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: MessageRole::User,
            content: content.into(),
            timestamp: Utc::now(),
            tool_call_id: None,
            tool_name: None,
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: MessageRole::Assistant,
            content: content.into(),
            timestamp: Utc::now(),
            tool_call_id: None,
            tool_name: None,
        }
    }

    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: MessageRole::System,
            content: content.into(),
            timestamp: Utc::now(),
            tool_call_id: None,
            tool_name: None,
        }
    }

    /// Create a new tool result message
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: MessageRole::Tool,
            content: content.into(),
            timestamp: Utc::now(),
            tool_call_id: Some(tool_call_id.into()),
            tool_name: None,
        }
    }
}

/// Role of a message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System instructions
    System,
    /// User input
    User,
    /// Assistant response
    Assistant,
    /// Tool result
    Tool,
}

/// Context about the user's WatsonX Orchestrate environment
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WxoContext {
    /// User's role (admin, developer, end-user)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_role: Option<String>,

    /// Current topic being discussed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_topic: Option<String>,

    /// Relevant documentation sections found
    #[serde(default)]
    pub relevant_docs: Vec<DocReference>,

    /// User's WXO version (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wxo_version: Option<String>,

    /// User's deployment type (SaaS, on-prem, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_type: Option<String>,

    /// Custom metadata
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

/// Reference to a documentation section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocReference {
    /// Document title
    pub title: String,
    /// Document URL or path
    pub url: String,
    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,
    /// Brief excerpt from the document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
}

/// Main state type for WXOrca agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WxorcaState {
    /// Unique session identifier
    pub session_id: String,

    /// The type of agent handling this conversation
    pub agent_type: AgentType,

    /// Conversation messages
    pub messages: Vec<Message>,

    /// WatsonX Orchestrate-specific context
    pub context: WxoContext,

    /// Current iteration count (for loop detection)
    pub iteration: usize,

    /// Whether the conversation is complete
    pub is_complete: bool,

    /// Pending tool calls to execute
    #[serde(default)]
    pub pending_tool_calls: Vec<PendingToolCall>,

    /// When this state was created
    pub created_at: DateTime<Utc>,

    /// When this state was last updated
    pub updated_at: DateTime<Utc>,
}

/// A pending tool call to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingToolCall {
    /// Unique ID for this tool call
    pub id: String,
    /// Name of the tool to call
    pub name: String,
    /// Arguments to pass to the tool
    pub arguments: serde_json::Value,
}

impl Default for WxorcaState {
    fn default() -> Self {
        Self::new(AgentType::default())
    }
}

impl WxorcaState {
    /// Create a new state for the given agent type
    pub fn new(agent_type: AgentType) -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4().to_string(),
            agent_type,
            messages: Vec::new(),
            context: WxoContext::default(),
            iteration: 0,
            is_complete: false,
            pending_tool_calls: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new state with a specific session ID
    pub fn with_session_id(agent_type: AgentType, session_id: impl Into<String>) -> Self {
        let mut state = Self::new(agent_type);
        state.session_id = session_id.into();
        state
    }

    /// Add a user message to the conversation
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message::user(content));
        self.updated_at = Utc::now();
    }

    /// Add an assistant message to the conversation
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message::assistant(content));
        self.updated_at = Utc::now();
    }

    /// Add a tool result to the conversation
    pub fn add_tool_result(&mut self, tool_call_id: impl Into<String>, result: impl Into<String>) {
        self.messages.push(Message::tool_result(tool_call_id, result));
        self.updated_at = Utc::now();
    }

    /// Get the last user message
    pub fn last_user_message(&self) -> Option<&Message> {
        self.messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
    }

    /// Get the last assistant message
    pub fn last_assistant_message(&self) -> Option<&Message> {
        self.messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::Assistant)
    }

    /// Check if there are pending tool calls
    pub fn has_pending_tool_calls(&self) -> bool {
        !self.pending_tool_calls.is_empty()
    }

    /// Clear pending tool calls
    pub fn clear_tool_calls(&mut self) {
        self.pending_tool_calls.clear();
        self.updated_at = Utc::now();
    }

    /// Add a pending tool call
    pub fn add_tool_call(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        arguments: serde_json::Value,
    ) {
        self.pending_tool_calls.push(PendingToolCall {
            id: id.into(),
            name: name.into(),
            arguments,
        });
        self.updated_at = Utc::now();
    }

    /// Mark the conversation as complete
    pub fn mark_complete(&mut self) {
        self.is_complete = true;
        self.updated_at = Utc::now();
    }

    /// Increment the iteration counter
    pub fn increment_iteration(&mut self) {
        self.iteration += 1;
        self.updated_at = Utc::now();
    }

    /// Set a context metadata value
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.context.metadata.insert(key.into(), value.into());
        self.updated_at = Utc::now();
    }

    /// Get a context metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.context.metadata.get(key)
    }
}

// Implement the State trait from oxidizedgraph
impl State for WxorcaState {
    fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" },
                "agent_type": { "type": "string" },
                "messages": {
                    "type": "array",
                    "items": { "type": "object" }
                },
                "context": { "type": "object" },
                "iteration": { "type": "integer" },
                "is_complete": { "type": "boolean" }
            },
            "required": ["session_id", "agent_type"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_from_str() {
        assert_eq!(
            "admin-setup".parse::<AgentType>().unwrap(),
            AgentType::AdminSetup
        );
        assert_eq!(
            "troubleshoot".parse::<AgentType>().unwrap(),
            AgentType::Troubleshoot
        );
        assert_eq!("docs".parse::<AgentType>().unwrap(), AgentType::DocsHelper);
    }

    #[test]
    fn test_state_messages() {
        let mut state = WxorcaState::new(AgentType::UsageAssistant);
        state.add_user_message("Hello");
        state.add_assistant_message("Hi there!");

        assert_eq!(state.messages.len(), 2);
        assert_eq!(state.last_user_message().unwrap().content, "Hello");
        assert_eq!(state.last_assistant_message().unwrap().content, "Hi there!");
    }

    #[test]
    fn test_state_tool_calls() {
        let mut state = WxorcaState::default();
        assert!(!state.has_pending_tool_calls());

        state.add_tool_call("call_1", "search_docs", serde_json::json!({"query": "setup"}));
        assert!(state.has_pending_tool_calls());

        state.clear_tool_calls();
        assert!(!state.has_pending_tool_calls());
    }
}
