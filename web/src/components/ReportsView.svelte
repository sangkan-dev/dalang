<script lang="ts">
  import { api } from '../lib/api.ts';
  import type { ReportEntry } from '../lib/types';

  let reports = $state<ReportEntry[]>([]);
  let selectedReport = $state<string | null>(null);
  let reportHtml = $state('');
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function loadReports(): Promise<void> {
    loading = true;
    try {
      reports = await api.listReports();
      error = null;
    } catch (e) {
      error = (e as Error).message;
    } finally {
      loading = false;
    }
  }

  async function viewReport(filename: string): Promise<void> {
    try {
      const res = await fetch(`/api/reports/${filename}?format=html`);
      reportHtml = await res.text();
      selectedReport = filename;
    } catch (e) {
      error = (e as Error).message;
    }
  }

  loadReports();
</script>

<div class="flex h-full">
  <!-- Report list -->
  <div class="w-72 border-r border-[var(--border)] bg-[var(--bg-secondary)] overflow-y-auto">
    <div class="p-4 border-b border-[var(--border)]">
      <h2 class="text-lg font-bold">📋 Reports</h2>
      <p class="text-xs text-[var(--text-secondary)] mt-1">{reports.length} report(s)</p>
    </div>

    {#if loading}
      <div class="p-4 text-[var(--text-secondary)] text-sm">Loading...</div>
    {:else if error}
      <div class="p-4 text-[var(--danger)] text-sm">{error}</div>
    {:else if reports.length === 0}
      <div class="p-4 text-[var(--text-secondary)] text-sm">No reports yet. Run a scan to generate reports.</div>
    {:else}
      <div class="p-2 space-y-1">
        {#each reports as report}
          <button
            class="w-full text-left px-3 py-2.5 rounded-lg transition-colors
              {selectedReport === report.filename
                ? 'bg-[var(--accent)]/10 text-[var(--accent)]'
                : 'text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)]'}"
            onclick={() => viewReport(report.filename)}
          >
            <div class="text-sm font-medium truncate">{report.filename}</div>
            <div class="text-xs opacity-70">{report.size} bytes</div>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Report viewer -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if reportHtml}
      <div class="max-w-4xl mx-auto">
        {@html reportHtml}
      </div>
    {:else}
      <div class="flex items-center justify-center h-full text-[var(--text-secondary)]">
        <p>Select a report to view</p>
      </div>
    {/if}
  </div>
</div>
