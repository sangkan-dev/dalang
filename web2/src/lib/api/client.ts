import type {
	EngineEvent,
	ReportEntry,
	Session,
	SessionMode,
	Settings,
	SkillDetail,
	SkillSummary,
	TestConnectionResult,
	UpdateSettingsRequest
} from './types.js';

const API_BASE = '/api';

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
	const response = await fetch(`${API_BASE}${path}`, {
		headers: { 'Content-Type': 'application/json', ...options.headers },
		...options
	});

	if (!response.ok) {
		const text = await response.text();
		throw new Error(`${response.status}: ${text}`);
	}

	if (response.status === 204) {
		return null as T;
	}

	return response.json() as Promise<T>;
}

export const apiClient = {
	createSession: (target: string, mode: SessionMode = 'interactive'): Promise<Session> =>
		request<Session>('/sessions', { method: 'POST', body: JSON.stringify({ target, mode }) }),
	listSessions: (): Promise<Session[]> => request<Session[]>('/sessions'),
	deleteSession: (id: string): Promise<null> => request<null>(`/sessions/${id}`, { method: 'DELETE' }),
	getMessages: (id: string): Promise<Array<{ role: string; content: string }>> =>
		request<Array<{ role: string; content: string }>>(`/sessions/${id}/messages`),
	getSessionEvents: (id: string): Promise<EngineEvent[]> =>
		request<EngineEvent[]>(`/sessions/${id}/events`),
	listSkills: (): Promise<SkillSummary[]> => request<SkillSummary[]>('/skills'),
	getSkill: (name: string): Promise<SkillDetail> => request<SkillDetail>(`/skills/${name}`),
	updateSkill: (name: string, enabled: boolean): Promise<null> =>
		request<null>(`/skills/${name}`, { method: 'PUT', body: JSON.stringify({ enabled }) }),
	listReports: (): Promise<ReportEntry[]> => request<ReportEntry[]>('/reports'),
	getReport: (filename: string, format = 'html'): Promise<string> =>
		request<string>(`/reports/${filename}?format=${format}`),
	getSettings: (): Promise<Settings> => request<Settings>('/settings'),
	updateSettings: (settings: UpdateSettingsRequest): Promise<null> =>
		request<null>('/settings', { method: 'PUT', body: JSON.stringify(settings) }),
	testConnection: (): Promise<TestConnectionResult> =>
		request<TestConnectionResult>('/settings/test-connection', { method: 'POST' })
};
