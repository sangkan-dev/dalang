<script lang="ts">
	import { apiClient } from '$lib/api/client.js';
	import { renderMarkdown, renderMarkdownRaw } from '$lib/markdown.js';
	import type { SkillDetail, SkillSummary } from '$lib/api/types.js';
	import { toast } from '$lib/dashboard/toast.js';
	import { onMount } from 'svelte';

	let loading = $state(true);
	let error = $state('');
	let search = $state('');
	let viewMode = $state<'list' | 'grid'>('list');
	let detailMode = $state<'formatted' | 'raw'>('formatted');
	let skills = $state<SkillSummary[]>([]);
	let selectedSkill = $state<SkillDetail | null>(null);

	const filteredSkills = $derived.by(() => {
		const query = search.trim().toLowerCase();
		if (!query) return skills;
		return skills.filter((skill) => {
			return (
				skill.name.toLowerCase().includes(query) || skill.description.toLowerCase().includes(query)
			);
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
			detailMode = 'formatted';
		} catch (err) {
			toast.error(`Failed to load skill: ${err instanceof Error ? err.message : 'unknown error'}`);
		}
	}

	function toggleViewMode(): void {
		viewMode = viewMode === 'list' ? 'grid' : 'list';
	}

	function setDetailModeFormatted(): void {
		detailMode = 'formatted';
	}

	function setDetailModeRaw(): void {
		detailMode = 'raw';
	}

	async function toggleSkill(name: string, enabled: boolean): Promise<void> {
		try {
			await apiClient.updateSkill(name, enabled);
			skills = skills.map((skill) => (skill.name === name ? { ...skill, enabled } : skill));
			toast.success(`${name} ${enabled ? 'enabled' : 'disabled'}`);
		} catch (err) {
			toast.error(
				`Failed to update skill: ${err instanceof Error ? err.message : 'unknown error'}`
			);
		}
	}

	onMount(loadSkills);
</script>

<section class="space-y-4">
	<header
		class="surface-panel dashboard-warboard flex flex-wrap items-center justify-between gap-3 p-4"
	>
		<div>
			<p class="text-xs tracking-[0.2em] text-(--color-ash) uppercase">Dashboard / Skills</p>
			<h2 class="text-xl font-semibold text-(--color-text)">Skill Catalog</h2>
			<p class="text-xs text-(--color-ash)">
				Composable operator primitives and execution constraints.
			</p>
		</div>
		<div class="flex gap-2">
			<input
				bind:value={search}
				placeholder="Search skills..."
				class="rounded-lg border border-(--color-border) bg-transparent px-3 py-2 text-sm text-(--color-base-text)"
			/>
			<button class="control-chip" onclick={toggleViewMode}>
				{viewMode === 'list' ? 'Grid' : 'List'}
			</button>
		</div>
	</header>

	{#if loading}
		<div class="surface-panel p-4 text-sm text-(--color-ash)">Loading skills...</div>
	{:else if error}
		<div class="surface-panel p-4 text-sm text-(--color-rust)">{error}</div>
	{:else if viewMode === 'list'}
		<div class="grid gap-3 lg:grid-cols-[320px_1fr]">
			<div class="surface-panel max-h-[72vh] overflow-auto p-2">
				{#if filteredSkills.length === 0}
					<p class="px-2 py-2 text-sm text-(--color-ash)">No matching skills</p>
				{:else}
					{#each filteredSkills as skill (skill.name)}
						<button
							class="mb-1 w-full rounded-lg border px-3 py-2 text-left {selectedSkill?.name ===
							skill.name
								? 'border-(--color-gold)/40 bg-(--color-gold)/10'
								: 'border-transparent hover:border-(--color-border)'}"
							onclick={() => selectSkill(skill.name)}
						>
							<div class="flex items-center justify-between gap-2">
								<p class="truncate text-sm font-semibold text-(--color-base-text)">
									{skill.name}
								</p>
								{#if skill.enabled === false}
									<span
										class="rounded bg-(--color-rust)/20 px-1.5 py-0.5 text-[10px] text-(--color-rust)"
										>off</span
									>
								{/if}
							</div>
							<p class="truncate text-xs text-(--color-ash)">{skill.description}</p>
						</button>
					{/each}
				{/if}
			</div>

			<div class="surface-panel max-h-[72vh] overflow-auto p-4">
				{#if selectedSkill}
					<div class="mb-3 flex items-start justify-between gap-3">
						<div>
							<h3 class="text-lg font-semibold text-(--color-base-text)">
								{selectedSkill.name}
							</h3>
							<p class="text-sm text-(--color-ash)">{selectedSkill.description}</p>
						</div>
						<button
							class="rounded-lg border border-(--color-border) px-3 py-2 text-xs text-(--color-ash)"
							onclick={() => {
								const current = skills.find((entry) => entry.name === selectedSkill?.name);
								if (!current) return;
								toggleSkill(current.name, current.enabled === false);
							}}
						>
							{skills.find((entry) => entry.name === selectedSkill?.name)?.enabled === false
								? 'Enable'
								: 'Disable'}
						</button>
					</div>

					<div class="mb-3 flex items-center gap-2">
						<button
							class="control-chip"
							onclick={setDetailModeFormatted}
							disabled={detailMode === 'formatted'}>Formatted</button
						>
						<button class="control-chip" onclick={setDetailModeRaw} disabled={detailMode === 'raw'}
							>Raw</button
						>
					</div>

					<div class="space-y-2 text-sm">
						<p>
							<span class="text-(--color-ash)">Tool:</span>
							<code>{selectedSkill.tool_path ?? 'Browser-native'}</code>
						</p>
						<p>
							<span class="text-(--color-ash)">Root required:</span>
							{selectedSkill.requires_root ? 'Yes' : 'No'}
						</p>
						{#if selectedSkill.args?.length}
							<p>
								<span class="text-(--color-ash)">Args:</span>
								<code>{selectedSkill.args.join(' ')}</code>
							</p>
						{/if}
					</div>

					<div class="mt-4 space-y-3">
						{#if selectedSkill.role}
							<article class="rounded-lg border border-(--color-border) p-3">
								<p class="mb-1 text-xs tracking-[0.12em] text-(--color-ash) uppercase">Role</p>
								{#if detailMode === 'raw'}
									<pre class="dashboard-raw text-xs" dir="auto">{renderMarkdownRaw(
											selectedSkill.role
										)}</pre>
								{:else}
									<div class="dashboard-markdown text-xs" dir="auto">
										<!-- eslint-disable-next-line svelte/no-at-html-tags -->
										{@html renderMarkdown(selectedSkill.role)}
									</div>
								{/if}
							</article>
						{/if}
						{#if selectedSkill.task}
							<article class="rounded-lg border border-(--color-border) p-3">
								<p class="mb-1 text-xs tracking-[0.12em] text-(--color-ash) uppercase">Task</p>
								{#if detailMode === 'raw'}
									<pre class="dashboard-raw text-xs" dir="auto">{renderMarkdownRaw(
											selectedSkill.task
										)}</pre>
								{:else}
									<div class="dashboard-markdown text-xs" dir="auto">
										<!-- eslint-disable-next-line svelte/no-at-html-tags -->
										{@html renderMarkdown(selectedSkill.task)}
									</div>
								{/if}
							</article>
						{/if}
						<article class="rounded-lg border border-(--color-border) p-3">
							<p class="mb-1 text-xs tracking-[0.12em] text-(--color-ash) uppercase">
								System Prompt
							</p>
							{#if detailMode === 'raw'}
								<pre class="dashboard-raw text-xs" dir="auto">{renderMarkdownRaw(
										selectedSkill.system_prompt
									)}</pre>
							{:else}
								<div class="dashboard-markdown text-xs" dir="auto">
									<!-- eslint-disable-next-line svelte/no-at-html-tags -->
									{@html renderMarkdown(selectedSkill.system_prompt)}
								</div>
							{/if}
						</article>
					</div>
				{:else}
					<p class="text-sm text-(--color-ash)">Select a skill to view details.</p>
				{/if}
			</div>
		</div>
	{:else}
		<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
			{#each filteredSkills as skill (skill.name)}
				<button class="surface-panel p-4 text-left" onclick={() => selectSkill(skill.name)}>
					<div class="mb-1 flex items-center justify-between gap-2">
						<p class="truncate text-sm font-semibold text-(--color-base-text)">
							{skill.name}
						</p>
						{#if skill.enabled === false}
							<span
								class="rounded bg-(--color-rust)/20 px-1.5 py-0.5 text-[10px] text-(--color-rust)"
								>off</span
							>
						{/if}
					</div>
					<p class="text-xs text-(--color-ash)">{skill.description}</p>
				</button>
			{/each}
		</div>
	{/if}
</section>
