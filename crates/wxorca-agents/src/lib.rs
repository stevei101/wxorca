//! WXOrca Agents - AI-powered guides for IBM WatsonX Orchestrate
//!
//! This crate provides specialized agents for helping users with:
//! - Admin setup and configuration
//! - Usage assistance and workflow guidance
//! - Troubleshooting common issues
//! - Best practices and optimization
//! - Documentation navigation

pub mod agents;
pub mod db;
pub mod state;
pub mod tools;

pub use agents::{
    AdminSetupAgent, BestPracticesAgent, DocsHelperAgent, TroubleshootAgent, UsageAssistantAgent,
};
pub use db::Database;
pub use state::{AgentType, Message, WxoContext, WxorcaState};

/// Re-exports from oxidizedgraph for convenience
pub mod prelude {
    pub use oxidizedgraph::prelude::*;

    pub use crate::agents::*;
    pub use crate::db::Database;
    pub use crate::state::*;
    pub use crate::tools::*;
}
