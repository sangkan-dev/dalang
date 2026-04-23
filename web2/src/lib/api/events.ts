import type { ChatMessage, EngineEvent } from './types.js';

export function eventToChatMessages(event: EngineEvent): ChatMessage[] {
	switch (event.type) {
		case 'thinking':
			return [
				{
					role: 'status',
					content: event.max_iter
						? `Berpikir (langkah ${event.iteration} dari ${event.max_iter})…`
						: `Berpikir (langkah ${event.iteration})…`
				}
			];
		case 'assistant_message':
			return [{ role: 'assistant', content: event.content }];
		case 'tool_execution':
			return [
				{
					role: 'tool',
					content: `Menjalankan pemeriksaan: **${event.skill}**`,
					skill: event.skill,
					toolCommand: event.command
				}
			];
		case 'observation':
			return [
				{
					role: 'observation',
					content: event.content,
					bytes: event.bytes,
					skill: event.skill
				}
			];
		case 'safety_refusal':
			return [
				{
					role: 'warning',
					content: `Filter keamanan model aktif (percobaan ulang ke-${event.retry}). Mengarahkan ulang…`
				}
			];
		case 'browser_action': {
			const preview = event.content.slice(0, 500);
			const more = event.content.length > 500;
			return [
				{
					role: 'tool',
					content: `Aksi peramban: **${event.action}** — ${event.success ? 'berhasil' : 'gagal'}${more ? '\n\n*(cuplikan di bawah; buka detail untuk selengkapnya)*' : ''}\n\n${preview}${more ? '…' : ''}`,
					toolCommand: more ? event.content : undefined
				}
			];
		}
		case 'report':
			return [
				{
					role: 'report',
					content: event.markdown,
					filename: event.filename ?? undefined
				}
			];
		case 'status':
			return [{ role: 'status', content: event.message }];
		case 'error':
			return [{ role: 'error', content: event.message }];
		case 'done':
			return [{ role: 'status', content: `Selesai: ${event.reason}` }];
		default:
			return [];
	}
}

export function replayEvents(events: EngineEvent[]): ChatMessage[] {
	return events.flatMap((event) => eventToChatMessages(event));
}

/** Label UI Bahasa Indonesia untuk badge pesan chat */
export function chatRoleLabel(role: ChatMessage['role']): string {
	switch (role) {
		case 'user':
			return 'Anda';
		case 'assistant':
			return 'Asisten';
		case 'status':
			return 'Status';
		case 'tool':
			return 'Pemeriksaan';
		case 'observation':
			return 'Hasil teknis';
		case 'warning':
			return 'Peringatan';
		case 'error':
			return 'Kesalahan';
		case 'report':
			return 'Laporan';
		default:
			return role;
	}
}
