<script lang="ts">
	import { onMount } from 'svelte';
	import { apiClient } from '$lib/api/client.js';

	type LoadState = 'idle' | 'loading' | 'ready' | 'error';

	let state: LoadState = 'idle';
	let errorMessage = '';
	let totalSessions = 0;
	let totalSkills = 0;
	let totalReports = 0;
	let activeSessions = 0;
	let currentProvider = '-';
	let currentModel = '-';

	onMount(async () => {
		state = 'loading';
		try {
			const [sessions, skills, reports, settings] = await Promise.all([
				apiClient.listSessions(),
				apiClient.listSkills(),
				apiClient.listReports(),
				apiClient.getSettings()
			]);

			totalSessions = sessions.length;
			activeSessions = sessions.filter((session) => session.active).length;
			totalSkills = skills.length;
			totalReports = reports.length;
			currentProvider = settings.provider;
			currentModel = settings.model;
			state = 'ready';
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : 'Failed to load dashboard summary';
			state = 'error';
		}
	});
</script>

<section class="space-y-6">
	<header class="space-y-2">
		<p class="text-xs tracking-[0.2em] text-[color:var(--color-ash)] uppercase">
			Sprint 33 Initialization
		</p>
		<h2 class="text-2xl font-semibold text-[color:var(--color-text)]">
			Dashboard Migration Started
		</h2>
		<p class="max-w-2xl text-sm text-[color:var(--color-ash)]">
			Shared API client has been moved into SvelteKit. This page verifies REST connectivity while
			the chat, skills, reports, and settings views are being ported.
		</p>
		<div class="pt-2">
			<a
				href="/dashboard/chat"
				class="inline-flex items-center rounded-lg bg-[color:var(--color-gold)] px-4 py-2 text-sm font-semibold text-[color:#1f1708]"
			>
				Open Chat Console
			</a>
		</div>
	</header>

	{#if state === 'loading' || state === 'idle'}
		<div
			class="rounded-2xl border border-[color:var(--color-border)] bg-[color:var(--color-surface)] p-4 text-sm text-[color:var(--color-ash)]"
		>
			Loading backend summary...
		</div>
	{:else if state === 'error'}
		<div
			class="rounded-2xl border border-[color:var(--color-rust)]/50 bg-[color:var(--color-rust)]/10 p-4 text-sm text-[color:var(--color-rust)]"
		>
			Failed to fetch dashboard summary: {errorMessage}
		</div>
	{:else}
		<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
			<article
				class="rounded-2xl border border-[color:var(--color-border)] bg-[color:var(--color-surface)] p-4"
			>
				<p class="text-xs tracking-[0.15em] text-[color:var(--color-ash)] uppercase">Sessions</p>
				<p class="mt-2 text-2xl font-semibold text-[color:var(--color-text)]">{totalSessions}</p>
				<p class="text-xs text-[color:var(--color-ash)]">{activeSessions} active</p>
			</article>
			<article
				class="rounded-2xl border border-[color:var(--color-border)] bg-[color:var(--color-surface)] p-4"
			>
				<p class="text-xs tracking-[0.15em] text-[color:var(--color-ash)] uppercase">Skills</p>
				<p class="mt-2 text-2xl font-semibold text-[color:var(--color-text)]">{totalSkills}</p>
				<p class="text-xs text-[color:var(--color-ash)]">registered</p>
			</article>
			<article
				class="rounded-2xl border border-[color:var(--color-border)] bg-[color:var(--color-surface)] p-4"
			>
				<p class="text-xs tracking-[0.15em] text-[color:var(--color-ash)] uppercase">Reports</p>
				<p class="mt-2 text-2xl font-semibold text-[color:var(--color-text)]">{totalReports}</p>
				<p class="text-xs text-[color:var(--color-ash)]">stored artifacts</p>
			</article>
			<article
				class="rounded-2xl border border-[color:var(--color-border)] bg-[color:var(--color-surface)] p-4"
			>
				<p class="text-xs tracking-[0.15em] text-[color:var(--color-ash)] uppercase">Model</p>
				<p class="mt-2 truncate text-sm font-semibold text-[color:var(--color-text)]">
					{currentModel}
				</p>
				<p class="text-xs text-[color:var(--color-ash)]">provider: {currentProvider}</p>
			</article>
		</div>
	{/if}
</section>
