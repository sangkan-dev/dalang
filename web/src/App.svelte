<script lang="ts">
  import Sidebar from './components/Sidebar.svelte';
  import ChatView from './components/ChatView.svelte';
  import SkillsView from './components/SkillsView.svelte';
  import ReportsView from './components/ReportsView.svelte';
  import SettingsView from './components/SettingsView.svelte';
  import Toast from './components/Toast.svelte';
  import CommandPalette from './components/CommandPalette.svelte';
  import type { PageId, Session } from './lib/types';

  let currentPage = $state<PageId>('chat');
  let currentSessionId = $state<string | null>(null);
  let chatViewResetTrigger = $state(0);
  let showPalette = $state(false);

  function handleSelectSession(session: Session): void {
    currentPage = 'chat';
    currentSessionId = session.id;
  }

  function handleNewSession(): void {
    currentPage = 'chat';
    currentSessionId = null;
    chatViewResetTrigger++;
  }

  function handleKeydown(e: KeyboardEvent): void {
    // Ctrl+K / Cmd+K → command palette
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault();
      showPalette = !showPalette;
    }
    // Ctrl+N / Cmd+N → new session
    if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
      e.preventDefault();
      handleNewSession();
    }
    // Escape → close palette
    if (e.key === 'Escape') {
      showPalette = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<Toast />
{#if showPalette}
  <CommandPalette
    {currentPage}
    onNavigate={(page) => { currentPage = page; showPalette = false; }}
    onClose={() => showPalette = false}
  />
{/if}

<div class="flex h-screen overflow-hidden">
  <Sidebar
    {currentPage}
    onNavigate={(page) => currentPage = page}
    onSelectSession={handleSelectSession}
    onNewSession={handleNewSession}
  />

  <main class="flex-1 flex flex-col min-w-0 lg:ml-0 ml-0">
    {#if currentPage === 'chat'}
      <ChatView bind:sessionId={currentSessionId} resetTrigger={chatViewResetTrigger} />
    {:else if currentPage === 'skills'}
      <SkillsView />
    {:else if currentPage === 'reports'}
      <ReportsView />
    {:else if currentPage === 'settings'}
      <SettingsView />
    {/if}
  </main>
</div>
