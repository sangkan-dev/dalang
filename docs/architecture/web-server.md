# Web Server Architecture

Dalang's web UI is a single-binary full-stack application: an axum HTTP server serves an embedded Svelte 5 SPA and exposes REST + WebSocket APIs.

## High-Level Diagram

```
┌─────────────────────────────────────────────────┐
│                  Rust Binary                     │
│                                                  │
│  ┌────────────────┐   ┌──────────────────────┐  │
│  │  axum Router    │   │  rust-embed          │  │
│  │  /api/*  REST   │   │  web/dist/*          │  │
│  │  /api/ws  WS    │   │  (Svelte SPA)        │  │
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
│  │  event_senders:  DashMap<Uuid, mpsc::Tx>   │  │
│  │  disabled_skills: DashMap<String, bool>    │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

## REST API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/sessions` | List all sessions |
| `POST` | `/api/sessions` | Create a new session |
| `GET` | `/api/sessions/{id}/messages` | Get messages for a session |
| `DELETE` | `/api/sessions/{id}` | Delete a session |
| `GET` | `/api/ws/{session_id}` | WebSocket upgrade for chat |
| `GET` | `/api/skills` | List all skills (with enabled status) |
| `GET` | `/api/skills/{name}` | Get skill detail |
| `PUT` | `/api/skills/{name}` | Toggle skill enabled/disabled |
| `GET` | `/api/reports` | List saved reports |
| `GET` | `/api/reports/{filename}` | Get report content |
| `GET` | `/api/settings` | Get current settings |
| `PUT` | `/api/settings` | Update settings |
| `POST` | `/api/settings/test-connection` | Test LLM connection |

## WebSocket Protocol

The WebSocket endpoint (`/api/ws/{session_id}`) uses JSON messages in both directions.

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
{ "type": "report", "markdown": "# Report\n...", "filename": "report_20250101.md" }
{ "type": "error", "message": "..." }
{ "type": "done", "reason": "max_iterations" }
```

## Shared State

`AppState` is an `Arc`-wrapped struct shared across all handlers:

- **`sessions`** — `DashMap<Uuid, Session>`: concurrent session storage.
- **`event_senders`** — `DashMap<Uuid, mpsc::Sender<EngineEvent>>`: channels for pushing engine events to WebSocket connections.
- **`disabled_skills`** — `DashMap<String, bool>`: runtime skill toggle state.
- **`verbose`** — `bool`: verbose mode flag.

## Frontend Stack

| Layer | Technology |
|-------|-----------|
| Framework | Svelte 5.34 (runes mode) |
| Styling | Tailwind CSS 4 |
| Build | Vite 6.4 |
| Language | TypeScript 5.8 |
| Markdown | marked 15 + highlight.js 11 |
| Testing | vitest 3.2 + jsdom 26 |

The compiled frontend (`web/dist/`) is embedded into the Rust binary via `rust-embed`, so `dalang web` works as a single self-contained binary with zero external dependencies for serving.

## Key Design Decisions

1. **DashMap over Mutex** — lock-free concurrent access for session state.
2. **mpsc channels for WS** — engine tasks push events via channel; the WS handler forwards them. Clean separation between computation and I/O.
3. **rust-embed** — zero-copy static serving from the binary. SPA fallback to `index.html` for client-side routing.
4. **Keyring persistence** — API keys and settings stored in the OS keyring (via `keyring` crate), not in plain-text config files.
