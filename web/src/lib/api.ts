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
};

// ─── WebSocket connection ───────────────────────────

export function createWebSocket(
  sessionId: string,
  callbacks: WebSocketCallbacks,
): DalangWebSocket {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const ws = new WebSocket(`${protocol}//${window.location.host}${API_BASE}/ws/${sessionId}`);

  ws.onopen = (): void => {
    console.log('[ws] connected to session', sessionId);
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
  };

  ws.onerror = (event: Event): void => {
    console.error('[ws] error:', event);
    callbacks.onError?.(event);
  };

  return {
    send(msg: ClientMessage): void {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(msg));
      }
    },
    sendChat(message: string): void {
      this.send({ type: 'chat', message });
    },
    startScan(target: string, maxIter: number = 15, cmdTimeout: number = 300): void {
      this.send({ type: 'start_scan', target, max_iter: maxIter, cmd_timeout: cmdTimeout });
    },
    startInteractive(target: string, cmdTimeout: number = 300): void {
      this.send({ type: 'start_interactive', target, cmd_timeout: cmdTimeout });
    },
    close(): void {
      ws.close();
    },
    get readyState(): number {
      return ws.readyState;
    },
  };
}
