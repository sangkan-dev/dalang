<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { apiClient } from '$lib/api/client.js';
	import type { Session } from '$lib/api/types.js';
	import ToastStack from '$lib/dashboard/ToastStack.svelte';
	import { toast } from '$lib/dashboard/toast.js';
	import DashboardNav from '$lib/features/layout/DashboardNav.svelte';
	import SessionsList from '$lib/features/layout/SessionsList.svelte';
	import CommandPalette from '$lib/features/layout/CommandPalette.svelte';

	const { children } = $props();

	let sessions = $state<Session[]>([]);
	let mobileOpen = $state(false);
	let showPalette = $state(false);

	let paletteQuery = $state('');
	let selectedIndex = $state(0);
	type DashboardRoute =
		| '/dashboard'
		| '/dashboard/chat'
		| '/dashboard/skills'
		| '/dashboard/reports'
		| '/dashboard/settings';

	const navItems: Array<{ route: DashboardRoute; label: string }> = [
		{ route: '/dashboard', label: 'Ringkasan' },
		{ route: '/dashboard/chat', label: 'Percakapan pemeriksaan' },
		{ route: '/dashboard/skills', label: 'Alat pemeriksaan' },
		{ route: '/dashboard/reports', label: 'Laporan' },
		{ route: '/dashboard/settings', label: 'Pengaturan' }
	];

	const paletteItems: Array<{ route: DashboardRoute; label: string; desc: string }> = [
		{
			route: '/dashboard',
			label: 'Buka ringkasan',
			desc: 'Angka singkat dan pintasan ke fitur utama'
		},
		{
			route: '/dashboard/chat',
			label: 'Percakapan pemeriksaan',
			desc: 'Mulai atau lanjutkan pemeriksaan keamanan'
		},
		{
			route: '/dashboard/skills',
			label: 'Alat pemeriksaan',
			desc: 'Daftar kemampuan yang dipakai AI'
		},
		{
			route: '/dashboard/reports',
			label: 'Laporan',
			desc: 'Arsip laporan yang sudah dibuat'
		},
		{
			route: '/dashboard/settings',
			label: 'Pengaturan',
			desc: 'Penyedia AI dan model'
		},
		{
			route: '/dashboard/chat',
			label: 'Sesi baru',
			desc: 'Buka halaman chat untuk target baru'
		}
	];

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
			toast.success('Sesi dihapus');
		} catch (error) {
			toast.error(
				`Gagal menghapus sesi: ${error instanceof Error ? error.message : 'tidak diketahui'}`
			);
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

	function toggleMobileMenu(): void {
		mobileOpen = !mobileOpen;
	}

	function closeMobileMenu(): void {
		mobileOpen = false;
	}

	function onPaletteInput(): void {
		selectedIndex = 0;
	}

	function onPaletteQueryChange(value: string): void {
		paletteQuery = value;
	}

	async function openSession(id: string): Promise<void> {
		window.location.assign(`${resolve('/dashboard/chat')}?session=${id}`);
		closeMobileMenu();
	}

	function selectPaletteItem(index: number): void {
		selectedIndex = index;
	}

	async function runPaletteSelection(): Promise<void> {
		const item = filteredPalette[selectedIndex];
		if (!item) return;
		await goto(resolve(item.route));
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
			void runPaletteSelection();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			closePalette();
		}
	}

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
	<header class="dashboard-header dashboard-command-frame">
		<div>
			<p class="dashboard-kicker">DASBOR DALANG</p>
			<h1 class="dashboard-title">Pemeriksaan keamanan situs</h1>
		</div>
		<div class="flex items-center gap-2">
			<button
				class="inline-flex items-center justify-center rounded-md border border-(--color-gold)/30 bg-white/5 px-3 py-2 font-mono text-xs tracking-[0.08em] text-(--color-gold-bright) uppercase transition hover:-translate-y-px hover:border-(--color-gold)/50"
				title="Ctrl+K / ⌘K"
				onclick={openPalette}>Cari halaman</button
			>
			<button
				class="inline-flex items-center justify-center rounded-md border border-(--color-gold)/30 bg-white/5 px-3 py-2 font-mono text-xs tracking-[0.08em] text-(--color-gold-bright) uppercase transition hover:-translate-y-px hover:border-(--color-gold)/50 lg:hidden"
				onclick={toggleMobileMenu}>Menu</button
			>
		</div>
	</header>

		<div class="relative">
			{#if mobileOpen}
				<div
					class="fixed inset-0 z-40 bg-black/60 lg:hidden"
					role="presentation"
					onclick={closeMobileMenu}
					onkeydown={() => {}}
				></div>
			{/if}

			<div class="grid items-start gap-4 lg:grid-cols-[minmax(230px,280px)_minmax(0,1fr)]">
				<aside
					class="z-50 space-y-4 lg:z-auto lg:block lg:static lg:translate-x-0
					{mobileOpen ? 'fixed left-3 top-24 block w-[min(18rem,calc(100%-1.5rem))] translate-x-0' : 'hidden -translate-x-2'} lg:!w-auto lg:!top-auto"
				>
					<div class="surface-panel dashboard-nav-panel p-3">
						<DashboardNav {navItems} activePath={page.url.pathname} onNavigate={closeMobileMenu} />
					</div>
					<div class="surface-panel p-3">
						<SessionsList {sessions} onOpenSession={openSession} onDeleteSession={deleteSession} />
					</div>
				</aside>

				<main class="dashboard-main min-w-0">{@render children()}</main>
			</div>
		</div>
</div>

<CommandPalette
	{showPalette}
	{paletteQuery}
	{selectedIndex}
	{filteredPalette}
	{onPaletteInput}
	{onPaletteQueryChange}
	onClose={closePalette}
	onKeydown={onPaletteKeydown}
	onSelectItem={selectPaletteItem}
	onRunSelection={runPaletteSelection}
/>

<ToastStack />
