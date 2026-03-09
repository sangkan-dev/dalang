<script lang="ts">
	import MarkdownContent from '$lib/ui/content/MarkdownContent.svelte';
	import ViewModeToggle from '$lib/ui/content/ViewModeToggle.svelte';

	let {
		loadingReport,
		reportHtml,
		reportMarkdown,
		reportView,
		onSetFormatted,
		onSetRaw,
		onDownloadMarkdown,
		onDownloadHtml
	}: {
		loadingReport: boolean;
		reportHtml: string;
		reportMarkdown: string;
		reportView: 'formatted' | 'raw';
		onSetFormatted: () => void;
		onSetRaw: () => void;
		onDownloadMarkdown: () => void | Promise<void>;
		onDownloadHtml: () => void;
	} = $props();
</script>

<div class="surface-panel min-w-0 p-4">
	{#if loadingReport}
		<p class="text-sm text-(--color-ash)">Loading report...</p>
	{:else if reportHtml}
		<div class="mb-3 flex flex-wrap items-center gap-2">
			<ViewModeToggle mode={reportView} onFormatted={onSetFormatted} onRaw={onSetRaw} />
			<button
				class="inline-flex items-center justify-center rounded-md border border-(--color-border) px-3 py-2 text-xs text-(--color-ash) transition hover:border-(--color-gold)/40 hover:text-(--color-base-text)"
				onclick={onDownloadMarkdown}
			>
				Download MD
			</button>
			<button
				class="inline-flex items-center justify-center rounded-md border border-(--color-border) px-3 py-2 text-xs text-(--color-ash) transition hover:border-(--color-gold)/40 hover:text-(--color-base-text)"
				onclick={onDownloadHtml}
			>
				Download HTML
			</button>
		</div>

		<MarkdownContent
			html={reportHtml}
			markdown={reportMarkdown}
			mode={reportView}
			className="min-w-0 rounded-lg border border-(--color-border) p-3"
			rawClassName="min-w-0"
		/>
	{:else}
		<p class="text-sm text-(--color-ash)">Select a report to view</p>
	{/if}
</div>
