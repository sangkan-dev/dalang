<script lang="ts">
	import type { ReportEntry } from '$lib/api/types.js';

	let {
		reports,
		loading,
		error,
		selectedReport,
		onSelect
	}: {
		reports: ReportEntry[];
		loading: boolean;
		error: string;
		selectedReport: string | null;
		onSelect: (filename: string) => void | Promise<void>;
	} = $props();
</script>

<div class="surface-panel max-h-[72vh] min-w-0 overflow-auto p-2">
	<div class="px-2 py-1 text-[10px] tracking-[0.13em] text-(--color-ash) uppercase">
		Laporan tersimpan: {reports.length}
	</div>
	{#if loading}
		<p class="px-2 py-2 text-sm text-(--color-ash)">Memuat laporan…</p>
	{:else if error}
		<p class="px-2 py-2 text-sm text-(--color-rust)">{error}</p>
	{:else if reports.length === 0}
		<p class="px-2 py-2 text-sm text-(--color-ash)">Belum ada laporan</p>
	{:else}
		{#each reports as report (report.filename)}
			<button
				class={`mb-1 w-full rounded-lg border px-3 py-2 text-left ${selectedReport === report.filename ? 'border-(--color-gold)/40 bg-(--color-gold)/10' : 'border-transparent hover:border-(--color-border)'}`}
				onclick={() => onSelect(report.filename)}
			>
				<p class="truncate text-sm font-semibold text-(--color-base-text)">
					{report.filename}
				</p>
				<p class="text-xs text-(--color-ash)">{(report.size / 1024).toFixed(1)} KB</p>
			</button>
		{/each}
	{/if}
</div>
