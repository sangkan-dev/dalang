<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import { apiClient } from '$lib/api/client.js';
	import { replayEvents } from '$lib/api/events.js';
	import { createDalangWebSocket } from '$lib/api/websocket.js';
	import type { ChatMessage, DalangWebSocket, EngineEvent, SessionMode } from '$lib/api/types.js';
	import { toast } from '$lib/dashboard/toast.js';
	import { onMount } from 'svelte';

	let mode: SessionMode = 'interactive';
	let target = '';
	let inputText = '';
	let messages: ChatMessage[] = [];
	let sessionId: string | null = null;
	let isThinking = false;
	let isConnected = false;
	let isReconnecting = false;
	let maxIter = 15;
	let cmdTimeout = 300;
	let showSetup = true;
	let ws: DalangWebSocket | null = null;
	let chatContainer: HTMLDivElement | null = null;
	let loadedSessionId: string | null = null;

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
			} else {
				ws = null;
				isConnected = false;
			}
		} catch (error) {
			toast.error(`Failed to load session: ${error instanceof Error ? error.message : 'unknown error'}`);
		}
	}

	async function syncFromUrl(): Promise<void> {
		const params = new URLSearchParams(window.location.search);
		const id = params.get('session');
		if (id && id !== loadedSessionId) {
			await loadExistingSession(id);
		}
	}

	function handleEvent(event: EngineEvent): void {
		const next = replayEvents([event]);
		addMessages(next);
		if (event.type === 'assistant_message' || event.type === 'error' || event.type === 'done') {
			isThinking = false;
		}
	}

	async function startSession(): Promise<void> {
		if (!target.trim()) return;

		showSetup = false;
		messages = [];
		isThinking = false;

		try {
			const session = await apiClient.createSession(target, mode);
			sessionId = session.id;

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

			addMessages([{ role: 'status', content: `Session started for target: ${target}` }]);

			if (mode === 'scan') {
				await ws.startScan(target, maxIter, cmdTimeout);
				addMessages([
					{
						role: 'status',
						content: maxIter === 0 ? 'Auto-pilot scan started (unlimited iterations).' : `Auto-pilot scan started (max ${maxIter} iterations).`
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
			toast.error(`Failed to replay events: ${error instanceof Error ? error.message : 'unknown error'}`);
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
		sessionId = null;
		isConnected = false;
		isReconnecting = false;
		isThinking = false;
		messages = [];
		inputText = '';
		showSetup = true;
		loadedSessionId = null;
	}

	onMount(() => {
		syncFromUrl();
	});

	afterNavigate(() => {
		syncFromUrl();
	});
</script>

<section class="space-y-4">
	<header class="flex items-center justify-between">
		<div>
			<p class="text-xs uppercase tracking-[0.2em] text-[color:var(--color-ash)]">Dashboard / Chat</p>
			<h2 class="text-xl font-semibold text-[color:var(--color-text)]">Interactive Console</h2>
		</div>
		<div class="flex items-center gap-2">
			<a href="/dashboard" class="rounded-lg border border-[color:var(--color-border)] px-3 py-2 text-xs text-[color:var(--color-ash)]">Overview</a>
			{#if !showSetup}
				<button class="rounded-lg border border-[color:var(--color-border)] px-3 py-2 text-xs text-[color:var(--color-ash)]" onclick={reloadSessionEvents}>Replay</button>
				<button class="rounded-lg border border-[color:var(--color-border)] px-3 py-2 text-xs text-[color:var(--color-ash)]" onclick={resetSession}>Reset</button>
			{/if}
		</div>
	</header>

	{#if showSetup}
		<div class="space-y-4 rounded-2xl border border-[color:var(--color-border)] bg-[color:var(--color-surface)] p-5">
			<div class="space-y-2">
				<label class="text-sm text-[color:var(--color-ash)]" for="target">Target</label>
				<input id="target" bind:value={target} type="text" placeholder="https://example.com or 192.168.1.1" class="w-full rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-text)] outline-none focus:border-[color:var(--color-gold)]" />
			</div>

			<div class="grid gap-3 sm:grid-cols-2">
				<button class="rounded-lg border px-3 py-2 text-sm {mode === 'interactive' ? 'border-[color:var(--color-gold)] text-[color:var(--color-gold)]' : 'border-[color:var(--color-border)] text-[color:var(--color-ash)]'}" onclick={() => (mode = 'interactive')}>Interactive</button>
				<button class="rounded-lg border px-3 py-2 text-sm {mode === 'scan' ? 'border-[color:var(--color-gold)] text-[color:var(--color-gold)]' : 'border-[color:var(--color-border)] text-[color:var(--color-ash)]'}" onclick={() => (mode = 'scan')}>Auto-pilot</button>
			</div>

			{#if mode === 'scan'}
				<div class="grid gap-3 sm:grid-cols-2">
					<div class="space-y-2">
						<label class="text-sm text-[color:var(--color-ash)]" for="max-iter">Max iterations (0 = unlimited)</label>
						<input id="max-iter" bind:value={maxIter} type="number" min="0" class="w-full rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-text)] outline-none focus:border-[color:var(--color-gold)]" />
					</div>
					<div class="space-y-2">
						<label class="text-sm text-[color:var(--color-ash)]" for="cmd-timeout">Command timeout (seconds)</label>
						<input id="cmd-timeout" bind:value={cmdTimeout} type="number" min="0" class="w-full rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-text)] outline-none focus:border-[color:var(--color-gold)]" />
					</div>
				</div>
			{/if}

			<button class="rounded-lg bg-[color:var(--color-gold)] px-4 py-2 text-sm font-semibold text-[color:#1f1708]" onclick={startSession}>Start session</button>
		</div>
	{:else}
		<div class="rounded-2xl border border-[color:var(--color-border)] bg-[color:var(--color-surface)]">
			<div class="flex items-center justify-between border-b border-[color:var(--color-border)] px-4 py-2 text-xs text-[color:var(--color-ash)]">
				<p>Session: <span class="font-mono">{sessionId}</span></p>
				<p>{isReconnecting ? 'Reconnecting...' : isConnected ? 'Connected' : 'Disconnected'}</p>
			</div>

			<div bind:this={chatContainer} class="max-h-[55vh] space-y-2 overflow-y-auto px-4 py-3">
				{#each messages as message}
					<div class="rounded-lg border border-[color:var(--color-border)] p-3 text-sm">
						<p class="mb-1 text-xs uppercase tracking-[0.12em] text-[color:var(--color-ash)]">{message.role}</p>
						<pre class="whitespace-pre-wrap break-words font-sans text-[color:var(--color-text)]">{message.content}</pre>
					</div>
				{/each}
			</div>

			<div class="flex items-end gap-2 border-t border-[color:var(--color-border)] px-4 py-3">
				<textarea bind:value={inputText} placeholder="Ask the agent..." rows="2" class="min-h-11 flex-1 resize-y rounded-lg border border-[color:var(--color-border)] bg-transparent px-3 py-2 text-sm text-[color:var(--color-text)] outline-none focus:border-[color:var(--color-gold)]" onkeydown={(event) => event.key === 'Enter' && !event.shiftKey && (event.preventDefault(), sendMessage())}></textarea>
				<button class="rounded-lg bg-[color:var(--color-gold)] px-4 py-2 text-sm font-semibold text-[color:#1f1708] disabled:opacity-50" disabled={!isConnected || isThinking || !inputText.trim()} onclick={sendMessage}>
					Send
				</button>
			</div>
		</div>
	{/if}
</section>