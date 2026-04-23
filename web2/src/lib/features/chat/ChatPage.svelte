<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { apiClient } from '$lib/api/client.js';
	import { chatRoleLabel, replayEvents } from '$lib/api/events.js';
	import { renderMarkdown, renderMarkdownRaw } from '$lib/markdown.js';
	import { createDalangWebSocket } from '$lib/api/websocket.js';
	import type { ChatMessage, DalangWebSocket, EngineEvent, SessionMode } from '$lib/api/types.js';
	import { toast } from '$lib/dashboard/toast.js';
	import { onMount } from 'svelte';

	let mode = $state<SessionMode>('interactive');
	let target = $state('');
	let inputText = $state('');
	let messages = $state<ChatMessage[]>([]);
	let sessionId = $state<string | null>(null);
	let isThinking = $state(false);
	let isConnected = $state(false);
	let isReconnecting = $state(false);
	let maxIter = $state(15);
	let cmdTimeout = $state(300);
	let showSetup = $state(true);
	let ws: DalangWebSocket | null = null;
	let wsSessionId: string | null = null;
	let chatContainer = $state<HTMLDivElement | null>(null);
	let loadedSessionId = $state<string | null>(null);
	let messageView = $state<'formatted' | 'raw'>('formatted');

	function scrollToBottom(): void {
		if (!chatContainer) return;
		requestAnimationFrame(() => {
			if (chatContainer) {
				chatContainer.scrollTop = chatContainer.scrollHeight;
			}
		});
	}

	function addMessages(next: ChatMessage[]): void {
		messages = [...messages, ...next];
		scrollToBottom();
	}

	function setModeInteractive(): void {
		mode = 'interactive';
	}

	function setModeScan(): void {
		mode = 'scan';
	}

	function setMessageViewFormatted(): void {
		messageView = 'formatted';
	}

	function setMessageViewRaw(): void {
		messageView = 'raw';
	}

	function isLongObservation(message: ChatMessage): boolean {
		return message.role === 'observation' && message.content.length > 700;
	}

	async function hydrateSessionHistory(id: string): Promise<void> {
		const events = await apiClient.getSessionEvents(id);
		messages = replayEvents(events);
		scrollToBottom();
	}

	async function loadExistingSession(id: string): Promise<void> {
		try {
			const sessions = await apiClient.listSessions();
			const session = sessions.find((entry) => entry.id === id);
			if (!session) {
				throw new Error(`Sesi ${id} tidak ditemukan`);
			}

			target = session.target;
			mode = session.mode;
			showSetup = false;
			sessionId = session.id;
			loadedSessionId = session.id;
			await hydrateSessionHistory(session.id);

			ws?.close();
			wsSessionId = null;
			if (session.active) {
				ws = createDalangWebSocket(session.id, {
					onOpen: () => {
						isConnected = true;
					},
					onEvent: handleEvent,
					onClose: () => {
						isConnected = false;
						isReconnecting = false;
					},
					onError: () => {
						isConnected = false;
					},
					onReconnecting: () => {
						isReconnecting = true;
						toast.warning('Menyambungkan ulang ke sesi…');
					},
					onReconnected: () => {
						isConnected = true;
						isReconnecting = false;
						toast.success('Sesi tersambung kembali');
					}
				});
				wsSessionId = session.id;
			} else {
				ws = null;
				wsSessionId = null;
				isConnected = false;
			}
		} catch (error) {
			toast.error(
				`Gagal memuat sesi: ${error instanceof Error ? error.message : 'kesalahan tidak diketahui'}`
			);
		}
	}

	async function syncFromUrl(): Promise<void> {
		const params = new URLSearchParams(window.location.search);
		const id = params.get('session');
		if (id && id !== loadedSessionId) {
			await loadExistingSession(id);
		} else if (!id && loadedSessionId) {
			resetSession();
		}
	}

	function handleEvent(event: EngineEvent): void {
		if (!sessionId || !ws || !wsSessionId || wsSessionId !== sessionId || showSetup) return;
		const next = replayEvents([event]);
		addMessages(next);
		if (event.type === 'assistant_message' || event.type === 'error' || event.type === 'done') {
			isThinking = false;
		}
	}

	async function startSession(): Promise<void> {
		if (!target.trim()) return;

		ws?.close();
		ws = null;
		wsSessionId = null;
		loadedSessionId = null;
		sessionId = null;
		isConnected = false;
		isReconnecting = false;
		messageView = 'formatted';
		showSetup = false;
		messages = [];
		isThinking = false;

		try {
			const session = await apiClient.createSession(target, mode);
			sessionId = session.id;
			loadedSessionId = session.id;

			ws = createDalangWebSocket(session.id, {
				onOpen: () => {
					isConnected = true;
				},
				onEvent: handleEvent,
				onClose: () => {
					isConnected = false;
					isReconnecting = false;
				},
				onError: () => {
					isConnected = false;
				},
				onReconnecting: () => {
					isReconnecting = true;
					toast.warning('Menyambungkan ulang ke sesi…');
				},
				onReconnected: () => {
					isConnected = true;
					isReconnecting = false;
					toast.success('Sesi tersambung kembali');
				}
			});
			wsSessionId = session.id;

			addMessages([
				{ role: 'status', content: `Pemeriksaan dimulai untuk: ${target}` }
			]);

			if (mode === 'scan') {
				await ws.startScan(target, maxIter, cmdTimeout);
				addMessages([
					{
						role: 'status',
						content:
							maxIter === 0
								? 'Mode otomatis berjalan (tanpa batas langkah).'
								: `Mode otomatis berjalan (maks. ${maxIter} langkah).`
					}
				]);
			} else {
				await ws.startInteractive(target, cmdTimeout);
			}
		} catch (error) {
			const message = `Gagal memulai sesi: ${error instanceof Error ? error.message : 'Kesalahan tidak diketahui'}`;
			addMessages([{ role: 'error', content: message }]);
			toast.error(message);
		}
	}

	function sendMessage(): void {
		if (!ws || !isConnected || !inputText.trim()) return;
		const message = inputText.trim();
		inputText = '';
		isThinking = true;
		addMessages([{ role: 'user', content: message }]);
		ws.sendChat(message);
	}

	async function reloadSessionEvents(): Promise<void> {
		if (!sessionId) return;
		try {
			await hydrateSessionHistory(sessionId);
		} catch (error) {
			toast.error(
				`Gagal memuat ulang riwayat: ${error instanceof Error ? error.message : 'kesalahan tidak diketahui'}`
			);
			addMessages([
				{
					role: 'error',
					content: `Gagal memuat ulang peristiwa sesi: ${error instanceof Error ? error.message : 'Kesalahan tidak diketahui'}`
				}
			]);
		}
	}

	function resetSession(): void {
		ws?.close();
		ws = null;
		wsSessionId = null;
		sessionId = null;
		isConnected = false;
		isReconnecting = false;
		isThinking = false;
		messages = [];
		inputText = '';
		showSetup = true;
		loadedSessionId = null;
		messageView = 'formatted';
	}

	function shortSessionLabel(id: string | null): string {
		if (!id) return '—';
		if (id.length <= 14) return id;
		return `${id.slice(0, 8)}…${id.slice(-4)}`;
	}

	async function copyFullSessionId(): Promise<void> {
		if (!sessionId) return;
		try {
			await navigator.clipboard.writeText(sessionId);
			toast.success('ID sesi disalin');
		} catch {
			toast.error('Gagal menyalin ID sesi');
		}
	}

	onMount(() => {
		syncFromUrl();
	});

	afterNavigate(() => {
		syncFromUrl();
	});
