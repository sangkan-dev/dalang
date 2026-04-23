<script lang="ts">
	import { tick } from 'svelte';
	import MarkdownContent from '$lib/ui/content/MarkdownContent.svelte';
	import ViewModeToggle from '$lib/ui/content/ViewModeToggle.svelte';
	import { extractReportToc, type ReportTocItem } from '$lib/reports/markdownToc.js';

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

	let markdownHost = $state<HTMLDivElement | null>(null);

	const toc = $derived(extractReportToc(reportMarkdown));

	function indentClass(level: ReportTocItem['level']): string {
		return level === 3 ? 'pl-3' : '';
	}

	$effect(() => {
		reportHtml;
		toc;
		if (!markdownHost || reportView !== 'formatted' || toc.length === 0) return;
		void tick().then(() => {
			const root = markdownHost?.querySelector('.dashboard-markdown');
			if (!root) return;
			const hs = root.querySelectorAll('h2, h3');
			hs.forEach((el, i) => {
				const item = toc[i];
				if (item) el.id = item.id;
			});
		});
	});
</script>

<div class="surface-panel min-w-0 p-4">
	{#if loadingReport}
		<p class="text-sm text-(--color-ash)">Memuat laporan…</p>
	{:else if reportHtml}
		<div class="mb-3 flex flex-wrap items-center gap-2">
			<ViewModeToggle mode={reportView} onFormatted={onSetFormatted} onRaw={onSetRaw} />
			<button
				class="inline-flex items-center justify-center rounded-md border border-(--color-border) px-3 py-2 text-xs text-(--color-ash) transition hover:border-(--color-gold)/40 hover:text-(--color-base-text)"
				onclick={onDownloadMarkdown}
			>
				Unduh Markdown
			</button>
			<button
				class="inline-flex items-center justify-center rounded-md border border-(--color-border) px-3 py-2 text-xs text-(--color-ash) transition hover:border-(--color-gold)/40 hover:text-(--color-base-text)"
				onclick={onDownloadHtml}
			>
				Unduh HTML
			</button>
		</div>

		<div
			class="grid gap-4 {toc.length > 0 && reportView === 'formatted'
				? 'lg:grid-cols-[minmax(0,1fr)_200px]'
				: ''}"
		>
			<div bind:this={markdownHost} class="min-w-0">
				<MarkdownContent
					html={reportHtml}
					markdown={reportMarkdown}
					mode={reportView}
					className="min-w-0 rounded-lg border border-(--color-border) p-3"
					rawClassName="min-w-0"
				/>
			</div>

			{#if toc.length > 0 && reportView === 'formatted'}
				<aside
					class="order-first max-h-[min(70vh,32rem)] overflow-auto rounded-lg border border-(--color-border) bg-(--color-surface)/80 p-3 lg:order-none lg:sticky lg:top-4 lg:self-start"
				>
					<p class="mb-2 text-[10px] font-semibold tracking-[0.12em] text-(--color-ash) uppercase">
						Bagian laporan
					</p>
					<nav class="space-y-1.5" aria-label="Daftar isi laporan">
						{#each toc as item (item.id)}
							<a
								href="#{item.id}"
								class="block text-xs leading-snug text-(--color-gold) underline-offset-2 hover:underline {indentClass(
									item.level
								)}"
							>
								{item.text}
							</a>
						{/each}
					</nav>
				</aside>
			{/if}
		</div>
	{:else}
		<p class="text-sm text-(--color-ash)">Pilih sebuah laporan di daftar kiri untuk menampilkannya.</p>
	{/if}
</div>
