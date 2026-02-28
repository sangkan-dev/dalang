// ─── Shared type definitions for Dalang Web UI ─────────────

// ─── Page navigation ────────────────────────────────────────

export type PageId = 'chat' | 'skills' | 'reports' | 'settings';

export interface NavItem {
  id: PageId;
  label: string;
  icon: string;
}

// ─── Engine events (mirrors Rust EngineEvent enum) ──────────

export type EngineEvent =
  | { type: 'thinking'; iteration: number; max_iter: number | null }
  | { type: 'assistant_message'; content: string; done: boolean }
  | { type: 'tool_execution'; skill: string; command: string }
  | { type: 'observation'; skill: string; content: string; bytes: number }
  | { type: 'safety_refusal'; retry: number }
  | { type: 'browser_action'; action: string; success: boolean; content: string }
  | { type: 'report'; markdown: string; filename: string | null }
  | { type: 'status'; message: string }
  | { type: 'error'; message: string }
  | { type: 'done'; reason: string };

// ─── Client → server messages ───────────────────────────────

export type ClientMessage =
  | { type: 'chat'; message: string }
  | { type: 'start_scan'; target: string; max_iter: number; cmd_timeout: number }
  | { type: 'start_interactive'; target: string; cmd_timeout: number };

// ─── Chat messages (UI state) ───────────────────────────────

export type MessageRole =
  | 'user'
  | 'assistant'
  | 'status'
  | 'tool'
  | 'observation'
  | 'warning'
  | 'error'
  | 'report';

export interface ChatMessage {
  role: MessageRole;
  content: string;
  bytes?: number;
  skill?: string;
  filename?: string;
}

export interface RoleConfig {
  label: string;
  icon: string;
  bg: string;
  border: string;
}

// ─── Session mode ───────────────────────────────────────────

export type SessionMode = 'interactive' | 'scan';

export interface Session {
  id: string;
  target: string;
  mode: SessionMode;
  messages: Array<{ role: string; content: string }>;
  created_at: string;
  active: boolean;
}

// ─── Skills ─────────────────────────────────────────────────

export interface SkillSummary {
  name: string;
  description: string;
  tool_path: string | null;
  requires_root: boolean;
  has_args: boolean;
  enabled?: boolean;
}

export interface SkillDetail {
  name: string;
  description: string;
  tool_path: string | null;
  requires_root: boolean;
  args: string[] | null;
  system_prompt: string;
  role: string | null;
  task: string | null;
  constraints: string | null;
}

// ─── Reports ────────────────────────────────────────────────

export interface ReportEntry {
  filename: string;
  size: number;
}

// ─── Settings ───────────────────────────────────────────────

export interface Settings {
  provider: string;
  model: string;
  auth_method: string;
  endpoint_mode: string;
  auth_status: string;
  api_key?: string;
  verbose?: boolean;
}

export interface TestConnectionResult {
  success: boolean;
  message: string;
  latency_ms: number;
}

export const PROVIDER_MODELS: Record<string, string[]> = {
  gemini: ['gemini-2.5-flash', 'gemini-2.5-pro', 'gemini-2.5-flash-lite', 'gemini-2.0-flash'],
  openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4.1', 'gpt-4.1-mini'],
  anthropic: ['claude-sonnet-4-20250514', 'claude-3-5-haiku-20241022'],
  ollama: ['llama3.1:latest', 'qwen2.5:latest', 'mistral:latest'],
};

// ─── WebSocket wrapper ──────────────────────────────────────

export interface DalangWebSocket {
  send(msg: ClientMessage): void;
  sendChat(message: string): void;
  startScan(target: string, maxIter?: number, cmdTimeout?: number): void;
  startInteractive(target: string, cmdTimeout?: number): void;
  close(): void;
  readonly readyState: number;
}

export interface WebSocketCallbacks {
  onEvent?: (event: EngineEvent) => void;
  onClose?: (event: CloseEvent) => void;
  onError?: (event: Event) => void;
  onReconnecting?: (attempt: number, maxAttempts: number) => void;
  onReconnected?: () => void;
}
