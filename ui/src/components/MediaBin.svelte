<script>
  import { callTool } from '../lib/api.js';
  import { Plus, Video, Music, Image as ImageIcon, Type, Clock, Trash2, Bot, Zap, Film } from 'lucide-svelte';

  export let project;
  export let onRefresh;
  export let onAddClip;
  export let aiLogs = [];

  let activeTab = 'media'; // 'media' | 'ai_log'
  let importPath = '';
  let isImporting = false;

  async function handleImport() {
    if (!importPath) return;
    isImporting = true;
    try {
      await callTool('import_media', { file_path: importPath });
      importPath = '';
      onRefresh();
    } catch (e) {
      alert('Import failed: ' + e.message);
    } finally {
      isImporting = false;
    }
  }

  function formatDuration(us) {
    const sec = (us || 0) / 1000000;
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }

  let isAnalyzing = false;

  async function handleAutoRoughCut(materialId) {
    if (!confirm("Tự động cắt bỏ khoảng lặng / ngập ngừng và dồn các đoạn nói khít lại trên timeline?")) {
      return;
    }
    isAnalyzing = true;
    try {
      const res = await callTool('auto_rough_cut', {
        material_id: materialId,
        min_silence_sec: 0.5,
      });
      alert(res.content?.[0]?.text || "Đã hoàn thành Auto Rough Cut!");
      onRefresh();
    } catch (e) {
      alert("Lỗi Auto Rough Cut: " + e.message);
    } finally {
      isAnalyzing = false;
    }
  }

  async function handleDetectScenes(materialId) {
    isAnalyzing = true;
    try {
      const res = await callTool('detect_scenes', {
        material_id: materialId,
        threshold: 0.2,
      });
      alert(res.content?.[0]?.text || "Đã quét xong các cảnh quay!");
    } catch (e) {
      alert("Lỗi quét cảnh: " + e.message);
    } finally {
      isAnalyzing = false;
    }
  }
</script>

