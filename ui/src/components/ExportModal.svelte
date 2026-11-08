<script>
  import { X, Download, CheckCircle, AlertCircle, Loader } from 'lucide-svelte';
  import { callTool } from '../lib/api.js';

  export let project;
  export let onClose;

  let outputPath = 'catcut_export.mp4';
  let resolution = '1080p'; // '1080p' | '720p' | '4k'
  let fps = 30;
  let crf = 23;
  let isExporting = false;
  let exportStatus = null; // null | 'success' | 'error'
  let statusMessage = '';

  async function handleExport() {
    isExporting = true;
    exportStatus = null;
    statusMessage = 'Rendering video with FFmpeg...';

    const width = resolution === '4k' ? 3840 : resolution === '720p' ? 1280 : 1920;
    const height = resolution === '4k' ? 2160 : resolution === '720p' ? 720 : 1080;

    try {
      const res = await callTool('export_project', {
        output_path: outputPath,
        width,
        height,
        fps,
        crf,
      });

      exportStatus = 'success';
      statusMessage = res.content?.[0]?.text || 'Export finished!';
    } catch (e) {
      exportStatus = 'error';
      statusMessage = e.message || 'Export failed';
    } finally {
      isExporting = false;
    }
  }
</script>

<div class="modal-backdrop" on:click={onClose}>
  <div class="modal-card" on:click|stopPropagation>
    <div class="modal-header">
      <div class="modal-title">
        <Download size={18} color="var(--accent)" />
        <span>Export Video (MP4)</span>
      </div>
      <button class="close-btn" on:click={onClose}>
        <X size={16} />
      </button>
    </div>

    <div class="modal-body">
      <div class="field">
        <label>Output File Path</label>
        <input type="text" bind:value={outputPath} placeholder="/path/to/output.mp4" />
      </div>

      <div class="field-row">
        <div class="field">
          <label>Resolution</label>
          <select bind:value={resolution}>
            <option value="1080p">1080p Full HD (1920x1080)</option>
            <option value="720p">720p HD (1280x720)</option>
            <option value="4k">4K Ultra HD (3840x2160)</option>
          </select>
        </div>

        <div class="field">
          <label>Framerate</label>
          <select bind:value={fps}>
            <option value={30}>30 FPS</option>
            <option value={60}>60 FPS</option>
            <option value={24}>24 FPS (Cinematic)</option>
          </select>
        </div>
      </div>

      <div class="field">
        <label>Quality (CRF: {crf})</label>
        <div class="quality-picker">
          <button class="q-btn" class:active={crf === 18} on:click={() => crf = 18}>High (CRF 18)</button>
          <button class="q-btn" class:active={crf === 23} on:click={() => crf = 23}>Standard (CRF 23)</button>
          <button class="q-btn" class:active={crf === 28} on:click={() => crf = 28}>Compact (CRF 28)</button>
        </div>
      </div>

      {#if exportStatus === 'success'}
        <div class="status-box success">
          <CheckCircle size={18} />
          <span>{statusMessage}</span>
        </div>
      {:else if exportStatus === 'error'}
        <div class="status-box error">
          <AlertCircle size={18} />
          <span>{statusMessage}</span>
        </div>
      {/if}

      <div class="actions">
        <button class="cancel-btn" on:click={onClose} disabled={isExporting}>Cancel</button>
        <button class="export-btn" on:click={handleExport} disabled={isExporting || !outputPath}>
          {#if isExporting}
            <Loader size={16} class="spin" />
            <span>Rendering...</span>
          {:else}
            <Download size={16} />
            <span>Start Export</span>
          {/if}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal-card {
    width: 480px;
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.7);
    overflow: hidden;
  }
  .modal-header {
    height: 48px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
  }
  .modal-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-main);
  }
  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
  }
  .modal-body {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field-row {
    display: flex;
    gap: 12px;
  }
  .field-row .field {
    flex: 1;
  }
  label {
    font-size: 12px;
    color: var(--text-muted);
  }
  input, select {
    background: var(--bg-base);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-main);
    padding: 8px 10px;
    font-size: 13px;
    outline: none;
  }
  input:focus, select:focus {
    border-color: var(--accent);
  }
  .quality-picker {
    display: flex;
    gap: 8px;
  }
  .q-btn {
    flex: 1;
    background: var(--bg-panel);
    border: 1px solid var(--border-color);
    color: var(--text-muted);
    padding: 8px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
  }
  .q-btn.active {
    background: var(--accent);
    color: #000;
    font-weight: 600;
    border-color: var(--accent);
  }
  .status-box {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: 6px;
    font-size: 12px;
  }
  .status-box.success {
    background: rgba(0, 230, 118, 0.1);
    color: #00e676;
    border: 1px solid rgba(0, 230, 118, 0.3);
  }
  .status-box.error {
    background: rgba(229, 62, 62, 0.1);
    color: #fc8181;
    border: 1px solid rgba(229, 62, 62, 0.3);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 8px;
  }
  .cancel-btn {
    background: var(--bg-panel);
    border: 1px solid var(--border-color);
    color: var(--text-muted);
    padding: 8px 14px;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
  }
  .export-btn {
    background: var(--accent);
    border: none;
    color: #000;
    font-weight: 600;
    padding: 8px 16px;
    border-radius: 4px;
    font-size: 13px;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }
  .export-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  :global(.spin) {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
