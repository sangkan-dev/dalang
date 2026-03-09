<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { apiClient } from '$lib/api/client.js';
	import type { Session } from '$lib/api/types.js';
	import ToastStack from '$lib/dashboard/ToastStack.svelte';
	import { toast } from '$lib/dashboard/toast.js';

	const { children } = $props();

	let sessions = $state<Session[]>([]);
	let mobileOpen = $state(false);
	let showPalette = $state(false);

	const navItems = [
		{ href: '/dashboard', label: 'Overview' },
		{ href: '/dashboard/chat', label: 'Chat' },
		{ href: '/dashboard/skills', label: 'Skills' },
		{ href: '/dashboard/reports', label: 'Reports' },
		{ href: '/dashboard/settings', label: 'Settings' }
	];

	const paletteItems = [
		{ href: '/dashboard/chat', label: 'Open Chat', desc: 'Interactive console' },
		{ href: '/dashboard/skills', label: 'Browse Skills', desc: 'Skill catalog and toggles' },
		{ href: '/dashboard/reports', label: 'Open Reports', desc: 'Saved report artifacts' },
		{ href: '/dashboard/settings', label: 'Open Settings', desc: 'Provider and model config' },
		{ href: '/dashboard/chat', label: 'New Session', desc: 'Start fresh target session' }
	];

	let paletteQuery = $state('');
	let selectedIndex = $state(0);

	const filteredPalette = $derived.by(() => {
		const query = paletteQuery.trim().toLowerCase();
		if (!query) return paletteItems;
		return paletteItems.filter((item) => {
			return item.label.toLowerCase().includes(query) || item.desc.toLowerCase().includes(query);
		});
	});

	async function loadSessions(): Promise<void> {
		try {
			sessions = await apiClient.listSessions();
		} catch {
			sessions = [];
		}
	}

	async function deleteSession(event: Event, id: string): Promise<void> {
		event.preventDefault();
		event.stopPropagation();
		try {
			await apiClient.deleteSession(id);
			sessions = sessions.filter((session) => session.id !== id);
			toast.success('Session deleted');
		} catch (error) {
			toast.error(`Failed to delete session: ${error instanceof Error ? error.message : 'unknown'}`);
		}
	}

	function openPalette(): void {
		showPalette = true;
		paletteQuery = '';
		selectedIndex = 0;
	}

	function closePalette(): void {
		showPalette = false;
		paletteQuery = '';
		selectedIndex = 0;
	}

	function selectPaletteItem(index: number): void {
		selectedIndex = index;
	}

	function runPaletteSelection(): void {
		const item = filteredPalette[selectedIndex];
		if (!item) return;
		goto(item.href);
		closePalette();
	}

	function onPaletteKeydown(event: KeyboardEvent): void {
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, filteredPalette.length - 1);
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0);
		} else if (event.key === 'Enter') {
			event.preventDefault();
			runPaletteSelection();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			closePalette();
		}
	}

	$effect(() => {
		paletteQuery;
		selectedIndex = 0;
	});

	onMount(() => {
		loadSessions();
		const timer = setInterval(loadSessions, 10000);

		const onKey = (event: KeyboardEvent): void => {
			if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
				event.preventDefault();
				openPalette();
			}
		};

		window.addEventListener('keydown', onKey);
		return () => {
			clearInterval(timer);
			window.removeEventListener('keydown', onKey);
		};
	});
</script>

<div class="dashboard-shell">
	<header class="dashboard-header">
		<div>
			<p class="dashboard-kicker">DALANG DASHBOARD</p>
			<h1 class="dashboard-title">Operational Console</h1>
		</div>
		<div class="flex items-center gap-2">
			<button class="rounded-lg border border-[color:var(--color-border)] px-3 py-2 text-xs text-[color:var(--color-ash)]" onclick={openPalette}>Command Palette</button>
			<button class="rounded-lg border border-[color:var(--color-border)] px-3 py-2 text-xs text-[color:var(--color-ash)] lg:hidden" onclick={() => (mobileOpen = !mobileOpen)}>Menu</button>
		</div>
	</header>

	<div class="grid gap-4 lg:grid-cols-[230px_1fr]">
		<aside class="surface-panel p-3 {mobileOpen ? 'block' : 'hidden'} lg:block">
			<nav class="space-y-1">
				{#each navItems as item}
					<a
						href={item.href}
						class="block rounded-lg px-3 py-2 text-sm transition-colors {page.url.pathname === item.href ? 'bg-[color:var(--color-gold)]/20 text-[color:var(--color-gold-bright)]' : 'text-[color:var(--color-ash)] hover:bg-white/5 hover:text-[color:var(--color-base-text)]'}"
						onclick={() => (mobileOpen = false)}
					>
						{item.label}
					</a>
				{/each}
			</nav>

			<div class="mt-4 border-t border-[color:var(--color-border)] pt-3">
				<div class="mb-2 flex items-center justify-between">
					<p class="text-[10px] uppercase tracking-[0.16em] text-[color:var(--color-ash)]">Sessions</p>
					<a href="/dashboard/chat" class="text-xs text-[color:var(--color-gold-bright)]">new</a>
				</div>
				<div class="space-y-1">
					{#if sessions.length === 0}
						<p class="px-2 py-1 text-xs text-[color:var(--color-ash)]">No sessions yet</p>
					{:else}
						{#each sessions as session}
							<a href={`/dashboard/chat?session=${session.id}`} class="group flex items-center justify-between rounded-md px-2 py-1.5 text-xs text-[color:var(--color-ash)] hover:bg-white/5">
								<span class="truncate pr-2">{session.target}</span>
								<button class="hidden text-[color:var(--color-rust)] group-hover:block" onclick={(event) => deleteSession(event, session.id)} aria-label="Delete session">x</button>
							</a>
						{/each}
					{/if}
				</div>
			</div>
		</aside>

		<main class="dashboard-main">{@render children()}</main>
	</div>
</div>

{#if showPalette}
	<div class="fixed inset-0 z-50 bg-black/60" role="presentation" onclick={closePalette} onkeydown={() => {}}>
		<div class="mx-auto mt-24 w-full max-w-xl rounded-xl border border-[color:var(--color-border)] bg-[color:var(--color-surface)]" role="dialog" tabindex="-1" onclick={(event) => event.stopPropagation()} onkeydown={onPaletteKeydown}>
			<div class="border-b border-[color:var(--color-border)] px-4 py-3">
				<input bind:value={paletteQuery} placeholder="Search command..." class="w-full bg-transparent text-sm text-[color:var(--color-base-text)] outline-none placeholder:text-[color:var(--color-ash)]" />
			</div>
			<div class="max-h-72 overflow-y-auto p-2">
				{#if filteredPalette.length === 0}
					<p class="px-2 py-3 text-sm text-[color:var(--color-ash)]">No matches</p>
				{:else}
					{#each filteredPalette as item, index}
						<button class="w-full rounded-lg px-3 py-2 text-left text-sm {index === selectedIndex ? 'bg-[color:var(--color-gold)]/20 text-[color:var(--color-gold-bright)]' : 'text-[color:var(--color-ash)] hover:bg-white/5'}" onclick={() => { selectPaletteItem(index); runPaletteSelection(); }}>
							<p>{item.label}</p>
							<p class="text-xs opacity-70">{item.desc}</p>
						</button>
					{/each}
				{/if}
			</div>
		</div>
	</div>
{/if}

<ToastStack />
