<script lang="ts">
  import { api, createWebSocket } from '../lib/api.ts';
  import { renderMarkdown } from '../lib/markdown.ts';
  import { toast } from '../lib/toast.ts';
  import ChatMessage from './ChatMessage.svelte';
  import type { ChatMessage as ChatMsg, DalangWebSocket, EngineEvent, SessionMode } from '../lib/types';

  let { sessionId = $bindable(null), resetTrigger = 0 }: { sessionId: string | null; resetTrigger?: number } = $props();

  let messages = $state<ChatMsg[]>([]);
  let inputText = $state('');
  let target = $state('');
  let isConnected = $state(false);
  let isThinking = $state(false);
  let isReconnecting = $state(false);
  let mode = $state<SessionMode>('interactive');
  let maxIter = $state(15);
  let cmdTimeout = $state(300);
  let ws = $state<DalangWebSocket | null>(null);
  let chatContainer = $state<HTMLDivElement | null>(null);
  let showSetup = $state(true);

  // Watch for resetTrigger changes from parent (new session via sidebar)
  $effect(() => {
    if (resetTrigger) {
      resetSession();
    }
  });

  function scrollToBottom(): void {
    const el = chatContainer;
    if (el) {
      requestAnimationFrame(() => {
        el.scrollTop = el.scrollHeight;
      });
    }
  }

  function handleEvent(event: EngineEvent): void {
    switch (event.type) {
      case 'thinking':
        isThinking = true;
        messages = [...messages, {
          role: 'status',
          content: event.max_iter
            ? `Reasoning (iteration ${event.iteration}/${event.max_iter})...`
            : `Reasoning (iteration ${event.iteration})...`,
        }];
        break;

      case 'assistant_message':
        isThinking = false;
        messages = [...messages, { role: 'assistant', content: event.content }];
        break;

      case 'tool_execution':
        messages = [...messages, {
          role: 'tool',
          content: `Executing skill: **${event.skill}**\n\`\`\`bash\n${event.command}\n\`\`\``,
        }];
        break;

      case 'observation':
        messages = [...messages, {
          role: 'observation',
          content: event.content,
          bytes: event.bytes,
          skill: event.skill,
        }];
        break;

      case 'safety_refusal':
        messages = [...messages, {
          role: 'warning',
          content: `Safety filter triggered (retry ${event.retry}). Re-prompting...`,
        }];
        break;

      case 'browser_action':
        messages = [...messages, {
          role: 'tool',
          content: `Browser: ${event.action} — ${event.success ? '✓' : '✗'}\n${event.content.substring(0, 500)}`,
        }];
        break;

      case 'report':
        messages = [...messages, {
          role: 'report',
          content: event.markdown,
          filename: event.filename ?? undefined,
        }];
        break;

      case 'status':
        messages = [...messages, { role: 'status', content: event.message }];
        break;

      case 'error':
        isThinking = false;
        messages = [...messages, { role: 'error', content: event.message }];
        break;

      case 'done':
        isThinking = false;
        messages = [...messages, { role: 'status', content: `✓ ${event.reason}` }];
        break;
    }
    scrollToBottom();
  }

  async function startSession(): Promise<void> {
    if (!target.trim()) return;
    showSetup = false;

    try {
      const session = await api.createSession(target, mode);
      sessionId = session.id;

      ws = createWebSocket(session.id, {
        onEvent: handleEvent,
        onClose: () => { isConnected = false; isReconnecting = false; },
        onError: () => { isConnected = false; },
        onReconnecting: (attempt, max) => {
          isReconnecting = true;
          toast.warning(`Reconnecting... (${attempt}/${max})`);
        },
        onReconnected: () => {
          isReconnecting = false;
          isConnected = true;
          toast.success('Reconnected!');
        },
      });

      isConnected = true;
      messages = [{ role: 'status', content: `Session started for target: ${target}` }];

      if (mode === 'scan') {
        // Auto-start scan
        const conn = ws;
        setTimeout(() => {
          conn.startScan(target, maxIter, cmdTimeout);
          messages = [...messages, { role: 'status', content: `Auto-pilot scan started (max ${maxIter} iterations)` }];
        }, 500);
      } else {
        ws.startInteractive(target, cmdTimeout);
      }
    } catch (err) {
      messages = [{ role: 'error', content: `Failed to create session: ${(err as Error).message}` }];
    }
  }

  function sendMessage(): void {
    if (!inputText.trim() || !ws || !isConnected) return;
    messages = [...messages, { role: 'user', content: inputText }];
    ws.sendChat(inputText);
    inputText = '';
    isThinking = true;
    scrollToBottom();
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  function resetSession(): void {
    if (ws) ws.close();
    ws = null;
    sessionId = null;
    messages = [];
    isConnected = false;
    isThinking = false;
    isReconnecting = false;
    showSetup = true;
    target = '';
  }
</script>

{#if showSetup}
  <!-- Setup screen -->
  <div class="flex-1 flex items-center justify-center p-8">
    <div class="w-full max-w-lg space-y-6">
      <div class="text-center mb-8">
        <h1 class="text-3xl font-bold mb-2">🛡️ Dalang</h1>
        <p class="text-[var(--text-secondary)]">AI-Powered Cybersecurity Agent</p>
      </div>

      <div class="bg-[var(--bg-secondary)] rounded-xl p-6 space-y-4 border border-[var(--border)]">
        <div>
          <label class="block text-sm font-medium mb-1.5 text-[var(--text-secondary)]" for="target">Target</label>
          <input
            id="target"
            type="text"
            bind:value={target}
            placeholder="e.g., https://example.com or 192.168.1.1"
            class="w-full px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
              text-[var(--text-primary)] placeholder-[var(--text-secondary)]/50
              focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]"
            onkeydown={(e) => e.key === 'Enter' && startSession()}
          />
        </div>

        <div>
          <label class="block text-sm font-medium mb-1.5 text-[var(--text-secondary)]">Mode</label>
          <div class="flex gap-3">
            <button
              class="flex-1 py-2 rounded-lg text-sm font-medium transition-colors border
                {mode === 'interactive'
                  ? 'bg-[var(--accent)]/10 border-[var(--accent)] text-[var(--accent)]'
                  : 'bg-[var(--bg-primary)] border-[var(--border)] text-[var(--text-secondary)] hover:border-[var(--text-secondary)]'}"
              onclick={() => mode = 'interactive'}
            >
              💬 Interactive
            </button>
            <button
              class="flex-1 py-2 rounded-lg text-sm font-medium transition-colors border
                {mode === 'scan'
                  ? 'bg-[var(--accent)]/10 border-[var(--accent)] text-[var(--accent)]'
                  : 'bg-[var(--bg-primary)] border-[var(--border)] text-[var(--text-secondary)] hover:border-[var(--text-secondary)]'}"
              onclick={() => mode = 'scan'}
            >
              🤖 Auto-Pilot Scan
            </button>
          </div>
        </div>

        {#if mode === 'scan'}
          <div>
            <label class="block text-sm font-medium mb-1.5 text-[var(--text-secondary)]" for="maxiter">
              Max Iterations
              <span class="text-xs text-[var(--text-secondary)]/60">(0 = unlimited)</span>
            </label>
            <input
              id="maxiter"
              type="number"
              bind:value={maxIter}
              class="w-full px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
                text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
            />
          </div>
        {/if}

        <div>
          <label class="block text-sm font-medium mb-1.5 text-[var(--text-secondary)]" for="cmdtimeout">
            Command Timeout (seconds)
            <span class="text-xs text-[var(--text-secondary)]/60">(0 = unlimited)</span>
          </label>
          <input
            id="cmdtimeout"
            type="number"
            bind:value={cmdTimeout}
            class="w-full px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
              text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
          />
        </div>

        <button
          class="w-full py-3 bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-[var(--bg-primary)]
            font-semibold rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          disabled={!target.trim()}
          onclick={startSession}
        >
          Start Session
        </button>
      </div>
    </div>
  </div>
{:else}
  <!-- Chat interface -->
  <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--border)] bg-[var(--bg-secondary)]">
    <div class="flex items-center gap-3">
      <span class="w-2 h-2 rounded-full {isConnected ? 'bg-[var(--success)]' : 'bg-[var(--danger)]'}"></span>
      <span class="text-sm font-medium">{target}</span>
      <span class="text-xs px-2 py-0.5 rounded-full bg-[var(--bg-tertiary)] text-[var(--text-secondary)]">
        {mode === 'scan' ? '🤖 Auto-Pilot' : '💬 Interactive'}
      </span>
    </div>
    <button
      class="text-xs px-3 py-1.5 rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-secondary)]
        hover:text-[var(--danger)] hover:bg-[var(--danger)]/10 transition-colors"
      onclick={resetSession}
    >
      End Session
    </button>
  </div>

  <!-- Reconnecting banner -->
  {#if isReconnecting}
    <div class="px-4 py-2 bg-amber-950/30 border-b border-amber-800/30 text-amber-300 text-xs flex items-center gap-2">
      <span class="animate-spin">↻</span>
      <span>Reconnecting to server...</span>
    </div>
  {/if}

  <!-- Messages -->
  <div class="flex-1 overflow-y-auto p-4 space-y-3" bind:this={chatContainer}>
    {#each messages as msg, i (i)}
      <ChatMessage message={msg} />
    {/each}

    {#if isThinking}
      <div class="flex items-center gap-2 text-[var(--text-secondary)] text-sm py-2">
        <div class="flex gap-1">
          <span class="w-1.5 h-1.5 bg-[var(--accent)] rounded-full animate-bounce" style="animation-delay: 0ms"></span>
          <span class="w-1.5 h-1.5 bg-[var(--accent)] rounded-full animate-bounce" style="animation-delay: 150ms"></span>
          <span class="w-1.5 h-1.5 bg-[var(--accent)] rounded-full animate-bounce" style="animation-delay: 300ms"></span>
        </div>
        <span>Thinking...</span>
      </div>
    {/if}
  </div>

  <!-- Input area (interactive mode only) -->
  {#if mode === 'interactive'}
    <div class="p-4 border-t border-[var(--border)] bg-[var(--bg-secondary)]">
      <div class="flex gap-3">
        <textarea
          bind:value={inputText}
          placeholder="Ask the security agent..."
          rows="1"
          class="flex-1 px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
            text-[var(--text-primary)] placeholder-[var(--text-secondary)]/50 resize-none
            focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]"
          onkeydown={handleKeydown}
          disabled={!isConnected}
        ></textarea>
        <button
          class="px-6 py-2.5 bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-[var(--bg-primary)]
            font-semibold rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          disabled={!inputText.trim() || !isConnected}
          onclick={sendMessage}
        >
          Send
        </button>
      </div>
    </div>
  {/if}
{/if}
