<script lang="ts">
	import { subscribeToasts, removeToast, type ToastItem } from './toast.js';

	let items = $state<ToastItem[]>([]);

	$effect(() => {
		const unsubscribe = subscribeToasts((next) => {
			items = next;
		});
		return unsubscribe;
	});

	const colors: Record<string, string> = {
		success: 'border-emerald-400/40 bg-emerald-500/20 text-emerald-100',
		error: 'border-rose-400/40 bg-rose-500/20 text-rose-100',
		warning: 'border-amber-400/40 bg-amber-500/20 text-amber-100',
		info: 'border-sky-400/40 bg-sky-500/20 text-sky-100'
	};

	const icons: Record<string, string> = {
		success: 'OK',
		error: 'ERR',
		warning: 'WARN',
		info: 'INFO'
	};
</script>

{#if items.length > 0}
	<div class="pointer-events-none fixed top-4 right-4 z-50 flex max-w-sm flex-col gap-2">
		{#each items as toastItem (toastItem.id)}
			<div
				class="pointer-events-auto rounded-lg border px-3 py-2 text-sm backdrop-blur {colors[
					toastItem.type
				] ?? colors.info}"
			>
				<div class="flex items-start gap-2">
					<span class="font-mono text-[10px] tracking-[0.14em] opacity-80"
						>{icons[toastItem.type] ?? icons.info}</span
					>
					<p class="flex-1">{toastItem.message}</p>
					<button
						class="text-xs opacity-80 hover:opacity-100"
						onclick={() => removeToast(toastItem.id)}
						aria-label="Dismiss"
					>
						x
					</button>
				</div>
			</div>
		{/each}
	</div>
{/if}
