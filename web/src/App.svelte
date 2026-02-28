<script lang="ts">
  import Sidebar from './components/Sidebar.svelte';
  import ChatView from './components/ChatView.svelte';
  import SkillsView from './components/SkillsView.svelte';
  import ReportsView from './components/ReportsView.svelte';
  import SettingsView from './components/SettingsView.svelte';
  import type { PageId } from './lib/types';

  let currentPage = $state<PageId>('chat');
  let currentSessionId = $state<string | null>(null);
</script>

<div class="flex h-screen overflow-hidden">
  <Sidebar
    {currentPage}
    onNavigate={(page) => currentPage = page}
  />

  <main class="flex-1 flex flex-col min-w-0">
    {#if currentPage === 'chat'}
      <ChatView bind:sessionId={currentSessionId} />
    {:else if currentPage === 'skills'}
      <SkillsView />
    {:else if currentPage === 'reports'}
      <ReportsView />
    {:else if currentPage === 'settings'}
      <SettingsView />
    {/if}
  </main>
</div>
