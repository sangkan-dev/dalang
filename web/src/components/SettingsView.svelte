<script lang="ts">
  import { api } from '../lib/api.ts';
  import { toast } from '../lib/toast.ts';
  import type { Settings, TestConnectionResult } from '../lib/types';
  import { PROVIDER_MODELS } from '../lib/types';

  let settings = $state<Settings>({
    provider: '',
    model: '',
    auth_method: '',
    endpoint_mode: '',
    auth_status: 'not_authenticated',
  });
  let loading = $state(true);
  let saving = $state(false);
  let testing = $state(false);
  let testResult = $state<TestConnectionResult | null>(null);
  let showApiKey = $state(false);
  let apiKeyInput = $state('');
  let useCustomModel = $state(false);

  // Derive available models from the selected provider
  let availableModels = $derived(PROVIDER_MODELS[settings.provider] ?? []);

  async function loadSettings(): Promise<void> {
    loading = true;
    try {
      settings = await api.getSettings();
      // Check if current model is custom (not in the provider's known list)
      const models = PROVIDER_MODELS[settings.provider] ?? [];
      useCustomModel = models.length > 0 && !models.includes(settings.model);
    } catch (e) {
      toast.error((e as Error).message);
    } finally {
      loading = false;
    }
  }

  async function saveSettings(): Promise<void> {
    saving = true;
    try {
      const payload: Settings = { ...settings };
      if (apiKeyInput) {
        payload.api_key = apiKeyInput;
      }
      await api.updateSettings(payload);
      apiKeyInput = '';
      showApiKey = false;
      toast.success('Settings saved successfully');
      // Reload to reflect new has_api_key etc.
      await loadSettings();
    } catch (e) {
      toast.error((e as Error).message);
    } finally {
      saving = false;
    }
  }

  async function testConnection(): Promise<void> {
    testing = true;
    testResult = null;
    try {
      testResult = await api.testConnection();
      if (testResult.success) {
        toast.success(`Connected (${testResult.latency_ms}ms)`);
      } else {
        toast.error(testResult.message);
      }
    } catch (e) {
      toast.error((e as Error).message);
    } finally {
      testing = false;
    }
  }

  function onProviderChange(): void {
    // Reset model to first known model when provider changes
    const models = PROVIDER_MODELS[settings.provider] ?? [];
    if (models.length > 0) {
      settings.model = models[0];
      useCustomModel = false;
    }
  }

  loadSettings();
</script>

