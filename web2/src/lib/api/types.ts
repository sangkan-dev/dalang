export type SessionMode = 'interactive' | 'scan';

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

export type ClientMessage =
	| { type: 'chat'; message: string }
	| { type: 'start_scan'; target: string; max_iter: number; cmd_timeout: number }
	| { type: 'start_interactive'; target: string; cmd_timeout: number };

export interface Session {
	id: string;
	target: string;
	mode: SessionMode;
	created_at: string;
	active: boolean;
	message_count?: number;
	event_count?: number;
}

export interface SkillSummary {
	name: string;
	description: string;
	tool_path: string | null;
	requires_root: boolean;
	has_args: boolean;
	enabled?: boolean;
	tool_available?: boolean;
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
	tool_available?: boolean;
}

export interface ReportEntry {
	filename: string;
	size: number;
}

export interface Settings {
	provider: string;
	model: string;
	auth_method: string;
	endpoint_mode: string;
	auth_status: string;
	has_api_key?: boolean;
	api_key?: string;
	custom_base_url?: string;
	verbose?: boolean;
}

export interface UpdateSettingsRequest {
	model?: string;
	provider?: string;
	endpoint_mode?: string;
	api_key?: string;
	verbose?: boolean;
	custom_base_url?: string;
}

export interface TestConnectionResult {
	success: boolean;
	message: string;
	latency_ms: number;
}

export interface DalangWebSocket {
	send(msg: ClientMessage): void;
	sendChat(message: string): void;
	startScan(target: string, maxIter?: number, cmdTimeout?: number): Promise<void>;
	startInteractive(target: string, cmdTimeout?: number): Promise<void>;
	close(): void;
	readonly readyState: number;
}

export interface WebSocketCallbacks {
	onEvent?: (event: EngineEvent) => void;
	onOpen?: () => void;
	onClose?: (event: CloseEvent) => void;
	onError?: (event: Event) => void;
	onReconnecting?: (attempt: number, maxAttempts: number) => void;
	onReconnected?: () => void;
}

export type ChatMessageRole =
	| 'user'
	| 'assistant'
	| 'status'
	| 'tool'
	| 'observation'
	| 'warning'
	| 'error'
	| 'report';

export interface ChatMessage {
	role: ChatMessageRole;
	content: string;
	bytes?: number;
	skill?: string;
	filename?: string;
	/** Perintah lengkap untuk role tool — ditampilkan di blok lipat */
	toolCommand?: string;
}
