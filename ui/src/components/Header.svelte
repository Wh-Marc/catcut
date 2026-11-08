<script>
  import { callTool } from '../lib/api.js';
  import { Undo, Redo, Download, FolderOpen, Bot, Share2, Sparkles } from 'lucide-svelte';

  export let project;
  export let onRefresh;
  export let onShowMcpModal;
  export let onShowExportModal;

  async function handleUndo() {
    try {
      await callTool('undo');
      onRefresh();
    } catch (e) {
      console.error(e);
    }
  }

  async function handleRedo() {
    try {
      await callTool('redo');
      onRefresh();
    } catch (e) {
      console.error(e);
    }
  }

  async function handleCapcutImport() {
    const path = prompt("Enter path to CapCut draft_content.json file:");
    if (!path) return;
    try {
      await callTool('import_capcut_draft', { draft_path: path });
      onRefresh();
    } catch (e) {
      alert("Failed to import CapCut draft: " + e.message);
    }
  }

  async function handleCapcutExport() {
    const path = prompt("Enter output path for CapCut draft (e.g. /tmp/draft_content.json):");
    if (!path) return;
    try {
      await callTool('export_capcut_draft', { output_path: path });
      alert("Successfully exported to CapCut draft: " + path);
    } catch (e) {
      alert("Failed to export CapCut draft: " + e.message);
    }
  }
</script>

<header class="header">
  <div class="left-section">
    <div class="logo">
      <span class="logo-icon">🐱</span>
      <span class="logo-text">CatCut</span>
      <span class="tag">AI MCP</span>
    </div>
    <div class="project-info">
      <span class="project-name">{project?.name || 'Untitled Project'}</span>
      <span class="divider">•</span>
      <span class="meta">{project?.width || 1920}x{project?.height || 1080}</span>
      <span class="divider">•</span>
      <span class="meta">{project?.fps || 30} FPS</span>
    </div>
  </div>

  <div class="center-section">
    <button class="mcp-badge" on:click={onShowMcpModal} title="Click to view MCP Configuration for Claude / Cursor / omp">
      <span class="dot"></span>
      <Bot size={15} />
      <span>MCP Server Active</span>
      <span class="port">:8765</span>
    </button>
  </div>

  <div class="right-section">
    <div class="btn-group">
      <button class="icon-btn" on:click={handleUndo} title="Undo (Ctrl+Z)">
        <Undo size={16} />
      </button>
      <button class="icon-btn" on:click={handleRedo} title="Redo (Ctrl+Y)">
        <Redo size={16} />
      </button>
    </div>

    <div class="dropdown-wrapper">
      <button class="btn btn-secondary" on:click={handleCapcutImport} title="Import CapCut draft">
        <FolderOpen size={15} />
        <span>Import CapCut</span>
      </button>
      <button class="btn btn-secondary" on:click={handleCapcutExport} title="Export to CapCut draft">
        <Share2 size={15} />
        <span>Export CapCut</span>
      </button>
    </div>

    <button class="btn btn-primary" on:click={onShowExportModal}>
      <Download size={16} />
      <span>Export MP4</span>
    </button>
  </div>
</header>

<style>
  .header {
    height: 48px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    z-index: 50;
  }
  .left-section, .right-section, .center-section {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .logo {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 700;
    font-size: 16px;
    letter-spacing: -0.5px;
  }
  .logo-icon {
    font-size: 20px;
  }
  .logo-text {
    background: linear-gradient(135deg, #00d2b4, #00a8ff);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  .tag {
    font-size: 10px;
    padding: 2px 6px;
    background: rgba(0, 210, 180, 0.15);
    color: var(--accent);
    border-radius: 4px;
    font-weight: 600;
  }
  .project-info {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text-muted);
    margin-left: 8px;
    padding-left: 12px;
    border-left: 1px solid var(--border-color);
  }
  .project-name {
    font-weight: 600;
    color: var(--text-main);
  }
  .divider {
    color: var(--border-active);
  }
  .mcp-badge {
    background: rgba(0, 210, 180, 0.1);
    border: 1px solid rgba(0, 210, 180, 0.3);
    color: var(--accent);
    padding: 4px 10px;
    border-radius: 16px;
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }
  .mcp-badge:hover {
    background: rgba(0, 210, 180, 0.2);
    border-color: var(--accent);
  }
  .dot {
    width: 7px;
    height: 7px;
    background: #00e676;
    border-radius: 50%;
    box-shadow: 0 0 8px #00e676;
    animation: pulse 2s infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(0.9); }
  }
  .port {
    font-family: 'JetBrains Mono', monospace;
    opacity: 0.8;
  }
  .btn-group {
    display: flex;
    background: var(--bg-panel);
    border-radius: 6px;
    padding: 2px;
    border: 1px solid var(--border-color);
  }
  .icon-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    padding: 6px 10px;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-main);
  }
  .dropdown-wrapper {
    display: flex;
    gap: 6px;
  }
  .btn {
    border: none;
    border-radius: 6px;
    padding: 6px 12px;
    font-size: 13px;
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    transition: background 0.15s;
  }
  .btn-secondary {
    background: var(--bg-panel);
    color: var(--text-main);
    border: 1px solid var(--border-color);
  }
  .btn-secondary:hover {
    background: var(--bg-hover);
  }
  .btn-primary {
    background: var(--accent);
    color: #0b1016;
    font-weight: 600;
  }
  .btn-primary:hover {
    background: var(--accent-hover);
  }
</style>
