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
	let authActionBusy = $state(false);
	let authHint = $state<string | null>(null);
	let copilotDevice = $state<{
		verification_uri: string;
		user_code: string;
		expires_in: number;
		device_code: string;
		interval: number;
		started_at_ms: number;
	} | null>(null);

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
			toast.error(`Gagal memuat pengaturan: ${message}`);
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

		// Defaults that match real provider behavior.
		if (settings.provider === 'copilot') {
			// Copilot CAPI is required for some models (eg gpt-5.3-codex).
			settings.endpoint_mode =
				settings.model === 'gpt-5.3-codex' ? 'copilot_capi' : 'copilot';
		} else if (settings.provider === 'gemini') {
			// Gemini CLI/gcloud session works through CloudCode endpoint mode.
			settings.endpoint_mode = 'cloudcode';
		} else if (settings.endpoint_mode === 'copilot' || settings.endpoint_mode === 'copilot_capi') {
			settings.endpoint_mode = 'openai_compat';
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

	$effect(() => {
		// Keep endpoint mode aligned when model/provider changes.
		if (settings.provider === 'copilot') {
			settings.endpoint_mode = settings.model === 'gpt-5.3-codex' ? 'copilot_capi' : 'copilot';
		}
	});

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
			toast.success('Pengaturan disimpan');
			await loadSettings();
		} catch (err) {
			toast.error(
				`Gagal menyimpan pengaturan: ${err instanceof Error ? err.message : 'kesalahan tidak diketahui'}`
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
				toast.success(`Terhubung dalam ${testResult.latency_ms}ms`);
			} else {
				toast.warning(testResult.message);
			}
		} catch (err) {
			toast.error(
				`Tes koneksi gagal: ${err instanceof Error ? err.message : 'kesalahan tidak diketahui'}`
			);
		} finally {
			testing = false;
		}
	}

	function authStatusLabel(): string {
		if (settings.auth_status === 'authenticated') return 'Sudah login';
		if (settings.auth_status === 'env_var') return 'Token dari variabel lingkungan';
		return 'Belum login';
	}

	function needsOAuth(provider: string): boolean {
		return provider === 'copilot' || provider === 'gemini';
	}

	async function startAuthFlow(): Promise<void> {
		authHint = null;
		copilotDevice = null;
		authActionBusy = true;
		try {
			const resp = await fetch('/api/settings/auth/start', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ provider: settings.provider })
			});
			if (!resp.ok) {
				const text = await resp.text();
				throw new Error(`${resp.status}: ${text}`);
			}
			const data = (await resp.json()) as
				| {
						kind: 'copilot_device';
						verification_uri: string;
						user_code: string;
						expires_in: number;
						device_code: string;
						interval: number;
				  }
				| { kind: 'cli_extract'; message: string };

			if (data.kind === 'copilot_device') {
				copilotDevice = {
					verification_uri: data.verification_uri,
					user_code: data.user_code,
					expires_in: data.expires_in,
					device_code: data.device_code,
					interval: data.interval,
					started_at_ms: Date.now()
				};
				authHint = 'Buka tautan verifikasi, masukkan kode, lalu kembali ke sini untuk menyelesaikan login.';
			} else {
				authHint = data.message;
				await loadSettings();
			}
		} catch (err) {
			toast.error(
				`Gagal memulai login: ${err instanceof Error ? err.message : 'kesalahan tidak diketahui'}`
			);
		} finally {
			authActionBusy = false;
		}
	}

	async function refreshAuthStatus(): Promise<void> {
		authHint = null;
		if (settings.provider === 'copilot' && copilotDevice) {
			authActionBusy = true;
			try {
				const resp = await fetch('/api/settings/auth/copilot/poll', {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ device_code: copilotDevice.device_code })
				});
				if (!resp.ok) {
					const text = await resp.text();
					throw new Error(`${resp.status}: ${text}`);
				}
				const data = (await resp.json()) as
					| { kind: 'pending'; message: string }
					| { kind: 'authenticated'; message: string };
				authHint = data.message;
				await loadSettings();
				if (data.kind === 'authenticated') {
					copilotDevice = null;
				}
			} catch (err) {
				toast.error(
					`Gagal memeriksa status login: ${err instanceof Error ? err.message : 'kesalahan tidak diketahui'}`
				);
			} finally {
				authActionBusy = false;
			}
			return;
		}

		await loadSettings();
	}

	onMount(loadSettings);
