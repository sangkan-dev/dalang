<script lang="ts">
	import { renderMarkdownRaw } from '$lib/markdown.js';

	let {
		html,
		markdown,
		mode,
		className = '',
		rawClassName = ''
	}: {
		html: string;
		markdown: string;
		mode: 'formatted' | 'raw';
		className?: string;
		rawClassName?: string;
	} = $props();
</script>

{#if mode === 'raw'}
	<pre class={`dashboard-raw ${rawClassName}`} dir="auto">{renderMarkdownRaw(markdown)}</pre>
{:else}
	<div class={`dashboard-markdown ${className}`} dir="auto">
		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		{@html html}
	</div>
{/if}
