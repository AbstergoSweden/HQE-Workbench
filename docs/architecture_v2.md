# HQE Workbench v2 Architecture Strategy

## 1. Executive Summary & App Understanding
**Current State:** The HQE Workbench is a localized, high-assurance code auditing tool built on a robust Rust (Tauri) backend and React frontend. It features a pipeline for ingestion, redaction, and LLM-based analysis using OpenAI-compatible providers.
**Future Vision:** The goal is to evolve this into a **Universal MCP Host & Server Platform**. Instead of just "auditing code," the app will become a dynamic runtime environment ("MCP Server inside an app") that can load "Topics" (domains like Finance, Health, Legal, Coding) as modular extensions. It will act as a central nervous system for AI agents, providing them with tools, memory, and real-time data connectivity.

## 2. Core Microservice Responsibilities
To achieve the "MCP Server" vision within a desktop app, we will use a **Modular Monolith** pattern with **Sidecar Services** for isolation.

| Service | Responsibility |
| :--- | :--- |
| **Core Kernel (Rust)** | App lifecycle, window management, secure keystore (Keychain), and inter-service message bus. |
| **Ingestion Engine** | Topic-agnostic data loader. Watches folders, connects to APIs, and normalizes data into a common schema. |
| **Vector Memory** | (New) Local vector database (e.g., Qdrant/LanceDB) for RAG across all "Topics." |
| **MCP Orchestrator** | Acts as the "Server" for external agents and "Host" for internal tools. Routes tool calls to the appropriate plugin. |
| **Flow Runtime** | Executes multi-step workflows (defined in YAML/JSON) like those in `GENKIT.md`. |
| **Sync Service** | Manages WebSockets and Real-time subscriptions for live data updates. |

## 3. Scalable Data Models (Topic-Agnostic)
We will use a **Entity-Attribute-Value (EAV)** inspired schema or JSONB-heavy SQLite design to allow flexible schema definitions per Topic.

```typescript
// Core Entity (Polymorphic)
interface Entity {
  id: string;
  topic_id: string; // e.g., "financial_analysis", "code_audit"
  kind: string;     // e.g., "StockTicker", "PullRequest"
  data: Record<string, any>; // Flexible JSON payload
  vector_embedding?: number[]; // For semantic search
  created_at: number;
}

// Topic Definition (The "Plugin" Manifest)
interface TopicManifest {
  id: string;
  name: string;
  version: string;
  capabilities: {
    tools: MCPToolDefinition[];
    prompts: PromptTemplate[];
    flows: WorkflowDefinition[];
  };
  data_schema: JSONSchema; // Validation rules for 'data'
}
```

## 4. Real-Time Sync Mechanisms
*   **Internal (App <-> Backend):** Use **Tauri Events** (IPC) for sub-5ms latency updates to the UI.
*   **External (App <-> Cloud/Agents):**
    *   **Embedded NATS Server:** A lightweight message broker embedded in the Rust backend. Allows disparate plugins to publish/subscribe to events (e.g., `topic.finance.price_update`).
    *   **WebSockets:** Exposed by the app for external agents to connect and receive streams.

## 5. Cross-Platform Strategy
*   **Desktop (Primary):** Continue with **Tauri v2**. It offers the best performance-to-resource ratio.
*   **Mobile (iOS/Android):** Leverage **Tauri Mobile**. Since the core logic is Rust, 90% of the backend code (Ingestion, MCP Orchestrator) compiles directly to iOS/Android native libraries.
*   **Web/Cloud:** The "Microservices" design allows packaging the Rust Core as a Docker container, replacing the Tauri Window binding with a REST/GraphQL server binding.

## 6. API Design
**API-First Approach:** The UI is just one consumer of the backend API.

*   **GraphQL (Frontend API):** Best for fetching complex, nested topic data.
*   **MCP Protocol (Agent API):** Native support for the Model Context Protocol.
*   **REST (Ingestion/Webhooks):** High-throughput endpoint for pushing data.

## 7. Performance Optimization
*   **Database:** Use **SQLite** with WAL mode and memory mapping.
*   **Caching:** Implement a multi-layer cache (L1: Memory/Moka, L2: SQLite).
*   **Concurrency:** Use Rust's `tokio` runtime.

## 8. Security Hardening
*   **Sandboxing:** "Topics" that require custom code execution run inside a **WebAssembly (Wasm)** runtime (wasmtime).
*   **Threat Modeling:** Assume "Topics" are untrusted. Verify all inputs against the `TopicManifest` schema.
*   **Auth:** OAuth2 for user login. API Keys for external agents.

## 9. Observability Stack
*   **Local Telemetry:** Embed a lightweight **OpenTelemetry** collector.
*   **Dashboard:** A hidden "Admin" view in the app.

## 10. Proposed Tech Stack
*   **Language:** Rust (Core/Backend), TypeScript (UI/Scripting).
*   **Framework:** Tauri v2 (App Shell), React 19 (UI).
*   **Database:** SQLite (Relational), Qdrant (Vector - Embedded).
*   **Message Bus:** NATS (Embedded) or Tokio Channels (Local).
*   **AI Orchestration:** Google Genkit (Flows) + custom Rust MCP implementation.
*   **Runtime:** Wasmtime (for secure plugin execution).
