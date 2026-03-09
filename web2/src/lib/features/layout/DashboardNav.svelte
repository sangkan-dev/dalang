<script lang="ts">
	import { resolve } from '$app/paths';

	type DashboardRoute =
		| '/dashboard'
		| '/dashboard/chat'
		| '/dashboard/skills'
		| '/dashboard/reports'
		| '/dashboard/settings';

	let {
		navItems,
		activePath,
		onNavigate
	}: {
		navItems: Array<{ route: DashboardRoute; label: string }>;
		activePath: string;
		onNavigate: () => void;
	} = $props();
</script>

<nav class="space-y-1">
	{#each navItems as item (item.route)}
		<a
			href={resolve(item.route)}
			class={`block rounded-lg px-3 py-2 text-sm transition-colors ${activePath === item.route ? 'bg-(--color-gold)/20 text-(--color-gold-bright)' : 'text-(--color-ash) hover:bg-white/5 hover:text-(--color-base-text)'}`}
			onclick={onNavigate}
		>
			{item.label}
		</a>
	{/each}
</nav>