</script>

<section class="space-y-4">
	<header
		class="surface-panel dashboard-warboard flex flex-wrap items-center justify-between gap-3 p-4"
	>
		<div>
			<p class="text-xs tracking-[0.2em] text-(--color-ash) uppercase">Dasbor / Percakapan</p>
			<h2 class="text-xl font-semibold text-(--color-text)">Pemeriksaan keamanan</h2>
			<p class="text-xs text-(--color-ash)">
				Jalankan pemeriksaan otomatis atau arahkan AI dengan percakapan — alur tetap sama saat Anda
				membuka ulang tautan sesi.
			</p>
		</div>
		<div class="flex items-center gap-2">
			<a href={resolve('/dashboard')} class="control-chip">Ringkasan</a>
			{#if !showSetup}
				<button class="control-chip" onclick={reloadSessionEvents}>Muat ulang riwayat</button>
				<button class="control-chip" onclick={resetSession}>Sesi baru</button>
			{/if}
		</div>
	</header>

	{#if showSetup}
		<div class="surface-panel space-y-5 p-5">
			<div class="dashboard-inline-pills">
				<span class="dashboard-pill">Menyiapkan pemeriksaan</span>
				<span class="dashboard-pill">Alur aman</span>
				<span class="dashboard-pill">Langsung dari server</span>
			</div>

			<div class="space-y-2">
				<label class="text-sm font-medium text-(--color-text)" for="target"
					>Alamat yang akan diperiksa</label
				>
				<input
					id="target"
					bind:value={target}
					type="text"
					placeholder="Contoh: https://contoh.com atau alamat IP"
					class="w-full rounded-lg border border-(--color-border) bg-transparent px-3 py-2 text-sm text-(--color-text) outline-none focus:border-(--color-gold)"
				/>
				<p class="text-xs text-(--color-ash)">
					Pastikan Anda punya izin resmi untuk memeriksa target ini (pemilik sistem, kontrak, atau
					lingkungan uji).
				</p>
			</div>

			<div>
				<p class="mb-2 text-sm font-medium text-(--color-text)">Mode pemeriksaan</p>
				<div class="grid gap-3 sm:grid-cols-2">
					<button
						type="button"
						class="rounded-lg border px-4 py-3 text-left text-sm transition {mode ===
						'interactive'
							? 'border-(--color-gold) bg-(--color-gold)/10 text-(--color-gold)'
							: 'border-(--color-border) text-(--color-ash) hover:border-(--color-gold)/40'}"
						onclick={setModeInteractive}
					>
						<span class="block font-semibold text-(--color-text)">Percakapan</span>
						<span class="mt-1 block text-xs leading-relaxed text-(--color-ash)">
							Anda mengarahkan dengan pertanyaan atau permintaan; AI menjalankan langkah sesuai arahan
							Anda.
						</span>
					</button>
					<button
						type="button"
						class="rounded-lg border px-4 py-3 text-left text-sm transition {mode === 'scan'
							? 'border-(--color-gold) bg-(--color-gold)/10 text-(--color-gold)'
							: 'border-(--color-border) text-(--color-ash) hover:border-(--color-gold)/40'}"
						onclick={setModeScan}
					>
						<span class="block font-semibold text-(--color-text)">Otomatis</span>
						<span class="mt-1 block text-xs leading-relaxed text-(--color-ash)">
							Sistem menjalankan rangkaian pemeriksaan bertahap hingga selesai atau mencapai batas
							langkah.
						</span>
					</button>
				</div>
			</div>

			<details class="rounded-lg border border-(--color-border) bg-(--color-surface)/60 px-3 py-2">
				<summary
					class="cursor-pointer select-none text-sm text-(--color-gold) hover:underline"
				>
					Pengaturan lanjutan (opsional)
				</summary>
				<div class="mt-3 space-y-4 pb-2">
					{#if mode === 'scan'}
						<div class="space-y-2">
							<label class="text-sm text-(--color-ash)" for="max-iter"
								>Batas langkah AI (0 = tanpa batas)</label
							>
							<p class="text-xs text-(--color-ash)">
								Hanya dipakai pada mode otomatis; membatasi berapa kali AI boleh beraksi berturut-turut.
							</p>
							<input
								id="max-iter"
								bind:value={maxIter}
								type="number"
								min="0"
								class="w-full rounded-lg border border-(--color-border) bg-transparent px-3 py-2 text-sm text-(--color-text) outline-none focus:border-(--color-gold)"
							/>
						</div>
					{/if}
					<div class="space-y-2">
						<label class="text-sm text-(--color-ash)" for="cmd-timeout"
							>Batas waktu per perintah (detik)</label
						>
						<p class="text-xs text-(--color-ash)">
							Jika sebuah langkah memakan waktu lebih lama, pemeriksaan akan dihentikan untuk langkah
							itu agar tidak menggantung.
						</p>
						<input
							id="cmd-timeout"
							bind:value={cmdTimeout}
							type="number"
							min="0"
							class="w-full rounded-lg border border-(--color-border) bg-transparent px-3 py-2 text-sm text-(--color-text) outline-none focus:border-(--color-gold)"
						/>
					</div>
				</div>
			</details>

			<button
				class="rounded-lg bg-(--color-gold) px-4 py-2 text-sm font-semibold text-[#1f1708] disabled:opacity-50"
				disabled={!target.trim()}
				onclick={startSession}>Mulai pemeriksaan</button
			>
		</div>
	{:else}
		<div class="rounded-2xl border border-(--color-border) bg-(--color-surface)">
			<div
				class="flex flex-wrap items-center justify-between gap-2 border-b border-(--color-border) px-4 py-2 text-xs text-(--color-ash)"
			>
				<div class="flex min-w-0 flex-wrap items-center gap-2">
					<p class="truncate">
						Sesi: <span class="font-mono text-(--color-text)" title={sessionId ?? ''}
							>{shortSessionLabel(sessionId)}</span
						>
					</p>
					<button
						type="button"
						class="shrink-0 text-(--color-gold) underline-offset-2 hover:underline"
						onclick={copyFullSessionId}>Salin ID lengkap</button
					>
					<details class="shrink-0">
						<summary class="cursor-pointer text-(--color-gold) hover:underline"
							>ID teknis</summary
						>
						<p class="mt-1 max-w-[min(100%,32rem)] break-all font-mono text-[10px] text-(--color-text)">
							{sessionId}
						</p>
					</details>
				</div>
				<div class="flex flex-wrap items-center gap-2">
					<button
						class="control-chip px-2! py-1!"
						onclick={setMessageViewFormatted}
						disabled={messageView === 'formatted'}>Ringkas</button
					>
					<button
						class="control-chip px-2! py-1!"
						onclick={setMessageViewRaw}
						disabled={messageView === 'raw'}>Mentah</button
					>
					<p>
						{isReconnecting
							? 'Menyambung ulang…'
							: isConnected
								? 'Terhubung'
								: 'Tidak terhubung'}
					</p>
				</div>
			</div>

			<div
				bind:this={chatContainer}
				class="max-h-[62vh] min-h-[45vh] space-y-2 overflow-auto px-4 py-3"
			>
				{#if messages.length === 0}
					<div
						class="flex min-h-[40vh] flex-col items-center justify-center gap-2 px-4 text-center"
					>
						<p class="text-sm text-(--color-ash)">
							{#if isThinking}
								Menunggu respons pertama dari mesin pemeriksaan…
							{:else if !isConnected}
								Belum terhubung ke server. Pastikan layanan Dalang berjalan di mesin Anda.
							{:else}
								Belum ada pesan. Ketik permintaan di bawah atau tunggu keluaran mode otomatis.
							{/if}
						</p>
					</div>
				{:else}
					{#each messages as message, index (`${message.role}-${index}`)}
						<div
							class="max-w-full overflow-x-hidden rounded-lg border border-(--color-border) p-3 text-sm"
						>
							<p class="mb-1 text-xs font-medium text-(--color-ash)">
								{chatRoleLabel(message.role)}
							</p>
							{#if message.filename}
								<p class="mb-2 text-xs text-(--color-ash)">
									Berkas tersimpan: <span class="font-mono">{message.filename}</span>
								</p>
							{/if}
							{#if messageView === 'raw'}
								<pre class="dashboard-raw" dir="auto">{renderMarkdownRaw(message.content)}</pre>
								{#if message.toolCommand}
									<pre class="dashboard-raw mt-2 border-t border-(--color-border) pt-2" dir="ltr"
										>{message.toolCommand}</pre
									>
								{/if}
							{:else if message.role === 'observation' && isLongObservation(message)}
								<p class="mb-1 text-xs text-(--color-ash)">
									Sumber: <span class="font-medium text-(--color-text)">{message.skill ?? '—'}</span>
									· {message.bytes?.toLocaleString('id-ID') ?? '—'} byte
								</p>
								<details class="rounded-md border border-(--color-border) bg-(--color-surface) p-2">
									<summary
										class="cursor-pointer select-none text-xs text-(--color-gold) hover:underline"
									>
										Tampilkan hasil lengkap
									</summary>
									<div class="dashboard-markdown mt-2 max-h-[50vh] overflow-auto" dir="auto">
										<!-- eslint-disable-next-line svelte/no-at-html-tags -->
										{@html renderMarkdown(message.content)}
									</div>
								</details>
							{:else}
								<div class="dashboard-markdown" dir="auto">
									<!-- eslint-disable-next-line svelte/no-at-html-tags -->
									{@html renderMarkdown(message.content)}
								</div>
								{#if message.toolCommand}
									<details
										class="mt-2 rounded-md border border-(--color-border) bg-(--color-surface)/80 p-2"
									>
										<summary
											class="cursor-pointer select-none text-xs text-(--color-ash) hover:text-(--color-gold)"
										>
											Detail perintah teknis
										</summary>
										<pre
											class="mt-2 overflow-x-auto rounded bg-[color-mix(in_oklab,var(--color-border)_40%,transparent)] p-2 font-mono text-xs whitespace-pre-wrap"
											dir="ltr">{message.toolCommand}</pre
										>
									</details>
								{/if}
							{/if}
						</div>
					{/each}
				{/if}
			</div>

			<div class="flex items-end gap-2 border-t border-(--color-border) px-4 py-3">
				<textarea
					bind:value={inputText}
					placeholder="Tulis pertanyaan atau instruksi untuk AI…"
					rows="2"
					class="min-h-11 flex-1 resize-y rounded-lg border border-(--color-border) bg-transparent px-3 py-2 text-sm text-(--color-text) outline-none focus:border-(--color-gold)"
					onkeydown={(event) =>
						event.key === 'Enter' && !event.shiftKey && (event.preventDefault(), sendMessage())}
				></textarea>
				<button
					class="rounded-lg bg-(--color-gold) px-4 py-2 text-sm font-semibold text-[#1f1708] disabled:opacity-50"
					disabled={!isConnected || isThinking || !inputText.trim()}
					onclick={sendMessage}
				>
					Kirim
				</button>
			</div>
		</div>
	{/if}
</section>
