# Tech Stack

## Core Kernel (Rust)
- **Language:** Rust 2021 Edition
- **Runtime:** Tokio (Async I/O)
- **Error Handling:** `anyhow` (App), `thiserror` (Lib)
- **Logging:** `tracing`

## Architecture: Modular Monolith
The application is structured as a cargo workspace with decoupled crates acting as "microservices" via function calls (and eventually NATS/channels).

### Crates
- **`hqe-protocol`**: Shared domain models (`Entity`, `TopicManifest`) and schemas.
- **`hqe-mcp`**: The Model Context Protocol orchestrator. Handles tool registration and routing.
- **`hqe-ingest`**: Data ingestion engine. Watches file systems and ingests data into the system.
- **`hqe-vector`**: Vector memory management (reserved for future vector-store integration).
- **`hqe-flow`**: Workflow runtime engine (reserved for future flow/agent runtimes).
- **`hqe-core`**: Core scan pipeline and shared models.

## Frontend (Tauri)
- **Framework:** Tauri v2
- **UI:** React 18.2 + TypeScript
- **Styling:** Tailwind CSS

## Data Storage
- **Relational:** SQLite (via `sqlx` - planned)
- **Vector:** Qdrant (planned)
- **Format:** JSONB / EAV for flexibility.

## Build System
- **Tool:** Cargo
- **CI:** GitHub Actions
