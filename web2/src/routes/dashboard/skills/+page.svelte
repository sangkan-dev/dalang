<script lang="ts">
	import { apiClient } from '$lib/api/client.js';
	import { renderMarkdown } from '$lib/markdown.js';
	import type { SkillDetail, SkillSummary } from '$lib/api/types.js';
	import { toast } from '$lib/dashboard/toast.js';
	import { onMount } from 'svelte';

	let loading = $state(true);
	let error = $state('');
	let search = $state('');
	let viewMode = $state<'list' | 'grid'>('list');
	let skills = $state<SkillSummary[]>([]);
	let selectedSkill = $state<SkillDetail | null>(null);

	const filteredSkills = $derived.by(() => {
		const query = search.trim().toLowerCase();
		if (!query) return skills;
		return skills.filter((skill) => {
			return skill.name.toLowerCase().includes(query) || skill.description.toLowerCase().includes(query);
		});
	});

	async function loadSkills(): Promise<void> {
		loading = true;
		try {
			skills = await apiClient.listSkills();
			error = '';
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load skills';
		} finally {
			loading = false;
		}
	}

	async function selectSkill(name: string): Promise<void> {
		try {
			selectedSkill = await apiClient.getSkill(name);
		} catch (err) {
			toast.error(`Failed to load skill: ${err instanceof Error ? err.message : 'unknown error'}`);
		}
	}

	async function toggleSkill(name: string, enabled: boolean): Promise<void> {
		try {
			await apiClient.updateSkill(name, enabled);
			skills = skills.map((skill) => (skill.name === name ? { ...skill, enabled } : skill));
			toast.success(`${name} ${enabled ? 'enabled' : 'disabled'}`);
		} catch (err) {
			toast.error(`Failed to update skill: ${err instanceof Error ? err.message : 'unknown error'}`);
		}
	}

	onMount(loadSkills);
	</script>

	<section class="space-y-4">
		<header class="flex items-center justify-between gap-3">
			<div>
				<p class="text-xs uppercase tracking-[0.2em] text-[color:var(--color-ash)]">Dashboard / Skills</p>
				<h2 class="text-xl font-semibold text-[color:var(--color-text)]">Skill Catalog</h2>
			</div>
			<div class="flex gap-2">
				<input bind:value={search} placeholder="Search skills..." class="rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-base-text)]" />
				<button class="rounded-lg border border-[color:var(--color-border)] px-3 py-2 text-xs text-[color:var(--color-ash)]" onclick={() => (viewMode = viewMode === 'list' ? 'grid' : 'list')}>
					{viewMode === 'list' ? 'Grid' : 'List'}
				</button>
			</div>
		</header>

		{#if loading}
			<div class="surface-panel p-4 text-sm text-[color:var(--color-ash)]">Loading skills...</div>
		{:else if error}
			<div class="surface-panel p-4 text-sm text-[color:var(--color-rust)]">{error}</div>
		{:else if viewMode === 'list'}
			<div class="grid gap-3 lg:grid-cols-[320px_1fr]">
				<div class="surface-panel max-h-[72vh] overflow-auto p-2">
					{#if filteredSkills.length === 0}
						<p class="px-2 py-2 text-sm text-[color:var(--color-ash)]">No matching skills</p>
					{:else}
						{#each filteredSkills as skill}
							<button class="mb-1 w-full rounded-lg border px-3 py-2 text-left {selectedSkill?.name === skill.name ? 'border-[color:var(--color-gold)]/40 bg-[color:var(--color-gold)]/10' : 'border-transparent hover:border-[color:var(--color-border)]'}" onclick={() => selectSkill(skill.name)}>
								<div class="flex items-center justify-between gap-2">
									<p class="truncate text-sm font-semibold text-[color:var(--color-base-text)]">{skill.name}</p>
									{#if skill.enabled === false}
										<span class="rounded bg-[color:var(--color-rust)]/20 px-1.5 py-0.5 text-[10px] text-[color:var(--color-rust)]">off</span>
									{/if}
								</div>
								<p class="truncate text-xs text-[color:var(--color-ash)]">{skill.description}</p>
							</button>
						{/each}
					{/if}
				</div>

				<div class="surface-panel max-h-[72vh] overflow-auto p-4">
					{#if selectedSkill}
						<div class="mb-3 flex items-start justify-between gap-3">
							<div>
								<h3 class="text-lg font-semibold text-[color:var(--color-base-text)]">{selectedSkill.name}</h3>
								<p class="text-sm text-[color:var(--color-ash)]">{selectedSkill.description}</p>
							</div>
							<button class="rounded-lg border border-[color:var(--color-border)] px-3 py-2 text-xs text-[color:var(--color-ash)]" onclick={() => {
								const current = skills.find((entry) => entry.name === selectedSkill?.name);
								if (!current) return;
								toggleSkill(current.name, current.enabled === false);
							}}>
								{skills.find((entry) => entry.name === selectedSkill?.name)?.enabled === false ? 'Enable' : 'Disable'}
							</button>
						</div>

						<div class="space-y-2 text-sm">
							<p><span class="text-[color:var(--color-ash)]">Tool:</span> <code>{selectedSkill.tool_path ?? 'Browser-native'}</code></p>
							<p><span class="text-[color:var(--color-ash)]">Root required:</span> {selectedSkill.requires_root ? 'Yes' : 'No'}</p>
							{#if selectedSkill.args?.length}
								<p><span class="text-[color:var(--color-ash)]">Args:</span> <code>{selectedSkill.args.join(' ')}</code></p>
							{/if}
						</div>

						<div class="mt-4 space-y-3">
							{#if selectedSkill.role}
								<article class="rounded-lg border border-[color:var(--color-border)] p-3">
									<p class="mb-1 text-xs uppercase tracking-[0.12em] text-[color:var(--color-ash)]">Role</p>
									<div class="dashboard-markdown text-xs" dir="auto">
										{@html renderMarkdown(selectedSkill.role)}
									</div>
								</article>
							{/if}
							{#if selectedSkill.task}
								<article class="rounded-lg border border-[color:var(--color-border)] p-3">
									<p class="mb-1 text-xs uppercase tracking-[0.12em] text-[color:var(--color-ash)]">Task</p>
									<div class="dashboard-markdown text-xs" dir="auto">
										{@html renderMarkdown(selectedSkill.task)}
									</div>
								</article>
							{/if}
							<article class="rounded-lg border border-[color:var(--color-border)] p-3">
								<p class="mb-1 text-xs uppercase tracking-[0.12em] text-[color:var(--color-ash)]">System Prompt</p>
								<div class="dashboard-markdown text-xs" dir="auto">
									{@html renderMarkdown(selectedSkill.system_prompt)}
								</div>
							</article>
						</div>
					{:else}
						<p class="text-sm text-[color:var(--color-ash)]">Select a skill to view details.</p>
					{/if}
				</div>
			</div>
		{:else}
			<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
				{#each filteredSkills as skill}
					<button class="surface-panel p-4 text-left" onclick={() => selectSkill(skill.name)}>
						<div class="mb-1 flex items-center justify-between gap-2">
							<p class="truncate text-sm font-semibold text-[color:var(--color-base-text)]">{skill.name}</p>
							{#if skill.enabled === false}
								<span class="rounded bg-[color:var(--color-rust)]/20 px-1.5 py-0.5 text-[10px] text-[color:var(--color-rust)]">off</span>
							{/if}
						</div>
						<p class="text-xs text-[color:var(--color-ash)]">{skill.description}</p>
					</button>
				{/each}
			</div>
		{/if}
	</section>
