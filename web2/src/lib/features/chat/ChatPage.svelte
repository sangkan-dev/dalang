<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { apiClient } from '$lib/api/client.js';
	import { replayEvents } from '$lib/api/events.js';
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
				throw new Error(`Session ${id} not found`);
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
						toast.warning('Reconnecting to session...');
					},
					onReconnected: () => {
						isConnected = true;
						isReconnecting = false;
						toast.success('Session reconnected');
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
				`Failed to load session: ${error instanceof Error ? error.message : 'unknown error'}`
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
					toast.warning('Reconnecting to session...');
				},
				onReconnected: () => {
					isConnected = true;
					isReconnecting = false;
					toast.success('Session reconnected');
				}
			});
			wsSessionId = session.id;

			addMessages([{ role: 'status', content: `Session started for target: ${target}` }]);

			if (mode === 'scan') {
				await ws.startScan(target, maxIter, cmdTimeout);
				addMessages([
					{
						role: 'status',
						content:
							maxIter === 0
								? 'Auto-pilot scan started (unlimited iterations).'
								: `Auto-pilot scan started (max ${maxIter} iterations).`
					}
				]);
			} else {
				await ws.startInteractive(target, cmdTimeout);
			}
		} catch (error) {
			const message = `Failed to start session: ${error instanceof Error ? error.message : 'Unknown error'}`;
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
				`Failed to replay events: ${error instanceof Error ? error.message : 'unknown error'}`
			);
			addMessages([
				{
					role: 'error',
					content: `Failed to reload session events: ${error instanceof Error ? error.message : 'Unknown error'}`
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
			<p class="text-xs tracking-[0.2em] text-(--color-ash) uppercase">Dashboard / Chat</p>
			<h2 class="text-xl font-semibold text-(--color-text)">Interactive Console</h2>
			<p class="text-xs text-(--color-ash)">Live operator stream with replay-safe state flow</p>
		</div>
		<div class="flex items-center gap-2">
			<a href={resolve('/dashboard')} class="control-chip">Overview</a>
			{#if !showSetup}
				<button class="control-chip" onclick={reloadSessionEvents}>Replay</button>
				<button class="control-chip" onclick={resetSession}>Reset</button>
			{/if}
		</div>
	</header>

	{#if showSetup}
		<div class="surface-panel space-y-4 p-5">
			<div class="dashboard-inline-pills">
				<span class="dashboard-pill">Session Setup</span>
				<span class="dashboard-pill">State Safe</span>
				<span class="dashboard-pill">Realtime Stream</span>
			</div>
			<div class="space-y-2">
				<label class="text-sm text-(--color-ash)" for="target">Target</label>
				<input
					id="target"
					bind:value={target}
					type="text"
					placeholder="https://example.com or 192.168.1.1"
					class="w-full rounded-lg border border-(--color-border) bg-transparent px-3 py-2 text-sm text-(--color-text) outline-none focus:border-(--color-gold)"
				/>
			</div>

			<div class="grid gap-3 sm:grid-cols-2">
				<button
					class="rounded-lg border px-3 py-2 text-sm {mode === 'interactive'
						? 'border-(--color-gold) text-(--color-gold)'
						: 'border-(--color-border) text-(--color-ash)'}"
					onclick={setModeInteractive}>Interactive</button
				>
				<button
					class="rounded-lg border px-3 py-2 text-sm {mode === 'scan'
						? 'border-(--color-gold) text-(--color-gold)'
						: 'border-(--color-border) text-(--color-ash)'}"
					onclick={setModeScan}>Auto-pilot</button
				>
			</div>

			{#if mode === 'scan'}
				<div class="grid gap-3 sm:grid-cols-2">
					<div class="space-y-2">
						<label class="text-sm text-(--color-ash)" for="max-iter"
							>Max iterations (0 = unlimited)</label
						>
						<input
							id="max-iter"
							bind:value={maxIter}
							type="number"
							min="0"
							class="w-full rounded-lg border border-(--color-border) bg-transparent px-3 py-2 text-sm text-(--color-text) outline-none focus:border-(--color-gold)"
						/>
					</div>
					<div class="space-y-2">
						<label class="text-sm text-(--color-ash)" for="cmd-timeout"
							>Command timeout (seconds)</label
						>
						<input
							id="cmd-timeout"
							bind:value={cmdTimeout}
							type="number"
							min="0"
							class="w-full rounded-lg border border-(--color-border) bg-transparent px-3 py-2 text-sm text-(--color-text) outline-none focus:border-(--color-gold)"
						/>
					</div>
				</div>
			{/if}

			<button
				class="rounded-lg bg-(--color-gold) px-4 py-2 text-sm font-semibold text-[#1f1708]"
				onclick={startSession}>Start session</button
			>
		</div>
	{:else}
		<div class="rounded-2xl border border-(--color-border) bg-(--color-surface)">
			<div
				class="flex flex-wrap items-center justify-between gap-2 border-b border-(--color-border) px-4 py-2 text-xs text-(--color-ash)"
			>
				<p>Session: <span class="font-mono">{sessionId}</span></p>
				<div class="flex items-center gap-2">
					<button
						class="control-chip px-2! py-1!"
						onclick={setMessageViewFormatted}
						disabled={messageView === 'formatted'}>Formatted</button
					>
					<button
						class="control-chip px-2! py-1!"
						onclick={setMessageViewRaw}
						disabled={messageView === 'raw'}>Raw</button
					>
					<p>{isReconnecting ? 'Reconnecting...' : isConnected ? 'Connected' : 'Disconnected'}</p>
				</div>
			</div>

			<div
				bind:this={chatContainer}
				class="max-h-[62vh] min-h-[45vh] space-y-2 overflow-auto px-4 py-3"
			>
				{#each messages as message, index (`${message.role}-${index}`)}
					<div
						class="max-w-full overflow-x-hidden rounded-lg border border-(--color-border) p-3 text-sm"
					>
						<p class="mb-1 text-xs tracking-[0.12em] text-(--color-ash) uppercase">
							{message.role}
						</p>
						{#if messageView === 'raw'}
							<pre class="dashboard-raw" dir="auto">{renderMarkdownRaw(message.content)}</pre>
						{:else}
							<div class="dashboard-markdown" dir="auto">
								<!-- eslint-disable-next-line svelte/no-at-html-tags -->
								{@html renderMarkdown(message.content)}
							</div>
						{/if}
					</div>
				{/each}
			</div>

			<div class="flex items-end gap-2 border-t border-(--color-border) px-4 py-3">
				<textarea
					bind:value={inputText}
					placeholder="Ask the agent..."
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
					Send
				</button>
			</div>
		</div>
	{/if}
</section>
