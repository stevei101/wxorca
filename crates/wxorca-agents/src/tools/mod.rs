//! Tools for WXOrca agents
//!
//! Provides specialized tools for searching documentation,
//! validating configurations, and fetching examples.

mod fetch_examples;
mod search_docs;
mod validate_config;

pub use fetch_examples::FetchExamplesTool;
pub use search_docs::SearchDocsTool;
pub use validate_config::ValidateConfigTool;

use oxidizedgraph::prelude::ToolRegistry;

/// Create a tool registry with all WXOrca tools
pub fn create_tool_registry() -> ToolRegistry {
    ToolRegistry::new()
        .register(SearchDocsTool::new())
        .register(ValidateConfigTool::new())
        .register(FetchExamplesTool::new())
}
