//! SurrealDB integration for WXOrca
//!
//! Provides persistence for conversations, documentation embeddings,
//! and user feedback.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};

use crate::state::{AgentType, Message, WxorcaState};

/// Database client wrapper for WXOrca
#[derive(Clone)]
pub struct Database {
    client: Surreal<Client>,
}

/// A conversation record stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub session_id: String,
    pub agent_type: AgentType,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A documentation record for RAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub title: String,
    pub content: String,
    pub category: String,
    pub url: Option<String>,
    /// Embedding vector for similarity search
    #[serde(default)]
    pub embedding: Vec<f32>,
    pub created_at: DateTime<Utc>,
}

/// User feedback record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub session_id: String,
    pub message_id: Option<String>,
    pub rating: i32,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Configuration for database connection
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8000,
            username: "root".to_string(),
            password: "root".to_string(),
            namespace: "wxorca".to_string(),
            database: "main".to_string(),
        }
    }
}

impl DbConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("SURREAL_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("SURREAL_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8000),
            username: std::env::var("SURREAL_USER").unwrap_or_else(|_| "root".to_string()),
            password: std::env::var("SURREAL_PASS").unwrap_or_else(|_| "root".to_string()),
            namespace: std::env::var("SURREAL_NS").unwrap_or_else(|_| "wxorca".to_string()),
            database: std::env::var("SURREAL_DB").unwrap_or_else(|_| "main".to_string()),
        }
    }

    /// Get the connection URL
    pub fn url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl Database {
    /// Connect to SurrealDB with the given configuration
    pub async fn connect(config: &DbConfig) -> Result<Self> {
        let client = Surreal::new::<Ws>(&config.url())
            .await
            .context("Failed to connect to SurrealDB")?;

        client
            .signin(Root {
                username: &config.username,
                password: &config.password,
            })
            .await
            .context("Failed to authenticate with SurrealDB")?;

        client
            .use_ns(&config.namespace)
            .use_db(&config.database)
            .await
            .context("Failed to select namespace and database")?;

        Ok(Self { client })
    }

    /// Initialize the database schema
    pub async fn init_schema(&self) -> Result<()> {
        // Conversations table
        self.client
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS conversations SCHEMAFULL;
                DEFINE FIELD session_id ON conversations TYPE string;
                DEFINE FIELD agent_type ON conversations TYPE string;
                DEFINE FIELD messages ON conversations TYPE array;
                DEFINE FIELD created_at ON conversations TYPE datetime DEFAULT time::now();
                DEFINE FIELD updated_at ON conversations TYPE datetime DEFAULT time::now();
                DEFINE INDEX idx_session ON conversations FIELDS session_id UNIQUE;
                "#,
            )
            .await
            .context("Failed to create conversations table")?;

        // Documentation table for RAG
        self.client
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS wxo_docs SCHEMAFULL;
                DEFINE FIELD title ON wxo_docs TYPE string;
                DEFINE FIELD content ON wxo_docs TYPE string;
                DEFINE FIELD category ON wxo_docs TYPE string;
                DEFINE FIELD url ON wxo_docs TYPE option<string>;
                DEFINE FIELD embedding ON wxo_docs TYPE array DEFAULT [];
                DEFINE FIELD created_at ON wxo_docs TYPE datetime DEFAULT time::now();
                DEFINE INDEX idx_category ON wxo_docs FIELDS category;
                "#,
            )
            .await
            .context("Failed to create wxo_docs table")?;

        // Feedback table
        self.client
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS feedback SCHEMAFULL;
                DEFINE FIELD session_id ON feedback TYPE string;
                DEFINE FIELD message_id ON feedback TYPE option<string>;
                DEFINE FIELD rating ON feedback TYPE int;
                DEFINE FIELD comment ON feedback TYPE option<string>;
                DEFINE FIELD created_at ON feedback TYPE datetime DEFAULT time::now();
                DEFINE INDEX idx_feedback_session ON feedback FIELDS session_id;
                "#,
            )
            .await
            .context("Failed to create feedback table")?;

        Ok(())
    }

    // ==================== Conversation Operations ====================

    /// Save or update a conversation
    pub async fn save_conversation(&self, state: &WxorcaState) -> Result<()> {
        let record = ConversationRecord {
            id: None,
            session_id: state.session_id.clone(),
            agent_type: state.agent_type,
            messages: state.messages.clone(),
            created_at: state.created_at,
            updated_at: state.updated_at,
        };

        // Upsert based on session_id
        self.client
            .query(
                r#"
                UPDATE conversations SET
                    agent_type = $agent_type,
                    messages = $messages,
                    updated_at = time::now()
                WHERE session_id = $session_id;

                IF (SELECT * FROM conversations WHERE session_id = $session_id).len() == 0 {
                    CREATE conversations SET
                        session_id = $session_id,
                        agent_type = $agent_type,
                        messages = $messages,
                        created_at = $created_at,
                        updated_at = time::now()
                };
                "#,
            )
            .bind(("session_id", &record.session_id))
            .bind(("agent_type", serde_json::to_string(&record.agent_type)?))
            .bind(("messages", &record.messages))
            .bind(("created_at", record.created_at))
            .await
            .context("Failed to save conversation")?;

        Ok(())
    }

    /// Load a conversation by session ID
    pub async fn load_conversation(&self, session_id: &str) -> Result<Option<WxorcaState>> {
        let mut result = self
            .client
            .query("SELECT * FROM conversations WHERE session_id = $session_id")
            .bind(("session_id", session_id))
            .await
            .context("Failed to query conversation")?;

        let records: Vec<ConversationRecord> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            let mut state = WxorcaState::with_session_id(record.agent_type, record.session_id);
            state.messages = record.messages;
            state.created_at = record.created_at;
            state.updated_at = record.updated_at;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    /// Delete a conversation
    pub async fn delete_conversation(&self, session_id: &str) -> Result<()> {
        self.client
            .query("DELETE FROM conversations WHERE session_id = $session_id")
            .bind(("session_id", session_id))
            .await
            .context("Failed to delete conversation")?;

        Ok(())
    }

    /// List recent conversations
    pub async fn list_conversations(&self, limit: usize) -> Result<Vec<ConversationRecord>> {
        let mut result = self
            .client
            .query("SELECT * FROM conversations ORDER BY updated_at DESC LIMIT $limit")
            .bind(("limit", limit))
            .await
            .context("Failed to list conversations")?;

        let records: Vec<ConversationRecord> = result.take(0)?;
        Ok(records)
    }

    // ==================== Documentation Operations ====================

    /// Add a documentation record
    pub async fn add_doc(&self, doc: &DocRecord) -> Result<Thing> {
        let created: Option<DocRecord> = self
            .client
            .create("wxo_docs")
            .content(doc)
            .await
            .context("Failed to add documentation")?;

        created
            .and_then(|d| d.id)
            .ok_or_else(|| anyhow::anyhow!("Failed to get created doc ID"))
    }

    /// Search documentation by text query (simple contains search)
    pub async fn search_docs(&self, query: &str, limit: usize) -> Result<Vec<DocRecord>> {
        let mut result = self
            .client
            .query(
                r#"
                SELECT * FROM wxo_docs
                WHERE content CONTAINS $query OR title CONTAINS $query
                LIMIT $limit
                "#,
            )
            .bind(("query", query))
            .bind(("limit", limit))
            .await
            .context("Failed to search documentation")?;

        let records: Vec<DocRecord> = result.take(0)?;
        Ok(records)
    }

    /// Search documentation by category
    pub async fn search_docs_by_category(
        &self,
        category: &str,
        limit: usize,
    ) -> Result<Vec<DocRecord>> {
        let mut result = self
            .client
            .query("SELECT * FROM wxo_docs WHERE category = $category LIMIT $limit")
            .bind(("category", category))
            .bind(("limit", limit))
            .await
            .context("Failed to search documentation by category")?;

        let records: Vec<DocRecord> = result.take(0)?;
        Ok(records)
    }

    /// Get all documentation categories
    pub async fn get_doc_categories(&self) -> Result<Vec<String>> {
        let mut result = self
            .client
            .query("SELECT DISTINCT category FROM wxo_docs")
            .await
            .context("Failed to get documentation categories")?;

        #[derive(Deserialize)]
        struct CategoryRow {
            category: String,
        }

        let rows: Vec<CategoryRow> = result.take(0)?;
        Ok(rows.into_iter().map(|r| r.category).collect())
    }

    // ==================== Feedback Operations ====================

    /// Submit user feedback
    pub async fn submit_feedback(&self, feedback: &FeedbackRecord) -> Result<()> {
        self.client
            .create::<Option<FeedbackRecord>>("feedback")
            .content(feedback)
            .await
            .context("Failed to submit feedback")?;

        Ok(())
    }

    /// Get feedback for a session
    pub async fn get_session_feedback(&self, session_id: &str) -> Result<Vec<FeedbackRecord>> {
        let mut result = self
            .client
            .query("SELECT * FROM feedback WHERE session_id = $session_id ORDER BY created_at DESC")
            .bind(("session_id", session_id))
            .await
            .context("Failed to get session feedback")?;

        let records: Vec<FeedbackRecord> = result.take(0)?;
        Ok(records)
    }

    /// Get average rating for an agent type
    pub async fn get_agent_rating(&self, agent_type: AgentType) -> Result<Option<f64>> {
        // First get all sessions for this agent type
        let mut result = self
            .client
            .query(
                r#"
                SELECT math::mean(rating) as avg_rating FROM feedback
                WHERE session_id IN (
                    SELECT session_id FROM conversations WHERE agent_type = $agent_type
                )
                "#,
            )
            .bind(("agent_type", serde_json::to_string(&agent_type)?))
            .await
            .context("Failed to get agent rating")?;

        #[derive(Deserialize)]
        struct AvgRow {
            avg_rating: Option<f64>,
        }

        let rows: Vec<AvgRow> = result.take(0)?;
        Ok(rows.into_iter().next().and_then(|r| r.avg_rating))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would require a running SurrealDB instance
    // These are placeholder tests for the type system

    #[test]
    fn test_db_config_default() {
        let config = DbConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8000);
        assert_eq!(config.namespace, "wxorca");
    }

    #[test]
    fn test_db_config_url() {
        let config = DbConfig::default();
        assert_eq!(config.url(), "localhost:8000");
    }
}
