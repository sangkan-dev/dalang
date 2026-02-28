<script lang="ts">
  import type { PageId, NavItem } from '../lib/types';

  let { currentPage, onNavigate }: { currentPage: PageId; onNavigate: (page: PageId) => void } = $props();

  const navItems: NavItem[] = [
    { id: 'chat', label: 'Chat', icon: '💬' },
    { id: 'skills', label: 'Skills', icon: '🧰' },
    { id: 'reports', label: 'Reports', icon: '📋' },
    { id: 'settings', label: 'Settings', icon: '⚙️' },
  ];
</script>

<aside class="w-16 lg:w-56 bg-[var(--bg-secondary)] border-r border-[var(--border)] flex flex-col shrink-0">
  <!-- Logo -->
  <div class="p-4 border-b border-[var(--border)] flex items-center gap-3">
    <span class="text-2xl">🛡️</span>
    <span class="hidden lg:block text-lg font-bold text-[var(--accent)]">Dalang</span>
  </div>

  <!-- Navigation -->
  <nav class="flex-1 p-2 space-y-1">
    {#each navItems as item}
      <button
        class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors text-left
          {currentPage === item.id
            ? 'bg-[var(--accent)]/10 text-[var(--accent)]'
            : 'text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]'}"
        onclick={() => onNavigate(item.id)}
      >
        <span class="text-lg">{item.icon}</span>
        <span class="hidden lg:block text-sm font-medium">{item.label}</span>
      </button>
    {/each}
  </nav>

  <!-- Footer -->
  <div class="p-3 border-t border-[var(--border)] text-center">
    <span class="text-xs text-[var(--text-secondary)] hidden lg:block">Dalang v0.1.0</span>
  </div>
</aside>
