<script lang="ts">
  import type { PageId } from '../lib/types';

  let { currentPage, onNavigate, onClose }: {
    currentPage: PageId;
    onNavigate: (page: PageId) => void;
    onClose: () => void;
  } = $props();

  let query = $state('');
  let inputEl = $state<HTMLInputElement | null>(null);

  interface PaletteItem {
    id: string;
    label: string;
    icon: string;
    description: string;
    action: () => void;
  }

  const items: PaletteItem[] = [
    { id: 'chat', label: 'Chat', icon: '💬', description: 'Open chat / new session', action: () => onNavigate('chat') },
    { id: 'skills', label: 'Skills', icon: '🧰', description: 'Browse skill catalog', action: () => onNavigate('skills') },
    { id: 'reports', label: 'Reports', icon: '📋', description: 'View scan reports', action: () => onNavigate('reports') },
    { id: 'settings', label: 'Settings', icon: '⚙️', description: 'Application settings', action: () => onNavigate('settings') },
  ];

  const filtered = $derived(
    query.trim()
      ? items.filter((i) =>
          i.label.toLowerCase().includes(query.toLowerCase()) ||
          i.description.toLowerCase().includes(query.toLowerCase())
        )
      : items
  );

  let selectedIndex = $state(0);

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === 'Enter' && filtered.length > 0) {
      e.preventDefault();
      filtered[selectedIndex].action();
    } else if (e.key === 'Escape') {
      onClose();
    }
  }

  // Reset selection when query changes
  $effect(() => {
    query;
    selectedIndex = 0;
  });

  // Focus input on mount
  $effect(() => {
    inputEl?.focus();
  });
</script>

<!-- Overlay -->
<div
  class="fixed inset-0 bg-black/60 z-50 flex items-start justify-center pt-[20vh]"
  role="presentation"
  onclick={onClose}
  onkeydown={() => {}}
>
  <div
    class="w-full max-w-md bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl shadow-2xl overflow-hidden"
    role="dialog"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={() => {}}
  >
    <!-- Search input -->
    <div class="flex items-center gap-3 px-4 py-3 border-b border-[var(--border)]">
      <span class="text-[var(--text-secondary)]">🔍</span>
      <input
        bind:this={inputEl}
        type="text"
        bind:value={query}
        placeholder="Search pages..."
        class="flex-1 bg-transparent text-[var(--text-primary)] placeholder-[var(--text-secondary)]/50
          text-sm focus:outline-none"
        onkeydown={handleKeydown}
      />
      <kbd class="text-[10px] px-1.5 py-0.5 rounded bg-[var(--bg-tertiary)] text-[var(--text-secondary)]">Esc</kbd>
    </div>

    <!-- Results -->
    <div class="max-h-64 overflow-y-auto p-1">
      {#each filtered as item, i (item.id)}
        <button
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-colors
            {i === selectedIndex
              ? 'bg-[var(--accent)]/10 text-[var(--accent)]'
              : 'text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)]'}"
          onclick={() => item.action()}
          onmouseenter={() => selectedIndex = i}
        >
          <span class="text-lg">{item.icon}</span>
          <div>
            <div class="text-sm font-medium">{item.label}</div>
            <div class="text-xs opacity-60">{item.description}</div>
          </div>
          {#if currentPage === item.id}
            <span class="ml-auto text-[10px] px-1.5 py-0.5 rounded-full bg-[var(--bg-tertiary)] text-[var(--text-secondary)]">Active</span>
          {/if}
        </button>
      {/each}
      {#if filtered.length === 0}
        <div class="px-4 py-6 text-center text-sm text-[var(--text-secondary)]">No results found</div>
      {/if}
    </div>

    <!-- Footer hint -->
    <div class="px-4 py-2 border-t border-[var(--border)] flex items-center gap-4 text-[10px] text-[var(--text-secondary)]">
      <span>↑↓ Navigate</span>
      <span>↵ Select</span>
      <span>Esc Close</span>
    </div>
  </div>
</div>
