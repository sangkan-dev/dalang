<script lang="ts">
	import { resolve } from '$app/paths';
	import type { Session } from '$lib/api/types.js';

	let {
		sessions,
		onOpenSession,
		onDeleteSession
	}: {
		sessions: Session[];
		onOpenSession: (id: string) => void | Promise<void>;
		onDeleteSession: (event: Event, id: string) => void | Promise<void>;
	} = $props();
</script>

<div class="mt-4 border-t border-(--color-border) pt-3">
	<div class="mb-2 flex items-center justify-between">
		<p class="text-[10px] tracking-[0.16em] text-(--color-ash) uppercase">Sessions</p>
		<a href={resolve('/dashboard/chat')} class="text-xs text-(--color-gold-bright)">new</a>
	</div>
	<div class="space-y-1">
		{#if sessions.length === 0}
			<p class="px-2 py-1 text-xs text-(--color-ash)">No sessions yet</p>
		{:else}
			{#each sessions as session (session.id)}
				<div
					class="group flex items-center justify-between rounded-md px-2 py-1.5 text-xs text-(--color-ash) hover:bg-white/5"
				>
					<button
						class="min-w-0 flex-1 truncate pr-2 text-left"
						onclick={() => onOpenSession(session.id)}
					>
						{session.target}
					</button>
					<button
						class="hidden text-(--color-rust) group-hover:block"
						onclick={(event) => onDeleteSession(event, session.id)}
						aria-label="Delete session"
					>
						x
					</button>
				</div>
			{/each}
		{/if}
	</div>
</div>
