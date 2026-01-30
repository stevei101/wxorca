//! WXOrca CLI - Command line interface for WXOrca agents
//!
//! Provides a CLI interface for interacting with WXOrca agents.
//! Used by the backend server via subprocess communication.

use anyhow::Result;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use tracing_subscriber::EnvFilter;
use wxorca_agents::prelude::*;

#[derive(Parser)]
#[command(name = "wxorca-cli")]
#[command(about = "WXOrca - AI-powered guide for IBM WatsonX Orchestrate")]
struct Cli {
    /// The type of agent to use
    #[arg(short, long)]
    agent: AgentTypeArg,

    /// Session ID for conversation persistence
    #[arg(short, long)]
    session: Option<String>,

    /// Single message to process (if not provided, enters interactive mode)
    #[arg(short, long)]
    message: Option<String>,

    /// Output format
    #[arg(short, long, default_value = "json")]
    format: OutputFormat,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, ValueEnum)]
enum AgentTypeArg {
    AdminSetup,
    Usage,
    Troubleshoot,
    BestPractices,
    Docs,
}

impl From<AgentTypeArg> for AgentType {
    fn from(arg: AgentTypeArg) -> Self {
        match arg {
            AgentTypeArg::AdminSetup => AgentType::AdminSetup,
            AgentTypeArg::Usage => AgentType::UsageAssistant,
            AgentTypeArg::Troubleshoot => AgentType::Troubleshoot,
            AgentTypeArg::BestPractices => AgentType::BestPractices,
            AgentTypeArg::Docs => AgentType::DocsHelper,
        }
    }
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Text,
}

#[derive(Serialize)]
struct AgentResponse {
    session_id: String,
    agent_type: String,
    response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Deserialize)]
struct InputMessage {
    message: String,
    #[serde(default)]
    session_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env().add_directive("wxorca=debug".parse()?))
            .init();
    }

    let agent_type: AgentType = cli.agent.into();

    if let Some(message) = cli.message {
        // Single message mode
        let response = process_message(&agent_type, cli.session.as_deref(), &message).await?;
        output_response(&response, &cli.format)?;
    } else {
        // Interactive mode (read from stdin)
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse as JSON, otherwise use as plain text
            let (message, session_id) = if let Ok(input) = serde_json::from_str::<InputMessage>(&line)
            {
                (input.message, input.session_id)
            } else {
                (line, cli.session.clone())
            };

            let response = process_message(&agent_type, session_id.as_deref(), &message).await?;
            output_response(&response, &cli.format)?;
            stdout.flush()?;
        }
    }

    Ok(())
}

async fn process_message(
    agent_type: &AgentType,
    session_id: Option<&str>,
    message: &str,
) -> Result<AgentResponse> {
    // Build the agent graph
    let graph = match build_agent_graph(*agent_type) {
        Ok(g) => g,
        Err(e) => {
            return Ok(AgentResponse {
                session_id: session_id.unwrap_or("").to_string(),
                agent_type: agent_type.to_string(),
                response: String::new(),
                error: Some(format!("Failed to build agent graph: {}", e)),
            });
        }
    };

    // Create or restore state
    let mut state = if let Some(sid) = session_id {
        WxorcaState::with_session_id(*agent_type, sid)
    } else {
        WxorcaState::new(*agent_type)
    };

    // Add the user message
    state.add_user_message(message);

    // Convert to AgentState for the runner
    let agent_state = convert_to_agent_state(&state);

    // Run the graph
    let runner = GraphRunner::new(
        graph,
        RunnerConfig::default()
            .max_iterations(10)
            .verbose(false),
    );

    match runner.invoke(agent_state).await {
        Ok(result_state) => {
            // Extract the assistant's response
            let response = result_state
                .last_assistant_message()
                .map(|m| m.content.clone())
                .unwrap_or_else(|| "I apologize, but I couldn't generate a response.".to_string());

            Ok(AgentResponse {
                session_id: state.session_id.clone(),
                agent_type: agent_type.to_string(),
                response,
                error: None,
            })
        }
        Err(e) => Ok(AgentResponse {
            session_id: state.session_id.clone(),
            agent_type: agent_type.to_string(),
            response: String::new(),
            error: Some(format!("Agent execution failed: {}", e)),
        }),
    }
}

fn convert_to_agent_state(wxorca_state: &WxorcaState) -> AgentState {
    // Use with_system_and_user if we have a user message, otherwise just create with system
    let system_prompt = wxorca_state.agent_type.system_prompt();

    let mut agent_state = if let Some(first_user_msg) = wxorca_state.messages.iter().find(|m| m.role == MessageRole::User) {
        AgentState::with_system_and_user(system_prompt, &first_user_msg.content)
    } else {
        let mut state = AgentState::new();
        state.messages.push(oxidizedgraph::prelude::Message::system(system_prompt));
        state
    };

    // Add remaining messages (skip the first user message as it's already added)
    let mut skip_first_user = true;
    for msg in &wxorca_state.messages {
        match msg.role {
            MessageRole::User => {
                if skip_first_user {
                    skip_first_user = false;
                    continue;
                }
                agent_state.add_user_message(&msg.content);
            }
            MessageRole::Assistant => agent_state.add_assistant_message(&msg.content),
            MessageRole::System => {
                // System messages are added via the initial state
                agent_state.messages.push(oxidizedgraph::prelude::Message::system(&msg.content));
            }
            MessageRole::Tool => {
                if let Some(ref tool_call_id) = msg.tool_call_id {
                    agent_state.add_tool_result(tool_call_id, &msg.content);
                }
            }
        }
    }

    // Set context
    agent_state.set_context("agent_type", serde_json::json!(wxorca_state.agent_type));
    agent_state.set_context("session_id", serde_json::json!(wxorca_state.session_id));

    agent_state
}

fn output_response(response: &AgentResponse, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(response)?);
        }
        OutputFormat::Text => {
            if let Some(ref error) = response.error {
                eprintln!("Error: {}", error);
            } else {
                println!("{}", response.response);
            }
        }
    }
    Ok(())
}
