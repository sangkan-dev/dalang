<script lang="ts">
	import { onMount } from 'svelte';
	import { apiClient } from '$lib/api/client.js';
	import type { Settings, TestConnectionResult, UpdateSettingsRequest } from '$lib/api/types.js';
	import { toast } from '$lib/dashboard/toast.js';

	const PROVIDER_MODELS: Record<string, string[]> = {
		gemini: ['gemini-2.5-flash', 'gemini-2.5-pro', 'gemini-2.5-flash-lite', 'gemini-3-pro-preview'],
		openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4.1', 'gpt-4.1-mini'],
		anthropic: ['claude-sonnet-4-20250514', 'claude-3-5-haiku-20241022'],
		copilot: ['claude-sonnet-4.6', 'claude-opus-4.6', 'gpt-5.2', 'gpt-4.1', 'gemini-3-pro-preview'],
		ollama: ['llama3.1:latest', 'qwen2.5:latest', 'mistral:latest'],
		custom: []
	};

	let loading = $state(true);
	let loadError = $state('');
	let saving = $state(false);
	let testing = $state(false);
	let testResult = $state<TestConnectionResult | null>(null);
	let apiKeyInput = $state('');
	let showApiKey = $state(false);
	let useCustomModel = $state(false);

	let settings = $state<Settings>({
		provider: 'gemini',
		model: 'gemini-2.5-flash',
		auth_method: 'apikey',
		endpoint_mode: 'openai_compat',
		auth_status: 'not_authenticated',
		verbose: false,
		has_api_key: false,
		custom_base_url: ''
	});

	const availableModels = $derived(PROVIDER_MODELS[settings.provider] ?? []);

	async function loadSettings(): Promise<void> {
		loading = true;
		loadError = '';
		try {
			settings = await apiClient.getSettings();
			const models = PROVIDER_MODELS[settings.provider] ?? [];
			useCustomModel = models.length > 0 && !models.includes(settings.model);
		} catch (err) {
			const message = err instanceof Error ? err.message : 'unknown error';
			loadError = message;
			toast.error(`Failed to load settings: ${message}`);
		} finally {
			loading = false;
		}
	}

	function onProviderChange(): void {
		const models = PROVIDER_MODELS[settings.provider] ?? [];
		if (models.length > 0) {
			settings.model = models[0];
			useCustomModel = false;
		}
	}

	function switchToPresets(): void {
		if (availableModels.length === 0) return;
		useCustomModel = false;
		settings.model = availableModels[0];
	}

	function switchToCustomModel(): void {
		useCustomModel = true;
	}

	function toggleApiKeyVisibility(): void {
		showApiKey = !showApiKey;
	}

	async function saveSettings(): Promise<void> {
		saving = true;
		try {
			const payload: UpdateSettingsRequest = {
				provider: settings.provider,
				model: settings.model,
				endpoint_mode: settings.endpoint_mode,
				verbose: settings.verbose,
				custom_base_url: settings.custom_base_url
			};

			if (apiKeyInput.trim()) {
				payload.api_key = apiKeyInput.trim();
			}

			await apiClient.updateSettings(payload);
			apiKeyInput = '';
			showApiKey = false;
			toast.success('Settings updated');
			await loadSettings();
		} catch (err) {
			toast.error(
				`Failed to save settings: ${err instanceof Error ? err.message : 'unknown error'}`
			);
		} finally {
			saving = false;
		}
	}

	async function testConnection(): Promise<void> {
		testing = true;
		testResult = null;
		try {
			testResult = await apiClient.testConnection();
			if (testResult.success) {
				toast.success(`Connected in ${testResult.latency_ms}ms`);
			} else {
				toast.warning(testResult.message);
			}
		} catch (err) {
			toast.error(
				`Connection test failed: ${err instanceof Error ? err.message : 'unknown error'}`
			);
		} finally {
			testing = false;
		}
	}

	onMount(loadSettings);
</script>

