<script lang="ts">
	let {
		showPalette,
		paletteQuery,
		selectedIndex,
		filteredPalette,
		onPaletteInput,
		onPaletteQueryChange,
		onClose,
		onKeydown,
		onSelectItem,
		onRunSelection
	}: {
		showPalette: boolean;
		paletteQuery: string;
		selectedIndex: number;
		filteredPalette: Array<{ route: string; label: string; desc: string }>;
		onPaletteInput: () => void;
		onPaletteQueryChange: (value: string) => void;
		onClose: () => void;
		onKeydown: (event: KeyboardEvent) => void;
		onSelectItem: (index: number) => void;
		onRunSelection: () => Promise<void>;
	} = $props();
</script>

{#if showPalette}
	<div
		class="fixed inset-0 z-50 bg-black/60"
		role="presentation"
		onclick={onClose}
		onkeydown={() => {}}
	>
		<div
			class="mx-auto mt-24 w-full max-w-xl rounded-xl border border-(--color-border) bg-(--color-surface)"
			role="dialog"
			tabindex="-1"
			onclick={(event) => event.stopPropagation()}
			onkeydown={onKeydown}
		>
			<div class="border-b border-(--color-border) px-4 py-3">
				<input
					value={paletteQuery}
					placeholder="Cari halaman atau perintah…"
					oninput={(event) => {
						onPaletteQueryChange((event.currentTarget as HTMLInputElement).value);
						onPaletteInput();
					}}
					class="w-full bg-transparent text-sm text-(--color-base-text) outline-none placeholder:text-(--color-ash)"
				/>
			</div>
			<div class="max-h-72 overflow-y-auto p-2">
				{#if filteredPalette.length === 0}
					<p class="px-2 py-3 text-sm text-(--color-ash)">Tidak ada yang cocok</p>
				{:else}
					{#each filteredPalette as item, index (item.route + index)}
						<button
							class={`w-full rounded-lg px-3 py-2 text-left text-sm ${index === selectedIndex ? 'bg-(--color-gold)/20 text-(--color-gold-bright)' : 'text-(--color-ash) hover:bg-white/5'}`}
							onclick={async () => {
								onSelectItem(index);
								await onRunSelection();
							}}
						>
							<p>{item.label}</p>
							<p class="text-xs opacity-70">{item.desc}</p>
						</button>
					{/each}
				{/if}
			</div>
		</div>
	</div>
{/if}
