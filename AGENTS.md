# hqe-workbench Context
> **Auto-Generated:** 2026-01-29 | **Sources:** `Cargo.toml`, `apps/workbench/package.json`, `scripts/dev.sh`, `apps/workbench/vite.config.ts`

## 1. System Identity
| Attribute | Value | Source |
| :--- | :--- | :--- |
| **Language** | TypeScript 5.3.0, Rust 1.75 | `apps/workbench/package.json`, `Cargo.toml` |
| **Runtime** | Node.js (V8), Rust | `apps/workbench/package.json`, `Cargo.toml` |
| **Framework** | React 18.2.0, Tauri 2.0.0 | `apps/workbench/package.json` |
| **Builder** | Vite 5.0.0, Cargo | `apps/workbench/package.json`, `Cargo.toml` |

## 2. Operational Commands
*Execute these exactly as shown.*
| Intent | Command | Context/Notes | Source |
| :--- | :--- | :--- | :--- |
| **Install** | `./scripts/bootstrap_macos.sh` | - | `scripts/bootstrap_macos.sh` |
| **Dev** | `./scripts/dev.sh` | Port 1420 | `scripts/dev.sh`, `apps/workbench/vite.config.ts` |
| **Build** | `./scripts/build_dmg.sh` | Outputs to `target/release/bundle` | `scripts/build_dmg.sh` |
| **Test** | `cargo test --workspace` | Rust Unit/Integration | `scripts/build_dmg.sh` |
| **Test (UI)** | `npm run test` | Vitest | `apps/workbench/package.json` |
| **Lint** | `npm run lint` | ESLint (TS/TSX) | `apps/workbench/package.json` |
| **Validate** | `./scripts/validate_protocol.sh` | Protocol Schema Validation | `scripts/dev.sh` |

## 3. Architecture & Conventions
- **Entry Point (UI):** `apps/workbench/src/main.tsx` (React Root)
- **Entry Point (CLI):** `cli/hqe/src/main.rs` (Rust Binary)
- **State Management:** `zustand` (via `apps/workbench/package.json`)
- **Styling:** `tailwindcss` (via `apps/workbench/package.json`)
- **Testing Strategy:** `vitest` (Frontend), `cargo test` (Backend)
- **Routing:** `react-router-dom` (via `apps/workbench/package.json`)

## 4. Environment Variables (.env)
*Derived from `.env.example` or config validation schemas.*
| Variable | Required | Description |
| :--- | :--- | :--- |
| - | - | No explicit .env configuration found |

## 5. Critical Files Map
- `Cargo.toml`: Workspace configuration and dependency management
- `apps/workbench/vite.config.ts`: Frontend build and dev server configuration
- `protocol/hqe-engineer.yaml`: Core protocol definition
- `cli/hqe/Cargo.toml`: CLI crate configuration