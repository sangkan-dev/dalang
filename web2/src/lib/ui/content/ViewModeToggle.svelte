<script lang="ts">
	let {
		mode,
		onFormatted,
		onRaw,
		compact = false
	}: {
		mode: 'formatted' | 'raw';
		onFormatted: () => void;
		onRaw: () => void;
		compact?: boolean;
	} = $props();

	function baseClass(): string {
		return compact
			? 'inline-flex items-center justify-center rounded-md border border-(--color-gold)/30 px-2 py-1 font-mono text-[10px] tracking-[0.08em] uppercase transition hover:-translate-y-px disabled:cursor-not-allowed disabled:opacity-45'
			: 'inline-flex items-center justify-center rounded-md border border-(--color-gold)/30 px-3 py-2 font-mono text-xs tracking-[0.08em] uppercase transition hover:-translate-y-px disabled:cursor-not-allowed disabled:opacity-45';
	}
</script>

<div class="flex items-center gap-2">
	<button
		class={`${baseClass()} ${mode === 'formatted' ? 'bg-(--color-gold)/15 text-(--color-gold-bright)' : 'text-(--color-ash) hover:text-(--color-gold-bright)'}`}
		onclick={onFormatted}
		disabled={mode === 'formatted'}
	>
		Formatted
	</button>
	<button
		class={`${baseClass()} ${mode === 'raw' ? 'bg-(--color-gold)/15 text-(--color-gold-bright)' : 'text-(--color-ash) hover:text-(--color-gold-bright)'}`}
		onclick={onRaw}
		disabled={mode === 'raw'}
	>
		Raw
	</button>
</div>
