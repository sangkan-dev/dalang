<script lang="ts">
  import { subscribe, removeToast } from '../lib/toast.ts';
  import type { Toast } from '../lib/toast.ts';

  let toasts = $state<Toast[]>([]);

  $effect(() => {
    const unsub = subscribe((t) => { toasts = t; });
    return unsub;
  });

  const iconMap: Record<string, string> = {
    success: '✓',
    error: '✗',
    warning: '⚠',
    info: 'ℹ',
  };

  const colorMap: Record<string, string> = {
    success: 'bg-emerald-900/90 border-emerald-700/50 text-emerald-200',
    error: 'bg-red-900/90 border-red-700/50 text-red-200',
    warning: 'bg-amber-900/90 border-amber-700/50 text-amber-200',
    info: 'bg-sky-900/90 border-sky-700/50 text-sky-200',
  };
</script>

{#if toasts.length > 0}
  <div class="fixed top-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
    {#each toasts as toast (toast.id)}
      <div
        class="flex items-start gap-2 px-4 py-3 rounded-lg border backdrop-blur-sm shadow-lg
          transition-all duration-300 animate-slide-in {colorMap[toast.type] ?? colorMap.info}"
        role="alert"
      >
        <span class="text-sm font-bold mt-0.5 shrink-0">{iconMap[toast.type] ?? 'ℹ'}</span>
        <span class="text-sm flex-1">{toast.message}</span>
        <button
          class="shrink-0 opacity-60 hover:opacity-100 text-sm ml-2"
          onclick={() => removeToast(toast.id)}
          aria-label="Dismiss"
        >×</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  @keyframes slide-in {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
  .animate-slide-in {
    animation: slide-in 0.3s ease-out;
  }
</style>
