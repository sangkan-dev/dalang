# Web Server Architecture

Dalang's web UI is a single-binary full-stack application: an axum HTTP server serves an embedded SvelteKit app artifact (`web2/build`) and exposes REST + WebSocket APIs.

## High-Level Diagram

```
┌─────────────────────────────────────────────────┐
│                  Rust Binary                     │
│                                                  │
│  ┌────────────────┐   ┌──────────────────────┐  │
│  │  axum Router    │   │  rust-embed          │  │
│  │  /api/*  REST   │   │  web2/build/*        │  │
│  │  /api/ws  WS    │   │  (SvelteKit static)  │  │
│  └───────┬────────┘   └──────────┬───────────┘  │
│          │                       │               │
│          ▼                       ▼               │
│  ┌────────────┐         ┌──────────────┐        │
│  │  Handlers  │         │  Static File │        │
│  │  sessions  │         │  Fallback    │        │
│  │  skills    │         │  (SPA index) │        │
│  │  reports   │         └──────────────┘        │
│  │  settings  │                                  │
│  │  chat (WS) │                                  │
│  └─────┬──────┘                                  │
│        │                                         │
│        ▼                                         │
│  ┌───────────────────────────────────────────┐  │
│  │           AppState (shared)                │  │
│  │  sessions:       DashMap<Uuid, Session>    │  │
│  │  event_senders:  DashMap<Uuid, (Uuid, Tx)> │  │
│  │  disabled_skills: DashMap<String, bool>    │  │
│  └───────────────────────────────────────────┘  │
│        │                                         │
│        ▼                                         │
│  ┌───────────────────────────────────────────┐  │
│  │  ~/.dalang/sessions/<id>/                  │  │
│  │    session.json  events.json  MEMORY.md    │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

## REST API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/sessions` | List all sessions (with metadata + event count) |
| `POST` | `/api/sessions` | Create a new session |
| `GET` | `/api/sessions/{id}/messages` | Get messages for a session |
| `GET` | `/api/sessions/{id}/events` | Replay all engine events for a session |
| `DELETE` | `/api/sessions/{id}` | Delete a session (and its persisted files) |
| `GET` | `/api/ws/{session_id}` | WebSocket upgrade for real-time chat |
| `GET` | `/api/skills` | List all skills (with enabled/available status) |
| `GET` | `/api/skills/{name}` | Get skill detail |
| `PUT` | `/api/skills/{name}` | Toggle skill enabled/disabled |
| `GET` | `/api/reports` | List saved reports |
| `GET` | `/api/reports/{filename}` | Get report content (supports `?format=html`) |
| `GET` | `/api/settings` | Get current settings |
| `PUT` | `/api/settings` | Update settings |
| `POST` | `/api/settings/test-connection` | Test LLM connection |

## WebSocket Protocol

The WebSocket endpoint (`/api/ws/{session_id}`) uses JSON messages in both directions. Each connection is tracked with a unique `conn_id` to prevent cleanup races when multiple connections target the same session.

### Client → Server

```json
{ "type": "chat", "message": "scan the target" }
{ "type": "start_scan", "target": "https://example.com", "max_iter": 20, "cmd_timeout": 300 }
{ "type": "start_interactive", "target": "https://example.com", "cmd_timeout": 300 }
```

### Server → Client (EngineEvent)

```json
{ "type": "thinking", "iteration": 1, "max_iter": 20 }
{ "type": "assistant_message", "content": "...", "done": false }
{ "type": "tool_execution", "skill": "nmap_scanner", "command": "nmap -sV ..." }
{ "type": "observation", "skill": "nmap_scanner", "content": "...", "bytes": 1234 }
{ "type": "browser_action", "action": "browser-navigate", "success": true, "content": "Navigated to ..." }
{ "type": "safety_refusal", "retry": 1 }
{ "type": "report", "markdown": "# Report\n...", "filename": "report_20250101.md" }
{ "type": "status", "message": "..." }
{ "type": "error", "message": "..." }
{ "type": "done", "reason": "max_iterations" }
```

### Connection Tracking

