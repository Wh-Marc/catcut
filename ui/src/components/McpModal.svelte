<script>
  import { X, Copy, Check, Terminal, Cpu } from 'lucide-svelte';

  export let onClose;

  let copiedTab = '';

  const claudeConfig = JSON.stringify({
    "mcpServers": {
      "catcut": {
        "command": "/home/marc/workspace/personal/catcut/target/debug/catcut-mcp",
        "args": ["--mode", "stdio", "--project", "/home/marc/workspace/personal/catcut/catcut_project.json"]
      }
    }
  }, null, 2);

  const cursorConfig = JSON.stringify({
    "mcpServers": {
      "catcut": {
        "url": "http://localhost:8765/sse"
      }
    }
  }, null, 2);

  function copyToClipboard(text, tab) {
    navigator.clipboard.writeText(text);
    copiedTab = tab;
    setTimeout(() => copiedTab = '', 2000);
  }
</script>

<div class="modal-backdrop" on:click={onClose}>
  <div class="modal-card" on:click|stopPropagation>
    <div class="modal-header">
      <div class="modal-title">
        <Cpu size={18} color="var(--accent)" />
        <span>Connect AI Chat via MCP (Model Context Protocol)</span>
      </div>
      <button class="close-btn" on:click={onClose}>
        <X size={16} />
      </button>
    </div>

    <div class="modal-body">
      <p class="desc">
        CatCut exposes 17 native video editing tools via MCP. You can connect Claude Desktop, Cursor, or omp to edit videos directly via chat conversation!
      </p>

      <div class="config-section">
        <div class="config-header">
          <span class="config-name">1. Claude Desktop Configuration (`claude_desktop_config.json`)</span>
          <button class="copy-btn" on:click={() => copyToClipboard(claudeConfig, 'claude')}>
            {#if copiedTab === 'claude'}
              <Check size={14} color="#00e676" />
              <span>Copied!</span>
            {:else}
              <Copy size={14} />
              <span>Copy</span>
            {/if}
          </button>
        </div>
        <pre class="code-block"><code>{claudeConfig}</code></pre>
      </div>

      <div class="config-section">
        <div class="config-header">
          <span class="config-name">2. Cursor / SSE Remote Configuration (`.cursor/mcp.json`)</span>
          <button class="copy-btn" on:click={() => copyToClipboard(cursorConfig, 'cursor')}>
            {#if copiedTab === 'cursor'}
              <Check size={14} color="#00e676" />
              <span>Copied!</span>
            {:else}
              <Copy size={14} />
              <span>Copy</span>
            {/if}
          </button>
        </div>
        <pre class="code-block"><code>{cursorConfig}</code></pre>
      </div>

      <div class="tips-box">
        <h4>💡 Example chat prompts to try:</h4>
        <ul>
          <li><em>"Import /path/to/myvideo.mp4 and add it to the timeline"</em></li>
          <li><em>"Cut out 5 seconds from 00:10 to 00:15 and ripple delete the gap"</em></li>
          <li><em>"Add title 'My Awesome Video' at 00:02 for 4 seconds in yellow"</em></li>
          <li><em>"Speed up clip 2 to 2x and inspect the frame at 00:08"</em></li>
        </ul>
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
    width: 620px;
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
  .close-btn:hover {
    color: var(--text-main);
  }
  .modal-body {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-height: 80vh;
    overflow-y: auto;
  }
  .desc {
    font-size: 13px;
    line-height: 1.5;
    color: var(--text-muted);
  }
  .config-section {
    background: var(--bg-base);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    overflow: hidden;
  }
  .config-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-color);
  }
  .config-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-main);
  }
  .copy-btn {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    color: var(--text-main);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    display: flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
  }
  .copy-btn:hover {
    background: var(--bg-hover);
  }
  .code-block {
    padding: 12px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
    color: #61afef;
    overflow-x: auto;
  }
  .tips-box {
    background: rgba(0, 210, 180, 0.05);
    border: 1px solid rgba(0, 210, 180, 0.2);
    border-radius: 6px;
    padding: 12px;
  }
  .tips-box h4 {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
    margin-bottom: 8px;
  }
  .tips-box ul {
    list-style-type: none;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .tips-box li {
    font-size: 12px;
    color: var(--text-main);
  }
</style>
