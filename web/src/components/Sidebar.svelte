<script lang="ts">
  import { api } from '../lib/api.ts';
  import { toggleTheme, getTheme } from '../lib/theme.ts';
  import type { PageId, NavItem, Session } from '../lib/types';

  let {
    currentPage,
    onNavigate,
    onSelectSession,
    onNewSession,
  }: {
    currentPage: PageId;
    onNavigate: (page: PageId) => void;
    onSelectSession: (session: Session) => void;
    onNewSession: () => void;
  } = $props();

  const navItems: NavItem[] = [
    { id: 'chat', label: 'Chat', icon: '💬' },
    { id: 'skills', label: 'Skills', icon: '🧰' },
    { id: 'reports', label: 'Reports', icon: '📋' },
    { id: 'settings', label: 'Settings', icon: '⚙️' },
  ];

  let sessions = $state<Session[]>([]);
  let mobileOpen = $state(false);
  let theme = $state(getTheme());

  async function loadSessions(): Promise<void> {
    try {
      sessions = await api.listSessions();
    } catch {
      sessions = [];
    }
  }

  function handleNavigate(page: PageId): void {
    onNavigate(page);
    mobileOpen = false;
  }

  function handleSelectSession(session: Session): void {
    onSelectSession(session);
    mobileOpen = false;
  }

  function handleNewSession(): void {
    onNewSession();
    mobileOpen = false;
  }

  function handleToggleTheme(): void {
    theme = toggleTheme();
  }

  async function handleDeleteSession(e: Event, sessionId: string): Promise<void> {
    e.stopPropagation();
    try {
      await api.deleteSession(sessionId);
      sessions = sessions.filter((s) => s.id !== sessionId);
    } catch { /* ignore */ }
  }

  // Load sessions on mount and refresh periodically
  loadSessions();
  const interval = setInterval(loadSessions, 10000);
  $effect(() => {
    return () => clearInterval(interval);
  });
</script>

<!-- Mobile hamburger button -->
<button
  class="lg:hidden fixed top-3 left-3 z-50 p-2 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border)] text-[var(--text-primary)]"
  onclick={() => mobileOpen = !mobileOpen}
  aria-label="Toggle menu"
>
  {mobileOpen ? '✕' : '☰'}
</button>

<!-- Mobile overlay -->
{#if mobileOpen}
  <div
    class="lg:hidden fixed inset-0 bg-black/50 z-40"
    role="presentation"
    onclick={() => mobileOpen = false}
    onkeydown={() => {}}
  ></div>
{/if}

<aside class="
  {mobileOpen ? 'translate-x-0' : '-translate-x-full'}
  lg:translate-x-0
  fixed lg:relative z-40
  w-64 lg:w-56 h-full
  bg-[var(--bg-secondary)] border-r border-[var(--border)]
  flex flex-col shrink-0
  transition-transform duration-200 ease-in-out
">
  <!-- Logo -->
  <div class="p-4 border-b border-[var(--border)] flex items-center gap-3">
    <span class="text-2xl">🛡️</span>
    <span class="text-lg font-bold text-[var(--accent)]">Dalang</span>
  </div>

  <!-- Navigation -->
  <nav class="p-2 space-y-1">
    {#each navItems as item}
      <button
        class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors text-left
          {currentPage === item.id
            ? 'bg-[var(--accent)]/10 text-[var(--accent)]'
            : 'text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]'}"
        onclick={() => handleNavigate(item.id)}
      >
        <span class="text-lg">{item.icon}</span>
        <span class="text-sm font-medium">{item.label}</span>
      </button>
    {/each}
  </nav>

  <!-- Session list -->
  <div class="flex-1 overflow-y-auto border-t border-[var(--border)]">
    <div class="flex items-center justify-between px-3 py-2">
      <span class="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wide">Sessions</span>
      <button
        class="text-xs px-2 py-1 rounded bg-[var(--accent)]/10 text-[var(--accent)] hover:bg-[var(--accent)]/20 transition-colors"
        onclick={handleNewSession}
        title="New Session"
      >+ New</button>
    </div>

    {#if sessions.length === 0}
      <div class="px-3 py-2 text-xs text-[var(--text-secondary)]">No sessions yet</div>
    {:else}
      <div class="px-2 space-y-0.5">
        {#each sessions as session}
          <!-- svelte: use div+role to avoid nested <button> -->
          <div
            class="w-full text-left px-2 py-2 rounded-lg transition-colors group cursor-pointer
              text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]"
            role="button"
            tabindex="0"
            onclick={() => handleSelectSession(session)}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleSelectSession(session); }}
          >
            <div class="flex items-center justify-between">
              <div class="truncate text-xs font-medium flex-1">{session.target}</div>
              <button
                class="opacity-0 group-hover:opacity-100 text-xs text-[var(--danger)] hover:text-[var(--danger)] ml-1 shrink-0"
                onclick={(e) => handleDeleteSession(e, session.id)}
                title="Delete session"
              >✕</button>
            </div>
            <div class="flex items-center gap-1.5 mt-0.5">
              <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-[var(--bg-tertiary)]">
                {session.mode === 'scan' ? '🤖' : '💬'}
              </span>
              <span class="text-[10px] opacity-60">
                {new Date(session.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
              </span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Footer: theme toggle + version -->
  <div class="p-3 border-t border-[var(--border)] flex items-center justify-between">
    <span class="text-xs text-[var(--text-secondary)]">v0.1.0</span>
    <button
      class="text-sm px-2 py-1 rounded-lg hover:bg-[var(--bg-tertiary)] transition-colors"
      onclick={handleToggleTheme}
      title="Toggle theme"
    >
      {theme === 'dark' ? '☀️' : '🌙'}
    </button>
  </div>
</aside>
