<script lang="ts">
	import { onMount } from 'svelte';
	import { apiClient } from '$lib/api/client.js';
	import { renderMarkdown } from '$lib/markdown.js';
	import type { ReportEntry } from '$lib/api/types.js';
	import { toast } from '$lib/dashboard/toast.js';

	let loading = $state(true);
	let loadingReport = $state(false);
	let error = $state('');
	let reports = $state<ReportEntry[]>([]);
	let selectedReport = $state<string | null>(null);
	let reportMarkdown = $state('');
	let reportHtml = $state('');

	async function loadReports(): Promise<void> {
		loading = true;
		try {
			reports = await apiClient.listReports();
			error = '';
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load reports';
		} finally {
			loading = false;
		}
	}

	async function viewReport(filename: string): Promise<void> {
		loadingReport = true;
		try {
			const response = await fetch(`/api/reports/${filename}`);
			if (!response.ok) {
				throw new Error(`HTTP ${response.status}`);
			}
			const data = (await response.json()) as { filename: string; content: string };
			reportMarkdown = data.content;
			reportHtml = renderMarkdown(data.content);
			selectedReport = filename;
		} catch (err) {
			toast.error(`Failed to load report: ${err instanceof Error ? err.message : 'unknown error'}`);
		} finally {
			loadingReport = false;
		}
	}

	async function downloadMarkdown(): Promise<void> {
		if (!selectedReport || !reportMarkdown) return;
		try {
			const blob = new Blob([reportMarkdown], { type: 'text/markdown' });
			const url = URL.createObjectURL(blob);
			const anchor = document.createElement('a');
			anchor.href = url;
			anchor.download = selectedReport;
			anchor.click();
			URL.revokeObjectURL(url);
		} catch (err) {
			toast.error(`Download failed: ${err instanceof Error ? err.message : 'unknown error'}`);
		}
	}

	function downloadHtml(): void {
		if (!selectedReport || !reportHtml) return;
		const html = `<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1" /><title>${selectedReport}</title></head><body style="font-family: ui-sans-serif, system-ui; margin: 24px; line-height: 1.55;">${reportHtml}</body></html>`;
		const blob = new Blob([html], { type: 'text/html' });
		const url = URL.createObjectURL(blob);
		const anchor = document.createElement('a');
		anchor.href = url;
		anchor.download = selectedReport.replace(/\.md$/, '.html');
		anchor.click();
		URL.revokeObjectURL(url);
	}

	onMount(loadReports);
</script>

<section class="space-y-4">
	<header>
		<p class="text-xs tracking-[0.2em] text-[color:var(--color-ash)] uppercase">
			Dashboard / Reports
		</p>
		<h2 class="text-xl font-semibold text-[color:var(--color-text)]">Reports Archive</h2>
	</header>

	<div class="grid gap-3 lg:grid-cols-[300px_1fr]">
		<div class="surface-panel max-h-[72vh] overflow-auto p-2">
			{#if loading}
				<p class="px-2 py-2 text-sm text-[color:var(--color-ash)]">Loading reports...</p>
			{:else if error}
				<p class="px-2 py-2 text-sm text-[color:var(--color-rust)]">{error}</p>
			{:else if reports.length === 0}
				<p class="px-2 py-2 text-sm text-[color:var(--color-ash)]">No reports yet</p>
			{:else}
				{#each reports as report}
					<button
						class="mb-1 w-full rounded-lg border px-3 py-2 text-left {selectedReport ===
						report.filename
							? 'border-[color:var(--color-gold)]/40 bg-[color:var(--color-gold)]/10'
							: 'border-transparent hover:border-[color:var(--color-border)]'}"
						onclick={() => viewReport(report.filename)}
					>
						<p class="truncate text-sm font-semibold text-[color:var(--color-base-text)]">
							{report.filename}
						</p>
						<p class="text-xs text-[color:var(--color-ash)]">
							{(report.size / 1024).toFixed(1)} KB
						</p>
					</button>
				{/each}
			{/if}
		</div>

		<div class="surface-panel p-4">
			{#if loadingReport}
				<p class="text-sm text-[color:var(--color-ash)]">Loading report...</p>
			{:else if reportHtml}
				<div class="mb-3 flex flex-wrap gap-2">
					<button
						class="rounded-lg border border-[color:var(--color-border)] px-3 py-2 text-xs text-[color:var(--color-ash)]"
						onclick={downloadMarkdown}>Download MD</button
					>
					<button
						class="rounded-lg border border-[color:var(--color-border)] px-3 py-2 text-xs text-[color:var(--color-ash)]"
						onclick={downloadHtml}>Download HTML</button
					>
				</div>
				<div
					class="dashboard-markdown rounded-lg border border-[color:var(--color-border)] p-3"
					dir="auto"
				>
					{@html reportHtml}
				</div>
			{:else}
				<p class="text-sm text-[color:var(--color-ash)]">Select a report to view</p>
			{/if}
		</div>
	</div>
</section>
