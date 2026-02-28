/**
 * WebSocket + REST API client for Dalang backend.
 */

import type {
  ClientMessage,
  DalangWebSocket,
  EngineEvent,
  ReportEntry,
  Session,
  SessionMode,
  Settings,
  SkillDetail,
  SkillSummary,
  TestConnectionResult,
  WebSocketCallbacks,
} from './types';

const API_BASE = '/api';

// ─── REST helpers ───────────────────────────────────

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: { 'Content-Type': 'application/json', ...options.headers },
    ...options,
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`${res.status}: ${text}`);
  }
  if (res.status === 204) return null as T;
  return res.json() as Promise<T>;
}

export const api = {
  // Sessions
  createSession: (target: string, mode: SessionMode = 'interactive'): Promise<Session> =>
    request<Session>('/sessions', { method: 'POST', body: JSON.stringify({ target, mode }) }),
  listSessions: (): Promise<Session[]> =>
    request<Session[]>('/sessions'),
  deleteSession: (id: string): Promise<null> =>
    request<null>(`/sessions/${id}`, { method: 'DELETE' }),
  getMessages: (id: string): Promise<Array<{ role: string; content: string }>> =>
    request<Array<{ role: string; content: string }>>(`/sessions/${id}/messages`),

  // Skills
  listSkills: (): Promise<SkillSummary[]> =>
    request<SkillSummary[]>('/skills'),
  getSkill: (name: string): Promise<SkillDetail> =>
    request<SkillDetail>(`/skills/${name}`),
  updateSkill: (name: string, enabled: boolean): Promise<null> =>
    request<null>(`/skills/${name}`, { method: 'PUT', body: JSON.stringify({ enabled }) }),

  // Reports
  listReports: (): Promise<ReportEntry[]> =>
    request<ReportEntry[]>('/reports'),
  getReport: (filename: string, format: string = 'html'): Promise<string> =>
    request<string>(`/reports/${filename}?format=${format}`),

  // Settings
  getSettings: (): Promise<Settings> =>
    request<Settings>('/settings'),
  updateSettings: (settings: Settings): Promise<null> =>
    request<null>('/settings', { method: 'PUT', body: JSON.stringify(settings) }),
  testConnection: (): Promise<TestConnectionResult> =>
    request<TestConnectionResult>('/settings/test-connection', { method: 'POST' }),
};

// ─── WebSocket connection ───────────────────────────

const MAX_RECONNECT_ATTEMPTS = 5;
const BASE_RECONNECT_DELAY = 1000; // 1s, doubles each attempt

export function createWebSocket(
  sessionId: string,
  callbacks: WebSocketCallbacks,
): DalangWebSocket {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const url = `${protocol}//${window.location.host}${API_BASE}/ws/${sessionId}`;

  let ws: WebSocket;
  let reconnectAttempts = 0;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let intentionalClose = false;

  function connect(): void {
    ws = new WebSocket(url);

    ws.onopen = (): void => {
      console.log('[ws] connected to session', sessionId);
      if (reconnectAttempts > 0) {
        callbacks.onReconnected?.();
      }
      reconnectAttempts = 0;
    };

    ws.onmessage = (event: MessageEvent): void => {
      try {
        const data = JSON.parse(event.data as string) as EngineEvent;
        callbacks.onEvent?.(data);
      } catch (e) {
        console.error('[ws] parse error:', e);
      }
    };

    ws.onclose = (event: CloseEvent): void => {
      console.log('[ws] closed', event.code, event.reason);
      callbacks.onClose?.(event);

      if (!intentionalClose && event.code !== 1000 && reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
        scheduleReconnect();
      }
    };

    ws.onerror = (event: Event): void => {
      console.error('[ws] error:', event);
      callbacks.onError?.(event);
    };
  }

  function scheduleReconnect(): void {
    reconnectAttempts++;
    const delay = BASE_RECONNECT_DELAY * Math.pow(2, reconnectAttempts - 1);
    console.log(`[ws] reconnecting in ${delay}ms (attempt ${reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS})`);
    callbacks.onReconnecting?.(reconnectAttempts, MAX_RECONNECT_ATTEMPTS);
    reconnectTimer = setTimeout(connect, delay);
  }

  connect();

  // Wait for the WebSocket to reach OPEN state (or timeout after 5s)
  function waitForOpen(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (ws.readyState === WebSocket.OPEN) return resolve();
      const timeout = setTimeout(() => reject(new Error('WebSocket open timeout')), 5000);
      const origOnOpen = ws.onopen;
      ws.onopen = (ev: Event): void => {
        clearTimeout(timeout);
        if (origOnOpen) (origOnOpen as (ev: Event) => void)(ev);
        resolve();
      };
    });
  }

  return {
    send(msg: ClientMessage): void {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(msg));
      } else {
        console.warn('[ws] send() called but WebSocket not OPEN (state:', ws.readyState, ')');
      }
    },
    sendChat(message: string): void {
      this.send({ type: 'chat', message });
    },
    async startScan(target: string, maxIter: number = 15, cmdTimeout: number = 300): Promise<void> {
      await waitForOpen();
      this.send({ type: 'start_scan', target, max_iter: maxIter, cmd_timeout: cmdTimeout });
    },
    async startInteractive(target: string, cmdTimeout: number = 300): Promise<void> {
      await waitForOpen();
      this.send({ type: 'start_interactive', target, cmd_timeout: cmdTimeout });
    },
    close(): void {
      intentionalClose = true;
      if (reconnectTimer) clearTimeout(reconnectTimer);
      ws.close();
    },
    get readyState(): number {
      return ws.readyState;
    },
  };
}
