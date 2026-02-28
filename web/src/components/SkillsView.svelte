<script lang="ts">
  import { api } from '../lib/api.ts';
  import { renderMarkdown } from '../lib/markdown.ts';
  import { toast } from '../lib/toast.ts';
  import type { SkillSummary, SkillDetail } from '../lib/types';

  let skills = $state<SkillSummary[]>([]);
  let selectedSkill = $state<SkillDetail | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let searchQuery = $state('');
  let viewMode = $state<'list' | 'grid'>('list');

  const filteredSkills = $derived(
    searchQuery.trim()
      ? skills.filter((s) =>
          s.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          s.description.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : skills
  );

  async function loadSkills(): Promise<void> {
    loading = true;
    try {
      skills = await api.listSkills();
      error = null;
    } catch (e) {
      error = (e as Error).message;
    } finally {
      loading = false;
    }
  }

  async function selectSkill(name: string): Promise<void> {
    try {
      selectedSkill = await api.getSkill(name);
      // On mobile/grid, scroll to detail
      if (viewMode === 'grid') {
        const el = document.getElementById('skill-detail');
        el?.scrollIntoView({ behavior: 'smooth' });
      }
    } catch (e) {
      toast.error((e as Error).message);
    }
  }

  async function toggleSkill(name: string, enabled: boolean): Promise<void> {
    try {
      await api.updateSkill(name, enabled);
      // Update local state
      skills = skills.map((s) =>
        s.name === name ? { ...s, enabled } : s
      );
      toast.success(`${name} ${enabled ? 'enabled' : 'disabled'}`);
    } catch {
      toast.error('Failed to update skill');
    }
  }

  loadSkills();
</script>

{#if viewMode === 'list'}
  <!-- List view: sidebar + detail panel -->
  <div class="flex h-full">
    <!-- Skills sidebar -->
    <div class="w-72 border-r border-[var(--border)] bg-[var(--bg-secondary)] overflow-y-auto flex flex-col">
      <div class="p-4 border-b border-[var(--border)]">
        <div class="flex items-center justify-between mb-2">
          <h2 class="text-lg font-bold">🧰 Skills</h2>
          <div class="flex gap-1">
            <button
              class="p-1.5 rounded text-xs transition-colors bg-[var(--accent)]/10 text-[var(--accent)]"
              title="List view"
            >☰</button>
            <button
              class="p-1.5 rounded text-xs transition-colors text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)]"
              onclick={() => viewMode = 'grid'}
              title="Grid view"
            >▦</button>
          </div>
        </div>
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search skills..."
          class="w-full px-3 py-2 text-xs bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
            text-[var(--text-primary)] placeholder-[var(--text-secondary)]/50
            focus:outline-none focus:border-[var(--accent)]"
        />
        <p class="text-[10px] text-[var(--text-secondary)] mt-1.5">{filteredSkills.length} of {skills.length} skills</p>
      </div>

      {#if loading}
        <div class="p-4 text-[var(--text-secondary)] text-sm">Loading...</div>
      {:else if error}
        <div class="p-4 text-[var(--danger)] text-sm">{error}</div>
      {:else if filteredSkills.length === 0}
        <div class="p-4 text-[var(--text-secondary)] text-sm">No skills match "{searchQuery}"</div>
      {:else}
        <div class="p-2 space-y-1 flex-1 overflow-y-auto">
          {#each filteredSkills as skill}
            <button
              class="w-full text-left px-3 py-2.5 rounded-lg transition-colors
                {selectedSkill?.name === skill.name
                  ? 'bg-[var(--accent)]/10 text-[var(--accent)]'
                  : 'text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]'}"
              onclick={() => selectSkill(skill.name)}
            >
              <div class="flex items-center gap-2">
                <div class="text-sm font-medium flex-1 truncate">{skill.name}</div>
                {#if skill.requires_root}
                  <span class="text-[9px] px-1 py-0.5 rounded bg-amber-950/30 text-amber-400 shrink-0">root</span>
                {/if}
                {#if skill.enabled === false}
                  <span class="text-[9px] px-1 py-0.5 rounded bg-red-950/30 text-red-400 shrink-0">off</span>
                {/if}
              </div>
              <div class="text-xs opacity-70 truncate">{skill.description}</div>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Detail panel -->
    <div class="flex-1 overflow-y-auto p-6" id="skill-detail">
      {#if selectedSkill}
        {@render skillDetail(selectedSkill)}
      {:else}
        <div class="flex items-center justify-center h-full text-[var(--text-secondary)]">
          <p>Select a skill to view details</p>
        </div>
      {/if}
    </div>
  </div>
{:else}
  <!-- Grid view -->
  <div class="flex-1 overflow-y-auto p-6">
    <div class="max-w-6xl mx-auto">
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-2xl font-bold">🧰 Skills Catalog</h1>
          <p class="text-xs text-[var(--text-secondary)] mt-1">{filteredSkills.length} of {skills.length} skills</p>
        </div>
        <div class="flex items-center gap-3">
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Search skills..."
            class="px-3 py-2 text-xs bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
              text-[var(--text-primary)] placeholder-[var(--text-secondary)]/50
              focus:outline-none focus:border-[var(--accent)] w-48"
          />
          <div class="flex gap-1">
            <button
              class="p-1.5 rounded text-xs transition-colors text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)]"
              onclick={() => viewMode = 'list'}
              title="List view"
            >☰</button>
            <button
              class="p-1.5 rounded text-xs transition-colors bg-[var(--accent)]/10 text-[var(--accent)]"
              title="Grid view"
            >▦</button>
          </div>
        </div>
      </div>

      {#if loading}
        <div class="text-[var(--text-secondary)] text-sm">Loading...</div>
      {:else if filteredSkills.length === 0}
        <div class="text-[var(--text-secondary)] text-sm">No skills match "{searchQuery}"</div>
      {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-8">
          {#each filteredSkills as skill}
            <button
              class="text-left rounded-xl border p-4 transition-colors
                {selectedSkill?.name === skill.name
                  ? 'bg-[var(--accent)]/5 border-[var(--accent)]/30'
                  : 'bg-[var(--bg-secondary)] border-[var(--border)] hover:border-[var(--accent)]/20'}
                {skill.tool_available === false || skill.enabled === false ? 'opacity-50' : ''}"
              onclick={() => selectSkill(skill.name)}
            >
              <div class="flex items-start justify-between mb-2">
                <h3 class="text-sm font-semibold text-[var(--text-primary)]">{skill.name}</h3>
                <div class="flex gap-1.5">
                  {#if skill.requires_root}
                    <span class="text-[9px] px-1.5 py-0.5 rounded-full bg-amber-950/30 text-amber-400">root</span>
                  {/if}
                  {#if skill.tool_available === false}
                    <span class="text-[9px] px-1.5 py-0.5 rounded-full bg-orange-950/30 text-orange-400">not installed</span>
                  {:else if skill.enabled === false}
                    <span class="text-[9px] px-1.5 py-0.5 rounded-full bg-red-950/30 text-red-400">disabled</span>
                  {/if}
                </div>
              </div>
              <p class="text-xs text-[var(--text-secondary)] line-clamp-2 mb-3">{skill.description}</p>
              {#if skill.tool_path}
                <code class="text-[10px] text-[var(--accent)] opacity-70">{skill.tool_path}</code>
              {/if}
            </button>
          {/each}
        </div>

        <!-- Inline detail when a skill is selected in grid mode -->
        {#if selectedSkill}
          <div id="skill-detail" class="border-t border-[var(--border)] pt-6">
            {@render skillDetail(selectedSkill)}
          </div>
        {/if}
      {/if}
    </div>
  </div>
{/if}

{#snippet skillDetail(skill: SkillDetail)}
  <div class="max-w-3xl">
    <div class="flex items-start justify-between mb-2">
      <h1 class="text-2xl font-bold">{skill.name}</h1>
      {#if skill.tool_available === false}
        <span class="text-xs px-3 py-1.5 rounded-lg bg-orange-950/30 text-orange-400 cursor-not-allowed">Not Installed</span>
      {:else}
        <button
          class="text-xs px-3 py-1.5 rounded-lg transition-colors
            {skills.find((s) => s.name === skill.name)?.enabled === false
              ? 'bg-[var(--success)]/10 text-[var(--success)] hover:bg-[var(--success)]/20'
              : 'bg-[var(--danger)]/10 text-[var(--danger)] hover:bg-[var(--danger)]/20'}"
          onclick={() => {
            const current = skills.find((s) => s.name === skill.name);
            const newEnabled = current?.enabled === false;
            toggleSkill(skill.name, newEnabled);
          }}
        >
          {skills.find((s) => s.name === skill.name)?.enabled === false ? 'Enable' : 'Disable'}
        </button>
      {/if}
    </div>
    {#if skill.tool_available === false}
      <div class="mb-3 px-3 py-2 rounded-lg bg-orange-950/20 border border-orange-900/30 text-orange-400 text-xs">
        ⚠ Tool binary <code class="font-mono">{skill.tool_path}</code> is not installed on this system. Install it to enable this skill.
      </div>
    {/if}
    <p class="text-[var(--text-secondary)] mb-4">{skill.description}</p>

    <div class="grid grid-cols-2 gap-4 mb-6">
      {#if skill.tool_path}
        <div class="bg-[var(--bg-secondary)] rounded-lg p-3 border border-[var(--border)]">
          <div class="text-xs text-[var(--text-secondary)] mb-1">Tool</div>
          <div class="flex items-center gap-2">
            <code class="text-sm text-[var(--accent)]">{skill.tool_path}</code>
            {#if skill.tool_available === false}
              <span class="text-[9px] px-1.5 py-0.5 rounded-full bg-orange-950/30 text-orange-400">missing</span>
            {:else}
              <span class="text-[9px] px-1.5 py-0.5 rounded-full bg-green-950/30 text-green-400">found</span>
            {/if}
          </div>
        </div>
      {/if}
      <div class="bg-[var(--bg-secondary)] rounded-lg p-3 border border-[var(--border)]">
        <div class="text-xs text-[var(--text-secondary)] mb-1">Root Required</div>
        <span class="text-sm {skill.requires_root ? 'text-[var(--warning)]' : 'text-[var(--success)]'}">
          {skill.requires_root ? 'Yes' : 'No'}
        </span>
      </div>
    </div>

    {#if skill.args?.length}
      <div class="mb-6">
        <h3 class="text-sm font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">Arguments</h3>
        <pre class="text-xs">{skill.args.join(' ')}</pre>
      </div>
    {/if}

    {#if skill.role}
      <div class="mb-4">
        <h3 class="text-sm font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">Role</h3>
        <div class="markdown-content bg-[var(--bg-secondary)] rounded-lg p-4 border border-[var(--border)] text-sm">
          {@html renderMarkdown(skill.role)}
        </div>
      </div>
    {/if}

    {#if skill.task}
      <div class="mb-4">
        <h3 class="text-sm font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">Task</h3>
        <div class="markdown-content bg-[var(--bg-secondary)] rounded-lg p-4 border border-[var(--border)] text-sm">
          {@html renderMarkdown(skill.task)}
        </div>
      </div>
    {/if}

    {#if skill.constraints}
      <div class="mb-4">
        <h3 class="text-sm font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">Constraints</h3>
        <div class="markdown-content bg-[var(--bg-secondary)] rounded-lg p-4 border border-[var(--border)] text-sm">
          {@html renderMarkdown(skill.constraints)}
        </div>
      </div>
    {/if}

    <div>
      <h3 class="text-sm font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">System Prompt</h3>
      <div class="markdown-content bg-[var(--bg-secondary)] rounded-lg p-4 border border-[var(--border)] text-sm">
        {@html renderMarkdown(skill.system_prompt)}
      </div>
    </div>
  </div>
{/snippet}