Each WebSocket connection generates a unique `conn_id` (UUID). The `event_senders` map stores `(conn_id, Sender)` tuples. On disconnect, only the connection's own sender is removed (via `remove_if`), preventing a race where reconnecting clients would have their new sender deleted by the old connection's cleanup.

## Shared State

`AppState` is an `Arc`-wrapped struct shared across all handlers:

| Field | Type | Purpose |
|-------|------|---------|
| `sessions` | `DashMap<Uuid, Session>` | Concurrent session storage |
| `event_senders` | `DashMap<Uuid, (Uuid, mpsc::Sender<EngineEvent>)>` | Per-session event channels with connection tracking |
| `disabled_skills` | `DashMap<String, bool>` | Runtime skill toggle state |
| `verbose` | `bool` | Verbose mode flag |

## Session Persistence

Sessions are persisted to `~/.dalang/sessions/<session-id>/`:

- **`session.json`** — Metadata: id, target, mode, created_at, updated_at, active flag
- **`events.json`** — Ordered array of all `EngineEvent`s emitted during the session
- **`MEMORY.md`** — Human-readable audit log with findings and observations

On startup, the server loads all persisted sessions from disk. When the Web UI loads an existing session, events are replayed from `events.json` to reconstruct the conversation.

## Frontend Stack

| Layer | Technology |
|-------|-----------|
| Framework | Svelte 5.34 (runes mode) |
| Styling | Tailwind CSS 4 |
| Build | Vite 6.4 |
| Language | TypeScript 5.8 |
| Markdown | marked 15 + highlight.js 11 |
| Testing | vitest 3.2 + jsdom 26 |

The compiled frontend (`web2/build/`) is embedded into the Rust binary via `rust-embed`, so `dalang web` works as a single self-contained binary with zero external dependencies for serving.

## Route And Runtime Contract (Sprint 31)

This section defines the current source-of-truth route ownership and deployment behavior.

### Route Ownership

| Route Pattern | Owner | Notes |
|---|---|---|
| `/` | SvelteKit landing | Public-facing marketing and product entry route. |
| `/dashboard/*` | SvelteKit dashboard app | Main operational interface. Future feature routes (chat, skills, reports, settings) live here. |
| `/api/*` | Rust axum backend | REST API surface for sessions, skills, reports, and settings. |
| `/api/ws/{session_id}` | Rust axum backend | Real-time engine event stream for interactive and autonomous runs. |

### Runtime Strategy

| Context | Frontend Delivery | Backend Delivery |
|---|---|---|
| Local development | `npm run dev` from `web2/` (Vite) | `cargo run -- web --port <port>` |
| Production single binary | Embedded static assets from `web2/build/` | Same Rust binary serves both static files and APIs |

### Invariants

1. Frontend build output path is `web2/build/`.
2. Rust embed folder must remain `web2/build/` (`src/adapters/inbound/web/embedded.rs`).
3. CI/release pipelines must build frontend from `web2/` before `cargo build`.
4. API and WebSocket namespaces stay under `/api` to avoid route collisions with SvelteKit pages.

### WebSocket Reconnection

The frontend WebSocket client includes automatic reconnection:

- **Max attempts**: 5
- **Backoff**: Exponential (1s, 2s, 4s, 8s, 16s)
- **Visual feedback**: Toast notifications for reconnecting/reconnected states
- **Guard**: `loadedSessionId` tracking prevents duplicate connections from Svelte `$effect` reactivity

## Key Design Decisions

1. **DashMap over Mutex** — lock-free concurrent access for session state.
2. **mpsc channels for WS** — engine tasks push events via channel; the WS handler forwards them. Clean separation between computation and I/O.
3. **Connection-tracked senders** — `(conn_id, Sender)` tuples prevent reconnection races where old cleanup removes new connections.
4. **rust-embed** — zero-copy static serving from the binary. SPA fallback to `index.html` for client-side routing.
5. **File-based persistence** — sessions survive server restarts. JSON format for events enables full replay.
6. **Keyring persistence** — API keys and settings stored in the OS keyring (via `keyring` crate), not in plain-text config files.
