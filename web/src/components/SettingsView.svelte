<script lang="ts">
  import { api } from '../lib/api.ts';
  import type { Settings } from '../lib/types';

  interface StatusMessage {
    type: 'success' | 'error';
    text: string;
  }

  let settings = $state<Settings>({
    provider: '',
    model: '',
    auth_method: '',
    endpoint_mode: '',
  });
  let loading = $state(true);
  let saving = $state(false);
  let message = $state<StatusMessage | null>(null);

  async function loadSettings(): Promise<void> {
    loading = true;
    try {
      settings = await api.getSettings();
    } catch (e) {
      message = { type: 'error', text: (e as Error).message };
    } finally {
      loading = false;
    }
  }

  async function saveSettings(): Promise<void> {
    saving = true;
    try {
      await api.updateSettings(settings);
      message = { type: 'success', text: 'Settings saved successfully' };
      setTimeout(() => message = null, 3000);
    } catch (e) {
      message = { type: 'error', text: (e as Error).message };
    } finally {
      saving = false;
    }
  }

  loadSettings();
</script>

<div class="flex-1 overflow-y-auto p-6">
  <div class="max-w-2xl mx-auto">
    <h1 class="text-2xl font-bold mb-6">⚙️ Settings</h1>

    {#if loading}
      <p class="text-[var(--text-secondary)]">Loading settings...</p>
    {:else}
      <div class="space-y-6">
        <div class="bg-[var(--bg-secondary)] rounded-xl p-6 border border-[var(--border)] space-y-4">
          <h2 class="text-lg font-semibold">LLM Provider</h2>

          <div>
            <label class="block text-sm font-medium mb-1.5 text-[var(--text-secondary)]" for="provider">Provider</label>
            <select
              id="provider"
              bind:value={settings.provider}
              class="w-full px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
                text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
            >
              <option value="gemini">Google Gemini</option>
              <option value="openai">OpenAI</option>
              <option value="anthropic">Anthropic</option>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium mb-1.5 text-[var(--text-secondary)]" for="model">Model</label>
            <input
              id="model"
              type="text"
              bind:value={settings.model}
              class="w-full px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
                text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
            />
          </div>

          <div>
            <label class="block text-sm font-medium mb-1.5 text-[var(--text-secondary)]" for="auth">Auth Method</label>
            <select
              id="auth"
              bind:value={settings.auth_method}
              class="w-full px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
                text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
            >
              <option value="apikey">API Key</option>
              <option value="oauth">OAuth / CloudCode</option>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium mb-1.5 text-[var(--text-secondary)]" for="endpoint">Endpoint Mode</label>
            <select
              id="endpoint"
              bind:value={settings.endpoint_mode}
              class="w-full px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
                text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
            >
              <option value="openai_compat">OpenAI Compatible</option>
              <option value="google_rest">Google REST</option>
            </select>
          </div>
        </div>

        {#if message}
          <div class="px-4 py-3 rounded-lg text-sm
            {message.type === 'success'
              ? 'bg-emerald-950/30 border border-emerald-800/30 text-[var(--success)]'
              : 'bg-red-950/30 border border-red-800/30 text-[var(--danger)]'}">
            {message.text}
          </div>
        {/if}

        <button
          class="px-6 py-3 bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-[var(--bg-primary)]
            font-semibold rounded-lg transition-colors disabled:opacity-50"
          disabled={saving}
          onclick={saveSettings}
        >
          {saving ? 'Saving...' : 'Save Settings'}
        </button>
      </div>
    {/if}
  </div>
</div>
