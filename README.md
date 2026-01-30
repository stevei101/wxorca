# WXOrca - WatsonX Orchestrate Explorer

🐳 An AI-powered smart guide for IBM WatsonX Orchestrate, built with [oxidizedgraph](https://github.com/stevedores-org/oxidizedgraph), SurrealDB, and a modern React + Bun stack.

## Overview

WXOrca provides specialized AI agents to help users with different aspects of IBM WatsonX Orchestrate:

| Agent | Description |
|-------|-------------|
| ⚙️ **Admin Setup Guide** | Guides administrators through setup and configuration |
| 💡 **Usage Assistant** | Helps users understand features and workflows |
| 🔧 **Troubleshooting Bot** | Diagnoses and resolves common issues |
| 🏆 **Best Practices Coach** | Provides optimization tips and recommendations |
| 📚 **Documentation Helper** | Navigates and explains WXO documentation |

## Architecture

```
wxorca/
├── crates/
│   └── wxorca-agents/        # Rust agent implementation
│       ├── src/
│       │   ├── agents/       # 5 specialized agents
│       │   ├── tools/        # Search, validate, fetch tools
│       │   ├── state.rs      # Agent state management
│       │   └── db.rs         # SurrealDB integration
│       └── Cargo.toml
├── backend/                   # Bun + Elysia API server
│   └── src/
│       ├── routes/           # API endpoints
│       └── services/         # Rust bridge
├── frontend/                  # React 19 + Vite + Tailwind
│   └── src/
│       ├── components/       # Chat UI components
│       └── services/         # API client
└── README.md
```

## Prerequisites

- **Rust** (1.75+) with cargo
- **Bun** (1.0+)
- **SurrealDB** (optional, for persistence)
- **Node.js** (18+, optional - Bun is preferred)

## Quick Start

### 1. Build the Rust agents

```bash
cd wxorca
cargo build --release
```

### 2. Start SurrealDB (optional)

```bash
surreal start memory
```

### 3. Start the backend

```bash
cd backend
bun install
bun run dev
```

### 4. Start the frontend

```bash
cd frontend
bun install
bun run dev
```

### 5. Open the app

Navigate to http://localhost:5173 in your browser.

## Development

### Rust Agents

The agents are built using [oxidizedgraph](https://github.com/stevedores-org/oxidizedgraph), a high-performance Rust implementation of LangGraph.

```rust
// Example: Building an agent graph
let graph = GraphBuilder::new()
    .add_node(AnalyzeQueryNode::new("analyze"))
    .add_node(SearchDocsNode::new("search"))
    .add_node(RespondNode::new("respond"))
    .set_entry_point("analyze")
    .add_edge("analyze", "search")
    .add_edge("search", "respond")
    .compile()?;
```

### Backend API

The backend uses [Elysia](https://elysiajs.com/) on Bun for high-performance API routing:

```typescript
// POST /api/agents/chat
{
  "sessionId": "session_123",
  "agentType": "admin-setup",
  "message": "How do I configure SSO?"
}
```

### Frontend

The frontend is built with React 19 and Tailwind CSS 4, following IBM's design language.

## Available Tools

The agents have access to specialized tools:

| Tool | Description |
|------|-------------|
| `search_wxo_docs` | Search WatsonX Orchestrate documentation |
| `validate_wxo_config` | Validate skill/workflow configurations |
| `fetch_wxo_examples` | Fetch code examples and samples |

## Configuration

### Environment Variables

```bash
# Backend
SURREAL_HOST=localhost
SURREAL_PORT=8000
SURREAL_USER=root
SURREAL_PASS=root
SURREAL_NS=wxorca
SURREAL_DB=main

# Frontend
VITE_API_URL=http://localhost:3000
```

## Deployment

### Building OCI Images with Nix

```bash
# Enter dev shell
nix develop

# Build backend image
nix build .#backend-image

# Build frontend image
nix build .#frontend-image

# Load images into Docker
docker load < result
```

### Push to Google Artifact Registry

```bash
# Authenticate
gcloud auth configure-docker us-central1-docker.pkg.dev

# Tag and push
docker tag wxorca-backend:latest us-central1-docker.pkg.dev/gcp-lornu-ai/cloud-run-source-deploy/wxorca-backend:latest
docker push us-central1-docker.pkg.dev/gcp-lornu-ai/cloud-run-source-deploy/wxorca-backend:latest
```

### GKE Deployment

WXOrca is deployed to GKE using [crossplane-heaven](https://github.com/stevedores-org/crossplane-heaven) with Flux GitOps:

- **Frontend**: https://wxorca.liteworks.media
- **API**: https://api.wxorca.liteworks.media

The deployment includes:
- 2 replicas of frontend (static-web-server)
- 2 replicas of backend (Bun + Rust agents)
- SurrealDB for persistence
- TLS via cert-manager + Let's Encrypt

## Roadmap

- [ ] Real LLM integration (Anthropic Claude, OpenAI)
- [ ] Vector embeddings for RAG
- [ ] Persistent conversation history
- [ ] User authentication
- [x] Nix OCI images for deployment
- [x] CI/CD pipeline

## License

MIT

## Built With

- [oxidizedgraph](https://github.com/stevedores-org/oxidizedgraph) - Rust graph execution engine
- [SurrealDB](https://surrealdb.com/) - Multi-model database
- [Elysia](https://elysiajs.com/) - Bun web framework
- [React](https://react.dev/) - UI framework
- [Tailwind CSS](https://tailwindcss.com/) - Styling