<section class="space-y-4">
	<header class="surface-panel dashboard-warboard space-y-2 p-4">
		<p class="text-xs tracking-[0.2em] text-[color:var(--color-ash)] uppercase">
			Dashboard / Settings
		</p>
		<h2 class="text-xl font-semibold text-[color:var(--color-text)]">Provider Configuration</h2>
		<p class="text-xs text-[color:var(--color-ash)]">
			Model routing, credentials, and endpoint posture control.
		</p>
	</header>

	{#if loading}
		<div class="surface-panel p-4 text-sm text-[color:var(--color-ash)]">Loading settings...</div>
	{:else if loadError}
		<div
			class="surface-panel space-y-3 border border-[color:var(--color-rust)]/40 bg-[color:var(--color-rust)]/10 p-4 text-sm text-[color:var(--color-rust)]"
		>
			<p>Could not load settings from the server: {loadError}</p>
			<button
				type="button"
				class="rounded-lg border border-[color:var(--color-border)] px-3 py-1.5 text-xs text-[color:var(--color-text)]"
				onclick={() => loadSettings()}
			>
				Retry
			</button>
		</div>
	{:else}
		<div class="space-y-4">
			<div class="surface-panel dashboard-warboard space-y-3 p-4">
				<p
					class="text-sm {settings.auth_status === 'authenticated'
						? 'text-emerald-300'
						: 'text-[color:var(--color-ash)]'}"
				>
					Auth status: {settings.auth_status}
				</p>
				<p class="text-xs text-[color:var(--color-ash)]">Method: {settings.auth_method}</p>
			</div>

			<div class="surface-panel space-y-4 p-4">
				<div class="dashboard-inline-pills">
					<span class="dashboard-pill">Provider Matrix</span>
					<span class="dashboard-pill">Auth Gate</span>
					<span class="dashboard-pill">Endpoint Policy</span>
				</div>
				<div>
					<label class="mb-1 block text-sm text-[color:var(--color-ash)]" for="provider"
						>Provider</label
					>
					<select
						id="provider"
						bind:value={settings.provider}
						onchange={onProviderChange}
						class="w-full rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-base-text)]"
					>
						<option value="gemini">Gemini</option>
						<option value="openai">OpenAI</option>
						<option value="anthropic">Anthropic</option>
						<option value="copilot">GitHub Copilot</option>
						<option value="ollama">Ollama</option>
						<option value="custom">Custom</option>
					</select>
				</div>

				<div>
					<label class="mb-1 block text-sm text-[color:var(--color-ash)]" for="model">Model</label>
					{#if useCustomModel}
						<div class="flex gap-2">
							<input
								id="model"
								bind:value={settings.model}
								class="w-full rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-base-text)]"
							/>
							{#if availableModels.length > 0}
								<button class="control-chip" onclick={switchToPresets}> Presets </button>
							{/if}
						</div>
					{:else}
						<div class="flex gap-2">
							<select
								id="model"
								bind:value={settings.model}
								class="w-full rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-base-text)]"
							>
								{#each availableModels as model (model)}
									<option value={model}>{model}</option>
								{/each}
							</select>
							<button class="control-chip" onclick={switchToCustomModel}> Custom </button>
						</div>
					{/if}
				</div>

				<div>
					<label class="mb-1 block text-sm text-[color:var(--color-ash)]" for="endpoint"
						>Endpoint mode</label
					>
					<select
						id="endpoint"
						bind:value={settings.endpoint_mode}
						class="w-full rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-base-text)]"
					>
						<option value="openai_compat">OpenAI Compatible</option>
						<option value="google_rest">Google REST</option>
					</select>
				</div>

				<div>
					<label class="mb-1 block text-sm text-[color:var(--color-ash)]" for="base-url"
						>Custom base URL</label
					>
					<input
						id="base-url"
						bind:value={settings.custom_base_url}
						placeholder="https://api.example.com/v1"
						class="w-full rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-base-text)]"
					/>
				</div>
			</div>

			<div class="surface-panel space-y-4 p-4">
				<div>
					<label class="mb-1 block text-sm text-[color:var(--color-ash)]" for="api-key"
						>API key</label
					>
					<div class="flex gap-2">
						<input
							id="api-key"
							type={showApiKey ? 'text' : 'password'}
							bind:value={apiKeyInput}
							placeholder={settings.has_api_key ? '•••••••• key already saved' : 'sk-...'}
							class="w-full rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-base-text)]"
						/>
						<button class="control-chip" onclick={toggleApiKeyVisibility}>
							{showApiKey ? 'Hide' : 'Show'}
						</button>
					</div>
				</div>

				<label
					class="flex items-center gap-2 text-sm text-[color:var(--color-base-text)]"
					for="verbose"
				>
					<input id="verbose" type="checkbox" bind:checked={settings.verbose} />
					Verbose mode
				</label>

				<div class="flex flex-wrap gap-2">
					<button class="control-chip" onclick={testConnection} disabled={testing}>
						{testing ? 'Testing...' : 'Test Connection'}
					</button>
					{#if testResult}
						<p
							class="text-xs {testResult.success
								? 'text-emerald-300'
								: 'text-[color:var(--color-rust)]'}"
						>
							{testResult.message}
						</p>
					{/if}
				</div>
			</div>

			<button class="control-cta" onclick={saveSettings} disabled={saving}>
				{saving ? 'Saving...' : 'Save Settings'}
			</button>
		</div>
	{/if}
</section>
