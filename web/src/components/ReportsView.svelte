<script lang="ts">
  import { api } from '../lib/api.ts';
  import { toast } from '../lib/toast.ts';
  import type { ReportEntry } from '../lib/types';

  let reports = $state<ReportEntry[]>([]);
  let selectedReport = $state<string | null>(null);
  let reportHtml = $state('');
  let loading = $state(true);
  let loadingReport = $state(false);
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
    loadingReport = true;
    try {
      const res = await fetch(`/api/reports/${filename}?format=html`);
      reportHtml = await res.text();
      selectedReport = filename;
    } catch (e) {
      toast.error(`Failed to load report: ${(e as Error).message}`);
    } finally {
      loadingReport = false;
    }
  }

  async function downloadMarkdown(): Promise<void> {
    if (!selectedReport) return;
    try {
      const res = await fetch(`/api/reports/${selectedReport}`);
      const text = await res.text();
      const blob = new Blob([text], { type: 'text/markdown' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = selectedReport;
      a.click();
      URL.revokeObjectURL(url);
      toast.success('Downloaded markdown report');
    } catch (e) {
      toast.error(`Download failed: ${(e as Error).message}`);
    }
  }

  function downloadHtml(): void {
    if (!reportHtml || !selectedReport) return;
    const fullHtml = `<!DOCTYPE html><html><head><meta charset="utf-8"><title>${selectedReport}</title><style>body{font-family:system-ui;max-width:900px;margin:0 auto;padding:2rem;line-height:1.6}pre{background:#f5f5f5;padding:1rem;border-radius:8px;overflow-x:auto}code{font-family:monospace}</style></head><body>${reportHtml}</body></html>`;
    const blob = new Blob([fullHtml], { type: 'text/html' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = selectedReport.replace(/\.md$/, '.html');
    a.click();
    URL.revokeObjectURL(url);
    toast.success('Downloaded HTML report');
  }

  function printReport(): void {
    window.print();
  }

  loadReports();
</script>

<div class="flex h-full">
  <!-- Report list -->
  <div class="w-72 border-r border-[var(--border)] bg-[var(--bg-secondary)] overflow-y-auto no-print">
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
            <div class="text-xs opacity-70">{(report.size / 1024).toFixed(1)} KB</div>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Report viewer -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if loadingReport}
      <div class="flex items-center justify-center h-full">
        <div class="flex items-center gap-2 text-[var(--text-secondary)]">
          <span class="animate-spin text-lg">↻</span>
          <span>Loading report...</span>
        </div>
      </div>
    {:else if reportHtml}
      <div class="max-w-4xl mx-auto">
        <!-- Toolbar -->
        <div class="flex items-center gap-2 mb-4 no-print">
          <button
            class="text-xs px-3 py-1.5 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border)]
              text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            onclick={downloadMarkdown}
          >📥 Markdown</button>
          <button
            class="text-xs px-3 py-1.5 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border)]
              text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            onclick={downloadHtml}
          >📥 HTML</button>
          <button
            class="text-xs px-3 py-1.5 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border)]
              text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            onclick={printReport}
          >🖨️ Print</button>
        </div>
        <div class="markdown-content">
          {@html reportHtml}
        </div>
      </div>
    {:else}
      <div class="flex items-center justify-center h-full text-[var(--text-secondary)]">
        <p>Select a report to view</p>
      </div>
    {/if}
  </div>
</div>