</script>

<section class="space-y-4">
	<header class="surface-panel dashboard-warboard space-y-2 p-4">
		<p class="text-xs tracking-[0.2em] text-[color:var(--color-ash)] uppercase">Dasbor / Pengaturan</p>
		<h2 class="text-xl font-semibold text-[color:var(--color-text)]">Penyedia AI & akses</h2>
		<p class="max-w-3xl text-xs leading-relaxed text-[color:var(--color-ash)]">
			Pilih penyedia AI dan model, lalu atur cara login. Untuk beberapa penyedia (mis. Copilot/Gemini),
			login dilakukan via OAuth/CLI, bukan API key.
		</p>
	</header>

	{#if loading}
		<div class="surface-panel p-4 text-sm text-[color:var(--color-ash)]">Memuat pengaturan…</div>
	{:else if loadError}
		<div
			class="surface-panel space-y-3 border border-[color:var(--color-rust)]/40 bg-[color:var(--color-rust)]/10 p-4 text-sm text-[color:var(--color-rust)]"
		>
			<p>Tidak bisa memuat pengaturan dari server: {loadError}</p>
			<button
				type="button"
				class="rounded-lg border border-[color:var(--color-border)] px-3 py-1.5 text-xs text-[color:var(--color-text)]"
				onclick={() => loadSettings()}
			>
				Coba lagi
			</button>
		</div>
	{:else}
		<div class="space-y-4">
			<div class="surface-panel dashboard-warboard space-y-2 p-4">
				<p class="text-sm {settings.auth_status === 'authenticated' ? 'text-emerald-300' : 'text-[color:var(--color-ash)]'}">
					Status login: {authStatusLabel()}
				</p>
				<p class="text-xs text-[color:var(--color-ash)]">
					Metode: <span class="font-mono">{settings.auth_method}</span>
				</p>
			</div>

			<div class="grid gap-3 lg:grid-cols-2">
				<div class="surface-panel space-y-4 p-4">
					<p class="text-[10px] tracking-[0.16em] text-[color:var(--color-ash)] uppercase">
						1) Penyedia & model
					</p>

					<div>
						<label class="mb-1 block text-sm text-[color:var(--color-ash)]" for="provider"
							>Penyedia</label
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
									<button class="control-chip" onclick={switchToPresets}>Preset</button>
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
								<button class="control-chip" onclick={switchToCustomModel}>Kustom</button>
							</div>
						{/if}
						<p class="mt-1 text-xs text-[color:var(--color-ash)]">
							Jika model tidak ada di daftar, pilih “Kustom” lalu isi manual.
						</p>
					</div>

					<div>
						<label class="mb-1 block text-sm text-[color:var(--color-ash)]" for="endpoint"
							>Mode endpoint</label
						>
						<select
							id="endpoint"
							bind:value={settings.endpoint_mode}
							class="w-full rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-base-text)]"
						>
							<option value="openai_compat">Kompatibel OpenAI</option>
							<option value="google_rest">Google REST</option>
						</select>
					</div>

					<div>
						<label class="mb-1 block text-sm text-[color:var(--color-ash)]" for="base-url"
							>Base URL kustom (opsional)</label
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
					<p class="text-[10px] tracking-[0.16em] text-[color:var(--color-ash)] uppercase">
						2) Autentikasi (login)
					</p>

					{#if needsOAuth(settings.provider)}
						<div class="rounded-lg border border-[color:var(--color-border)] bg-white/5 p-3">
							<p class="text-sm text-[color:var(--color-base-text)]">
								Penyedia ini biasanya butuh login OAuth/CLI.
							</p>
							<p class="mt-1 text-xs text-[color:var(--color-ash)]">
								Kamu bisa klik tombol di bawah (kalau didukung), atau login manual lewat terminal.
							</p>
							<div class="mt-3 flex flex-wrap gap-2">
								<button class="control-chip" onclick={startAuthFlow} disabled={authActionBusy}>
									{authActionBusy ? 'Memulai…' : settings.provider === 'copilot' ? 'Mulai login Copilot' : 'Deteksi sesi CLI'}
								</button>
								<button class="control-chip" onclick={refreshAuthStatus} disabled={authActionBusy}>
									{settings.provider === 'copilot' && copilotDevice ? 'Selesaikan login' : 'Muat ulang status'}
								</button>
							</div>

							{#if copilotDevice}
								<div class="mt-3 space-y-1 text-xs text-[color:var(--color-ash)]">
									<p>
										1) Buka: <a class="text-(--color-gold) hover:underline" href={copilotDevice.verification_uri} target="_blank" rel="noreferrer">{copilotDevice.verification_uri}</a>
									</p>
									<p>
										2) Masukkan kode: <span class="font-mono text-[color:var(--color-base-text)]">{copilotDevice.user_code}</span>
									</p>
									<p>3) Kembali ke sini, lalu klik “Muat ulang status”.</p>
								</div>
							{/if}

							{#if authHint}
								<p class="mt-3 text-xs text-[color:var(--color-ash)]">{authHint}</p>
							{/if}

							<div class="mt-3 rounded-lg border border-[color:var(--color-border)] p-3">
								<p class="text-xs font-semibold text-[color:var(--color-base-text)]">Login manual</p>
								<pre class="dashboard-raw mt-2 text-xs" dir="ltr">dalang login --provider {settings.provider}</pre>
								<p class="mt-2 text-xs text-[color:var(--color-ash)]">
									Setelah login di terminal, kembali ke halaman ini dan klik “Muat ulang status”.
								</p>
							</div>
						</div>
					{/if}

					<div>
						<label class="mb-1 block text-sm text-[color:var(--color-ash)]" for="api-key"
							>API key (opsional)</label
						>
						<div class="flex gap-2">
							<input
								id="api-key"
								type={showApiKey ? 'text' : 'password'}
								bind:value={apiKeyInput}
								placeholder={settings.has_api_key ? '•••••••• key sudah tersimpan' : 'sk-...'}
								class="w-full rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-base-text)]"
							/>
							<button class="control-chip" onclick={toggleApiKeyVisibility}>
								{showApiKey ? 'Sembunyikan' : 'Tampilkan'}
							</button>
						</div>
						<p class="mt-1 text-xs text-[color:var(--color-ash)]">
							Isi hanya jika penyedia kamu memang memakai API key. Untuk Copilot/Gemini biasanya tidak.
						</p>
					</div>

					<label class="flex items-center gap-2 text-sm text-[color:var(--color-base-text)]" for="verbose">
						<input id="verbose" type="checkbox" bind:checked={settings.verbose} />
						Mode verbose (lebih detail)
					</label>
				</div>
			</div>

			<div class="surface-panel space-y-3 p-4">
				<p class="text-[10px] tracking-[0.16em] text-[color:var(--color-ash)] uppercase">
					3) Tes koneksi
				</p>
				<div class="flex flex-wrap items-center gap-2">
					<button class="control-chip" onclick={testConnection} disabled={testing}>
						{testing ? 'Menguji…' : 'Tes koneksi'}
					</button>
					{#if testResult}
						<p
							class="text-xs {testResult.success ? 'text-emerald-300' : 'text-[color:var(--color-rust)]'}"
						>
							{testResult.message}
						</p>
					{/if}
				</div>
			</div>

			<button class="control-cta" onclick={saveSettings} disabled={saving}>
				{saving ? 'Menyimpan…' : 'Simpan pengaturan'}
			</button>
		</div>
	{/if}
</section>