<div class="media-bin">
  <div class="tabs">
    <button class="tab" class:active={activeTab === 'media'} on:click={() => (activeTab = 'media')}>
      <Video size={14} />
      <span>Media ({project?.materials?.length || 0})</span>
    </button>
    <button class="tab" class:active={activeTab === 'ai_log'} on:click={() => (activeTab = 'ai_log')}>
      <Bot size={14} />
      <span>AI Activity</span>
      {#if aiLogs.length > 0}
        <span class="badge">{aiLogs.length}</span>
      {/if}
    </button>
  </div>

  {#if activeTab === 'media'}
    <div class="import-bar">
      <input
        type="text"
        placeholder="Enter media path (e.g. /path/to/video.mp4)"
        bind:value={importPath}
        on:keydown={(e) => e.key === 'Enter' && handleImport()}
      />
      <button class="import-btn" on:click={handleImport} disabled={isImporting || !importPath}>
        <Plus size={16} />
      </button>
    </div>

    <div class="media-list">
      {#if !project?.materials || project.materials.length === 0}
        <div class="empty-state">
          <Video size={36} opacity={0.3} />
          <p>No media files imported</p>
          <span class="hint">Type a file path above or ask your AI chat to import via MCP!</span>
        </div>
      {:else}
        {#each project.materials as mat}
          <div class="media-card">
            <div class="media-thumb">
              {#if mat.material_type === 'video'}
                <Video size={20} />
              {:else if mat.material_type === 'audio'}
                <Music size={20} />
              {:else}
                <ImageIcon size={20} />
              {/if}
            </div>
            <div class="media-meta">
              <span class="media-name" title={mat.name}>{mat.name}</span>
              <div class="media-details">
                <span class="duration"><Clock size={11} /> {formatDuration(mat.duration)}</span>
                {#if mat.width > 0}
                  <span class="res">{mat.width}x{mat.height}</span>
                {/if}
              </div>
            </div>
            <div class="card-actions">
              <button class="action-btn smart-cut" on:click={() => handleAutoRoughCut(mat.id)} disabled={isAnalyzing} title="Auto Rough Cut: Tự động cắt bỏ khoảng lặng và dồn video">
                <Zap size={14} />
              </button>
              <button class="action-btn" on:click={() => handleDetectScenes(mat.id)} disabled={isAnalyzing} title="Quét cảnh quay">
                <Film size={14} />
              </button>
              <button class="action-btn add-btn" on:click={() => onAddClip(mat.id)} title="Thêm vào timeline">
                <Plus size={16} />
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  {:else}
    <div class="ai-log-list">
      {#if aiLogs.length === 0}
        <div class="empty-state">
          <Bot size={36} opacity={0.3} />
          <p>No AI commands executed yet</p>
          <span class="hint">Start chatting with Claude/Cursor/omp using MCP tools to see real-time log.</span>
        </div>
      {:else}
        {#each aiLogs as log}
          <div class="log-item">
            <span class="log-time">{log.time}</span>
            <div class="log-text">{log.message}</div>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .media-bin {
    width: 280px;
    background: var(--bg-surface);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .tabs {
    display: flex;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-panel);
  }
  .tab {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-muted);
    padding: 10px 8px;
    font-size: 12px;
    font-weight: 500;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: all 0.2s;
  }
  .tab:hover {
    color: var(--text-main);
  }
  .tab.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
    background: var(--bg-surface);
  }
  .badge {
    background: var(--accent);
    color: #000;
    font-size: 10px;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: 8px;
  }
  .import-bar {
    display: flex;
    padding: 10px;
    gap: 6px;
    border-bottom: 1px solid var(--border-color);
  }
  .import-bar input {
    flex: 1;
    background: var(--bg-base);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-main);
    padding: 6px 8px;
    font-size: 12px;
    outline: none;
  }
  .import-bar input:focus {
    border-color: var(--accent);
  }
  .import-btn {
    background: var(--bg-panel);
    border: 1px solid var(--border-color);
    color: var(--text-main);
    border-radius: 4px;
    width: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }
  .import-btn:hover:not(:disabled) {
    background: var(--accent);
    color: #000;
  }
  .import-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .media-list, .ai-log-list {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    text-align: center;
    gap: 8px;
    padding: 20px;
  }
  .empty-state p {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-main);
  }
  .hint {
    font-size: 11px;
    line-height: 1.4;
    color: var(--text-muted);
  }
  .media-card {
    display: flex;
    align-items: center;
    background: var(--bg-panel);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 8px;
    gap: 10px;
    transition: border-color 0.15s;
  }
  .media-card:hover {
    border-color: var(--border-active);
  }
  .media-thumb {
    width: 40px;
    height: 40px;
    background: var(--bg-base);
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--accent);
  }
  .media-meta {
    flex: 1;
    min-width: 0;
  }
  .media-name {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
  }
  .media-details {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 3px;
  }
  .duration {
    display: flex;
    align-items: center;
    gap: 3px;
  }
  .card-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .action-btn {
    background: var(--bg-base);
    border: 1px solid var(--border-color);
    color: var(--text-muted);
    border-radius: 4px;
    width: 26px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s;
  }
  .action-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-main);
    border-color: var(--border-active);
  }
  .action-btn.smart-cut:hover:not(:disabled) {
    background: #f6ad55;
    color: #000;
    border-color: #f6ad55;
  }
  .action-btn.add-btn:hover:not(:disabled) {
    background: var(--accent);
    color: #000;
    border-color: var(--accent);
  }
  .action-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
  .log-item {
    background: var(--bg-panel);
    border-left: 3px solid var(--accent);
    border-radius: 4px;
    padding: 8px 10px;
    font-size: 12px;
  }
  .log-time {
    font-size: 10px;
    color: var(--text-muted);
    display: block;
    margin-bottom: 2px;
    font-family: 'JetBrains Mono', monospace;
  }
  .log-text {
    color: var(--text-main);
    line-height: 1.4;
  }
</style>
