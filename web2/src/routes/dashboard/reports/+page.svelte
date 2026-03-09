<script lang="ts">
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
	import { apiClient } from '$lib/api/client.js';
	import { renderMarkdown } from '$lib/markdown.js';
	import type { ReportEntry } from '$lib/api/types.js';
	import { toast } from '$lib/dashboard/toast.js';
	import ReportsListPane from '$lib/features/reports/ReportsListPane.svelte';
	import ReportsDetailPane from '$lib/features/reports/ReportsDetailPane.svelte';

	let loading = $state(true);
	let loadingReport = $state(false);
	let error = $state('');
	let reports = $state<ReportEntry[]>([]);
	let selectedReport = $state<string | null>(null);
	let reportMarkdown = $state('');
	let reportHtml = $state('');
	let reportView = $state<'formatted' | 'raw'>('formatted');

	function setReportViewFormatted(): void {
		reportView = 'formatted';
	}

	function setReportViewRaw(): void {
		reportView = 'raw';
	}

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
			reportView = 'formatted';
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
	<header class="surface-panel dashboard-warboard space-y-2 p-4">
		<p class="text-xs tracking-[0.2em] text-(--color-ash) uppercase">Dashboard / Reports</p>
		<div class="flex flex-wrap items-center justify-between gap-2">
			<h2 class="text-xl font-semibold text-(--color-base-text)">Reports Archive</h2>
			<a href={resolve('/dashboard/chat')} class="control-chip">Go To Chat</a>
		</div>
		<p class="text-xs text-(--color-ash)">Inspect, diff, and export reproducible audit evidence.</p>
	</header>

	<div class="grid gap-3 lg:grid-cols-[300px_1fr]">
		<ReportsListPane {reports} {loading} {error} {selectedReport} onSelect={viewReport} />

		<ReportsDetailPane
			{loadingReport}
			{reportHtml}
			{reportMarkdown}
			{reportView}
			onSetFormatted={setReportViewFormatted}
			onSetRaw={setReportViewRaw}
			onDownloadMarkdown={downloadMarkdown}
			onDownloadHtml={downloadHtml}
		/>
	</div>
</section>