<div class="flex-1 overflow-y-auto p-6">
  <div class="max-w-2xl mx-auto">
    <h1 class="text-2xl font-bold mb-6">Settings</h1>

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
              : (settings as any).has_api_key
                ? 'bg-blue-950/30 border-blue-800/30'
                : 'bg-red-950/30 border-red-800/30'}">
          <span class="text-lg">
            {settings.auth_status === 'authenticated' ? '🔐'
              : settings.auth_status === 'env_var' ? '🔑'
              : (settings as any).has_api_key ? '🔑'
              : '⚠️'}
          </span>
          <div>
            <div class="text-sm font-medium">
              {settings.auth_status === 'authenticated' ? 'Authenticated via OAuth'
                : settings.auth_status === 'env_var' ? 'Using API Key (env var)'
                : (settings as any).has_api_key ? 'API Key configured'
                : 'Not Authenticated'}
            </div>
            <div class="text-xs text-[var(--text-secondary)]">
              {settings.auth_status === 'not_authenticated' && !(settings as any).has_api_key
                ? 'Enter an API key below or run `dalang login` in terminal.'
                : `Method: ${settings.auth_method}`}
            </div>
          </div>
        </div>

        <!-- LLM Provider section -->
        <div class="bg-[var(--bg-secondary)] rounded-xl p-6 border border-[var(--border)] space-y-4">
          <h2 class="text-lg font-semibold">LLM Provider</h2>

          <div>
            <label class="block text-sm font-medium mb-1.5 text-[var(--text-secondary)]" for="provider">Provider</label>
            <select
              id="provider"
              bind:value={settings.provider}
              onchange={onProviderChange}
              class="w-full px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
                text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
            >
              <option value="gemini">Google Gemini</option>
              <option value="openai">OpenAI</option>
              <option value="anthropic">Anthropic</option>
              <option value="ollama">Ollama (local)</option>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium mb-1.5 text-[var(--text-secondary)]" for="model">Model</label>
            {#if useCustomModel}
              <div class="flex gap-2">
                <input
                  id="model"
                  type="text"
                  bind:value={settings.model}
                  placeholder="Enter model name..."
                  class="flex-1 px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
                    text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
                />
                {#if availableModels.length > 0}
                  <button
                    type="button"
                    class="px-3 py-2 text-xs border border-[var(--border)] rounded-lg hover:bg-[var(--bg-primary)] text-[var(--text-secondary)]"
                    onclick={() => { useCustomModel = false; settings.model = availableModels[0]; }}
                  >Presets</button>
                {/if}
              </div>
            {:else}
              <div class="flex gap-2">
                <select
                  id="model"
                  bind:value={settings.model}
                  class="flex-1 px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
                    text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
                >
                  {#each availableModels as m}
                    <option value={m}>{m}</option>
                  {/each}
                </select>
                <button
                  type="button"
                  class="px-3 py-2 text-xs border border-[var(--border)] rounded-lg hover:bg-[var(--bg-primary)] text-[var(--text-secondary)]"
                  onclick={() => { useCustomModel = true; }}
                >Custom</button>
              </div>
            {/if}
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

        <!-- API Key section -->
        <div class="bg-[var(--bg-secondary)] rounded-xl p-6 border border-[var(--border)] space-y-4">
          <h2 class="text-lg font-semibold">API Key</h2>
          <div>
            <label class="block text-sm font-medium mb-1.5 text-[var(--text-secondary)]" for="apikey">
              {(settings as any).has_api_key ? 'Update API Key' : 'Enter API Key'}
            </label>
            <div class="flex gap-2">
              <input
                id="apikey"
                type={showApiKey ? 'text' : 'password'}
                bind:value={apiKeyInput}
                placeholder={(settings as any).has_api_key ? '••••••••  (key already saved)' : 'sk-...'}
                class="flex-1 px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
                  text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] font-mono text-sm"
              />
              <button
                type="button"
                class="px-3 py-2 text-sm border border-[var(--border)] rounded-lg hover:bg-[var(--bg-primary)] text-[var(--text-secondary)]"
                onclick={() => showApiKey = !showApiKey}
              >{showApiKey ? '🙈' : '👁️'}</button>
            </div>
            {#if (settings as any).has_api_key}
              <p class="text-xs text-[var(--text-secondary)] mt-1">A key is already saved. Enter a new value to replace it.</p>
            {/if}
          </div>

          <!-- Test connection -->
          <div class="flex items-center gap-3">
            <button
              type="button"
              class="px-4 py-2 text-sm border border-[var(--border)] rounded-lg hover:bg-[var(--bg-primary)]
                text-[var(--text-secondary)] disabled:opacity-50 transition-colors"
              disabled={testing}
              onclick={testConnection}
            >
              {#if testing}
                <span class="inline-flex items-center gap-1.5">
                  <span class="inline-block w-3.5 h-3.5 border-2 border-current border-t-transparent rounded-full animate-spin"></span>
                  Testing...
                </span>
              {:else}
                Test Connection
              {/if}
            </button>
            {#if testResult}
              <span class="text-sm {testResult.success ? 'text-emerald-400' : 'text-red-400'}">
                {testResult.success ? `OK (${testResult.latency_ms}ms)` : 'Failed'}
              </span>
            {/if}
          </div>
        </div>

        <!-- Preferences section -->
        <div class="bg-[var(--bg-secondary)] rounded-xl p-6 border border-[var(--border)] space-y-4">
          <h2 class="text-lg font-semibold">Preferences</h2>

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

          <label class="flex items-center gap-3 cursor-pointer" for="verbose">
            <input
              id="verbose"
              type="checkbox"
              bind:checked={settings.verbose}
              class="w-5 h-5 rounded border-[var(--border)] bg-[var(--bg-primary)] accent-[var(--accent)]"
            />
            <div>
              <span class="text-sm font-medium">Verbose mode</span>
              <p class="text-xs text-[var(--text-secondary)]">Show extra debug info in engine output</p>
            </div>
          </label>
        </div>

        <!-- Save button -->
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
