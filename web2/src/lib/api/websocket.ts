import type { ClientMessage, DalangWebSocket, EngineEvent, WebSocketCallbacks } from './types.js';

const API_BASE = '/api';
const MAX_RECONNECT_ATTEMPTS = 5;
const BASE_RECONNECT_DELAY_MS = 1000;
const OPEN_TIMEOUT_MS = 5000;

export function createDalangWebSocket(
	sessionId: string,
	callbacks: WebSocketCallbacks = {}
): DalangWebSocket {
	const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
	const url = `${protocol}//${window.location.host}${API_BASE}/ws/${sessionId}`;

	let socket: WebSocket;
	let reconnectAttempts = 0;
	let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	let intentionalClose = false;

	function connect(): void {
		socket = new WebSocket(url);

		socket.onopen = (): void => {
			callbacks.onOpen?.();
			if (reconnectAttempts > 0) {
				callbacks.onReconnected?.();
			}
			reconnectAttempts = 0;
		};

		socket.onmessage = (event: MessageEvent): void => {
			try {
				const payload = JSON.parse(event.data as string) as EngineEvent;
				callbacks.onEvent?.(payload);
			} catch {
				callbacks.onError?.(new Event('message-parse-error'));
			}
		};

		socket.onclose = (event: CloseEvent): void => {
			callbacks.onClose?.(event);
			if (!intentionalClose && event.code !== 1000 && reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
				scheduleReconnect();
			}
		};

		socket.onerror = (event: Event): void => {
			callbacks.onError?.(event);
		};
	}

	function scheduleReconnect(): void {
		reconnectAttempts += 1;
		const delay = BASE_RECONNECT_DELAY_MS * 2 ** (reconnectAttempts - 1);
		callbacks.onReconnecting?.(reconnectAttempts, MAX_RECONNECT_ATTEMPTS);
		reconnectTimer = setTimeout(connect, delay);
	}

	function waitForOpen(): Promise<void> {
		return new Promise((resolve, reject) => {
			if (socket.readyState === WebSocket.OPEN) {
				resolve();
				return;
			}

			const timeout = setTimeout(
				() => reject(new Error('WebSocket open timeout')),
				OPEN_TIMEOUT_MS
			);
			const previousOnOpen = socket.onopen;
			socket.onopen = (event: Event): void => {
				clearTimeout(timeout);
				if (typeof previousOnOpen === 'function') {
					previousOnOpen.call(socket, event);
				}
				resolve();
			};
		});
	}

	connect();

	return {
		send(message: ClientMessage): void {
			if (socket.readyState === WebSocket.OPEN) {
				socket.send(JSON.stringify(message));
			}
		},
		sendChat(message: string): void {
			this.send({ type: 'chat', message });
		},
		async startScan(target: string, maxIter = 15, cmdTimeout = 300): Promise<void> {
			await waitForOpen();
			this.send({ type: 'start_scan', target, max_iter: maxIter, cmd_timeout: cmdTimeout });
		},
		async startInteractive(target: string, cmdTimeout = 300): Promise<void> {
			await waitForOpen();
			this.send({ type: 'start_interactive', target, cmd_timeout: cmdTimeout });
		},
		close(): void {
			intentionalClose = true;
			if (reconnectTimer) {
				clearTimeout(reconnectTimer);
			}
			socket.close();
		},
		get readyState(): number {
			return socket.readyState;
		}
	};
}
