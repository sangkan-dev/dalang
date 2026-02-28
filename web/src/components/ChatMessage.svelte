<script lang="ts">
  import { renderMarkdown } from '../lib/markdown.ts';
  import type { ChatMessage, MessageRole, RoleConfig } from '../lib/types';

  let { message }: { message: ChatMessage } = $props();

  const roleConfig: Record<MessageRole, RoleConfig> = {
    user: { label: 'You', icon: '👤', bg: 'bg-[var(--accent)]/5', border: 'border-[var(--accent)]/20' },
    assistant: { label: 'Agent', icon: '🛡️', bg: 'bg-[var(--bg-secondary)]', border: 'border-[var(--border)]' },
    status: { label: 'System', icon: 'ℹ️', bg: 'bg-transparent', border: 'border-transparent' },
    tool: { label: 'Tool', icon: '🔧', bg: 'bg-emerald-950/30', border: 'border-emerald-800/30' },
    observation: { label: 'Output', icon: '📄', bg: 'bg-[var(--bg-primary)]', border: 'border-[var(--border)]' },
    warning: { label: 'Warning', icon: '⚠️', bg: 'bg-amber-950/30', border: 'border-amber-800/30' },
    error: { label: 'Error', icon: '❌', bg: 'bg-red-950/30', border: 'border-red-800/30' },
    report: { label: 'Report', icon: '📋', bg: 'bg-[var(--bg-secondary)]', border: 'border-[var(--accent)]/30' },
  };

  const config = $derived(roleConfig[message.role] || roleConfig.status);
  const isCompact = $derived(message.role === 'status' || message.role === 'warning');
  let expanded = $state(false);
</script>

{#if isCompact}
  <div class="flex items-center gap-2 text-xs text-[var(--text-secondary)] py-0.5">
    <span>{config.icon}</span>
    <span>{message.content}</span>
  </div>
{:else}
  <div class="rounded-xl border p-4 {config.bg} {config.border}">
    <div class="flex items-start gap-3">
      <span class="text-lg mt-0.5 shrink-0">{config.icon}</span>
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2 mb-1">
          <span class="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wide">{config.label}</span>
          {#if message.bytes}
            <span class="text-xs text-[var(--text-secondary)]/60">({message.bytes} bytes)</span>
          {/if}
          {#if message.filename}
            <span class="text-xs text-[var(--accent)]">📎 {message.filename}</span>
          {/if}
        </div>

        {#if message.role === 'observation'}
          <!-- Collapsible observation output -->
          <button
            class="text-xs text-[var(--accent)] hover:underline mb-1"
            onclick={() => expanded = !expanded}
          >
            {expanded ? '▼ Collapse' : '▶ Show output'} ({message.skill})
          </button>
          {#if expanded}
            <pre class="text-xs mt-2 max-h-96 overflow-auto">{message.content}</pre>
          {/if}
        {:else if message.role === 'error'}
          <p class="text-sm text-[var(--danger)]">{message.content}</p>
        {:else}
          <div class="markdown-content text-sm leading-relaxed">
            {@html renderMarkdown(message.content)}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
