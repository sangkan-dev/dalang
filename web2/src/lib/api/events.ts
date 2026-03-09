import type { ChatMessage, EngineEvent } from './types.js';

export function eventToChatMessages(event: EngineEvent): ChatMessage[] {
	switch (event.type) {
		case 'thinking':
			return [
				{
					role: 'status',
					content: event.max_iter
						? `Reasoning (iteration ${event.iteration}/${event.max_iter})...`
						: `Reasoning (iteration ${event.iteration})...`
				}
			];
		case 'assistant_message':
			return [{ role: 'assistant', content: event.content }];
		case 'tool_execution':
			return [
				{
					role: 'tool',
					content: `Executing skill: **${event.skill}**\n\`\`\`bash\n${event.command}\n\`\`\``
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
			return [{ role: 'warning', content: `Safety filter triggered (retry ${event.retry}). Re-prompting...` }];
		case 'browser_action':
			return [
				{
					role: 'tool',
					content: `Browser: ${event.action} - ${event.success ? 'OK' : 'FAIL'}\n${event.content.slice(0, 500)}`
				}
			];
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
			return [{ role: 'status', content: `OK ${event.reason}` }];
		default:
			return [];
	}
}

export function replayEvents(events: EngineEvent[]): ChatMessage[] {
	return events.flatMap((event) => eventToChatMessages(event));
}
