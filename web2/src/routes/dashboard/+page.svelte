<script lang="ts">
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
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
			errorMessage = error instanceof Error ? error.message : 'Gagal memuat ringkasan dasbor';
			state = 'error';
		}
	});
</script>

<section class="space-y-6">
	<header class="surface-panel dashboard-warboard space-y-3 p-5">
		<p class="text-xs tracking-[0.2em] text-[color:var(--color-ash)] uppercase">RINGKASAN DASBOR</p>
		<h2 class="text-2xl font-semibold text-[color:var(--color-text)]">Apa yang bisa Anda lakukan di sini</h2>
		<p class="max-w-2xl text-sm text-[color:var(--color-ash)]">
			Periksa keamanan sebuah situs atau alamat yang Anda punya izin untuk diuji, ajukan pertanyaan
			langkah demi langkah, lalu unduh laporan untuk dibagikan ke tim IT. Semua alur dirancang agar
			mudah diikuti tanpa istilah operator.
		</p>
		<div class="flex flex-wrap gap-2 pt-2">
			<a
				href={resolve('/dashboard/chat')}
				class="inline-flex items-center rounded-lg bg-[color:var(--color-gold)] px-4 py-2 text-sm font-semibold text-[color:#1f1708] no-underline"
			>
				Mulai pemeriksaan
			</a>
			<a href={resolve('/dashboard/reports')} class="control-chip">Lihat laporan</a>
			<a href={resolve('/dashboard/skills')} class="control-chip">Alat pemeriksaan</a>
		</div>
	</header>

	{#if state === 'loading' || state === 'idle'}
		<div
			class="rounded-2xl border border-[color:var(--color-border)] bg-[color:var(--color-surface)] p-4 text-sm text-[color:var(--color-ash)]"
		>
			Memuat ringkasan dari server…
		</div>
	{:else if state === 'error'}
		<div
			class="rounded-2xl border border-[color:var(--color-rust)]/50 bg-[color:var(--color-rust)]/10 p-4 text-sm text-[color:var(--color-rust)]"
		>
			Gagal mengambil ringkasan dasbor: {errorMessage}
		</div>
	{:else}
		<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
			<article class="surface-panel dashboard-metric-card p-4">
				<p class="text-xs tracking-[0.15em] text-[color:var(--color-ash)] uppercase">Sesi</p>
				<p class="mt-2 text-2xl font-semibold text-[color:var(--color-text)]">{totalSessions}</p>
				<p class="text-xs text-[color:var(--color-ash)]">
					{activeSessions} sedang berjalan
				</p>
			</article>
			<article class="surface-panel dashboard-metric-card p-4">
				<p class="text-xs tracking-[0.15em] text-[color:var(--color-ash)] uppercase">Alat</p>
				<p class="mt-2 text-2xl font-semibold text-[color:var(--color-text)]">{totalSkills}</p>
				<p class="text-xs text-[color:var(--color-ash)]">tersedia untuk AI</p>
			</article>
			<article class="surface-panel dashboard-metric-card p-4">
				<p class="text-xs tracking-[0.15em] text-[color:var(--color-ash)] uppercase">Laporan</p>
				<p class="mt-2 text-2xl font-semibold text-[color:var(--color-text)]">{totalReports}</p>
				<p class="text-xs text-[color:var(--color-ash)]">tersimpan untuk diunduh</p>
			</article>
			<article class="surface-panel dashboard-metric-card p-4">
				<p class="text-xs tracking-[0.15em] text-[color:var(--color-ash)] uppercase">Model AI</p>
				<p class="mt-2 truncate text-sm font-semibold text-[color:var(--color-text)]">
					{currentModel}
				</p>
				<p class="text-xs text-[color:var(--color-ash)]">penyedia: {currentProvider}</p>
			</article>
		</div>
	{/if}
</section>
