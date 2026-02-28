<script lang="ts">
  import { api } from '../lib/api.ts';
  import { toast } from '../lib/toast.ts';
  import type { Settings } from '../lib/types';

  let settings = $state<Settings>({
    provider: '',
    model: '',
    auth_method: '',
    endpoint_mode: '',
    auth_status: 'not_authenticated',
  });
  let loading = $state(true);
  let saving = $state(false);

  async function loadSettings(): Promise<void> {
    loading = true;
    try {
      settings = await api.getSettings();
    } catch (e) {
      toast.error((e as Error).message);
    } finally {
      loading = false;
    }
  }

  async function saveSettings(): Promise<void> {
    saving = true;
    try {
      await api.updateSettings(settings);
      toast.success('Settings saved successfully');
    } catch (e) {
      toast.error((e as Error).message);
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
        <!-- Auth status banner -->
        <div class="flex items-center gap-3 px-4 py-3 rounded-lg border
          {settings.auth_status === 'authenticated'
            ? 'bg-emerald-950/30 border-emerald-800/30'
            : settings.auth_status === 'env_var'
              ? 'bg-amber-950/30 border-amber-800/30'
              : 'bg-red-950/30 border-red-800/30'}">
          <span class="text-lg">
            {settings.auth_status === 'authenticated' ? '🔐' : settings.auth_status === 'env_var' ? '🔑' : '⚠️'}
          </span>
          <div>
            <div class="text-sm font-medium">
              {settings.auth_status === 'authenticated' ? 'Authenticated via OAuth'
                : settings.auth_status === 'env_var' ? 'Using API Key (env var)'
                : 'Not Authenticated'}
            </div>
            <div class="text-xs text-[var(--text-secondary)]">
              {settings.auth_status === 'not_authenticated' ? 'Run `dalang login` in terminal to authenticate.' : `Method: ${settings.auth_method}`}
            </div>
          </div>
        </div>

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
            <label class="block text-sm font-medium mb-1.5 text-[var(--text-secondary)]" for="auth">Auth Method <span class="text-xs opacity-60">(read-only)</span></label>
            <select
              id="auth"
              bind:value={settings.auth_method}
              disabled
              class="w-full px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
                text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] opacity-60 cursor-not-allowed"
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
