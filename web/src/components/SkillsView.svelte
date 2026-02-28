<script lang="ts">
  import { api } from '../lib/api.ts';
  import { renderMarkdown } from '../lib/markdown.ts';
  import type { SkillSummary, SkillDetail } from '../lib/types';

  let skills = $state<SkillSummary[]>([]);
  let selectedSkill = $state<SkillDetail | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

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
    } catch (e) {
      error = (e as Error).message;
    }
  }

  // Load on mount
  loadSkills();
</script>

<div class="flex h-full">
  <!-- Skills list -->
  <div class="w-72 border-r border-[var(--border)] bg-[var(--bg-secondary)] overflow-y-auto">
    <div class="p-4 border-b border-[var(--border)]">
      <h2 class="text-lg font-bold">🧰 Skills Catalog</h2>
      <p class="text-xs text-[var(--text-secondary)] mt-1">{skills.length} skills available</p>
    </div>

    {#if loading}
      <div class="p-4 text-[var(--text-secondary)] text-sm">Loading...</div>
    {:else if error}
      <div class="p-4 text-[var(--danger)] text-sm">{error}</div>
    {:else}
      <div class="p-2 space-y-1">
        {#each skills as skill}
          <button
            class="w-full text-left px-3 py-2.5 rounded-lg transition-colors
              {selectedSkill?.name === skill.name
                ? 'bg-[var(--accent)]/10 text-[var(--accent)]'
                : 'text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]'}"
            onclick={() => selectSkill(skill.name)}
          >
            <div class="text-sm font-medium">{skill.name}</div>
            <div class="text-xs opacity-70 truncate">{skill.description}</div>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Skill detail -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if selectedSkill}
      <div class="max-w-3xl">
        <h1 class="text-2xl font-bold mb-2">{selectedSkill.name}</h1>
        <p class="text-[var(--text-secondary)] mb-4">{selectedSkill.description}</p>

        <div class="grid grid-cols-2 gap-4 mb-6">
          {#if selectedSkill.tool_path}
            <div class="bg-[var(--bg-secondary)] rounded-lg p-3 border border-[var(--border)]">
              <div class="text-xs text-[var(--text-secondary)] mb-1">Tool</div>
              <code class="text-sm text-[var(--accent)]">{selectedSkill.tool_path}</code>
            </div>
          {/if}
          {#if selectedSkill.requires_root !== undefined}
            <div class="bg-[var(--bg-secondary)] rounded-lg p-3 border border-[var(--border)]">
              <div class="text-xs text-[var(--text-secondary)] mb-1">Root Required</div>
              <span class="text-sm {selectedSkill.requires_root ? 'text-[var(--warning)]' : 'text-[var(--success)]'}">
                {selectedSkill.requires_root ? 'Yes' : 'No'}
              </span>
            </div>
          {/if}
        </div>

        {#if selectedSkill.args?.length}
          <div class="mb-6">
            <h3 class="text-sm font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">Arguments</h3>
            <pre class="text-xs">{selectedSkill.args.join(' ')}</pre>
          </div>
        {/if}

        {#if selectedSkill.role}
          <div class="mb-4">
            <h3 class="text-sm font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">Role</h3>
            <div class="markdown-content bg-[var(--bg-secondary)] rounded-lg p-4 border border-[var(--border)] text-sm">
              {@html renderMarkdown(selectedSkill.role)}
            </div>
          </div>
        {/if}

        {#if selectedSkill.task}
          <div class="mb-4">
            <h3 class="text-sm font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">Task</h3>
            <div class="markdown-content bg-[var(--bg-secondary)] rounded-lg p-4 border border-[var(--border)] text-sm">
              {@html renderMarkdown(selectedSkill.task)}
            </div>
          </div>
        {/if}

        {#if selectedSkill.constraints}
          <div class="mb-4">
            <h3 class="text-sm font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">Constraints</h3>
            <div class="markdown-content bg-[var(--bg-secondary)] rounded-lg p-4 border border-[var(--border)] text-sm">
              {@html renderMarkdown(selectedSkill.constraints)}
            </div>
          </div>
        {/if}

        <div>
          <h3 class="text-sm font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">System Prompt</h3>
          <div class="markdown-content bg-[var(--bg-secondary)] rounded-lg p-4 border border-[var(--border)] text-sm">
            {@html renderMarkdown(selectedSkill.system_prompt)}
          </div>
        </div>
      </div>
    {:else}
      <div class="flex items-center justify-center h-full text-[var(--text-secondary)]">
        <p>Select a skill to view details</p>
      </div>
    {/if}
  </div>
</div>
