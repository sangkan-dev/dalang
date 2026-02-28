<script lang="ts">
  let {
    value = $bindable(''),
    disabled = false,
    onSend,
  }: {
    value: string;
    disabled?: boolean;
    onSend: () => void;
  } = $props();

  let textareaEl = $state<HTMLTextAreaElement | null>(null);

  // Auto-resize textarea based on content (max 6 rows ≈ 144px)
  $effect(() => {
    // track value changes
    value;
    const el = textareaEl;
    if (el) {
      el.style.height = 'auto';
      el.style.height = `${Math.min(el.scrollHeight, 144)}px`;
    }
  });

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      onSend();
    }
  }
</script>

<div class="p-4 border-t border-[var(--border)] bg-[var(--bg-secondary)]">
  <div class="flex gap-3 items-end">
    <textarea
      bind:this={textareaEl}
      bind:value={value}
      placeholder="Ask the security agent..."
      rows="1"
      class="flex-1 px-4 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg
        text-[var(--text-primary)] placeholder-[var(--text-secondary)]/50 resize-none
        focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]
        max-h-36 overflow-y-auto"
      onkeydown={handleKeydown}
      {disabled}
    ></textarea>
    <button
      class="px-6 py-2.5 bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-[var(--bg-primary)]
        font-semibold rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed shrink-0"
      disabled={!value.trim() || disabled}
      onclick={onSend}
    >
      Send
    </button>
  </div>
</div>
