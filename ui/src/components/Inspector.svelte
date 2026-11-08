<script>
  import { callTool } from '../lib/api.js';
  import { Sliders, Volume2, Move, Gauge, Type, Scissors, Trash2 } from 'lucide-svelte';

  export let project;
  export let selectedClipId = null;
  export let onRefresh;
  export let currentTimeSec = 0;

  $: selectedClipData = findClip(selectedClipId);

  function findClip(id) {
    if (!id || !project?.tracks) return null;
    for (const track of project.tracks) {
      const seg = track.segments.find(s => s.id == id);
      if (seg) {
        return { segment: seg, track };
      }
    }
    return null;
  }

  async function updateSpeed(newSpeed) {
    if (!selectedClipId) return;
    try {
      await callTool('set_speed', { clip_id: selectedClipId, speed: parseFloat(newSpeed) });
      onRefresh();
    } catch (e) {
      alert("Speed error: " + e.message);
    }
  }

  async function updateVolume(newVol) {
    if (!selectedClipId) return;
    try {
      await callTool('adjust_audio', { clip_id: selectedClipId, volume: parseFloat(newVol) });
      onRefresh();
    } catch (e) {
      alert("Audio error: " + e.message);
    }
  }

  async function updateTransform(key, val) {
    if (!selectedClipId) return;
    const current = selectedClipData?.segment?.transform || {};
    const args = {
      clip_id: selectedClipId,
      scale_x: key === 'scale_x' ? parseFloat(val) : current.scale_x,
      scale_y: key === 'scale_y' ? parseFloat(val) : current.scale_y,
      position_x: key === 'pos_x' ? parseFloat(val) : current.position_x,
      position_y: key === 'pos_y' ? parseFloat(val) : current.position_y,
      rotation: key === 'rotation' ? parseFloat(val) : current.rotation,
      opacity: key === 'opacity' ? parseFloat(val) : current.opacity,
    };
    try {
      await callTool('set_transform', args);
      onRefresh();
    } catch (e) {
      console.error(e);
    }
  }

  async function handleSplitHere() {
    if (!selectedClipId) return;
    try {
      await callTool('split_clip', { clip_id: selectedClipId, split_time_sec: currentTimeSec });
      onRefresh();
    } catch (e) {
      alert("Split error: " + e.message);
    }
  }

  async function handleDelete(ripple = false) {
    if (!selectedClipId) return;
    try {
      await callTool('remove_clip', { clip_id: selectedClipId, ripple });
      selectedClipId = null;
      onRefresh();
    } catch (e) {
      alert("Delete error: " + e.message);
    }
  }
</script>

<div class="inspector">
  <div class="panel-header">
    <Sliders size={16} />
    <span>Inspector</span>
  </div>

  <div class="content">
    {#if !selectedClipData}
      <div class="section">
        <h4 class="section-title">Project Settings</h4>
        <div class="field">
          <label>Project Name</label>
          <input type="text" value={project?.name || ''} readonly class="readonly-input" />
        </div>
        <div class="field-row">
          <div class="field">
            <label>Width</label>
            <input type="text" value={project?.width || 1920} readonly class="readonly-input" />
          </div>
          <div class="field">
            <label>Height</label>
            <input type="text" value={project?.height || 1080} readonly class="readonly-input" />
          </div>
        </div>
        <div class="field">
          <label>Framerate</label>
          <input type="text" value={`${project?.fps || 30} FPS`} readonly class="readonly-input" />
        </div>
      </div>
      <div class="empty-hint">
        <span>Click on any clip on the timeline to inspect and edit its properties.</span>
      </div>
    {:else}
      {@const seg = selectedClipData.segment}
      {@const track = selectedClipData.track}

      <div class="section actions-bar">
        <button class="btn btn-split" on:click={handleSplitHere} title="Split at playhead">
          <Scissors size={14} />
          <span>Split</span>
        </button>
        <button class="btn btn-delete" on:click={() => handleDelete(false)} title="Delete clip">
          <Trash2 size={14} />
          <span>Delete</span>
        </button>
        <button class="btn btn-ripple" on:click={() => handleDelete(true)} title="Ripple delete">
          <span>Ripple Del</span>
        </button>
      </div>

      <!-- Speed Control -->
      <div class="section">
        <h4 class="section-title"><Gauge size={14} /> Speed ({seg.speed}x)</h4>
        <div class="speed-presets">
          {#each [0.5, 1.0, 1.5, 2.0, 4.0] as s}
            <button
              class="preset-btn"
              class:active={Math.abs(seg.speed - s) < 0.01}
              on:click={() => updateSpeed(s)}
            >
              {s}x
            </button>
          {/each}
        </div>
      </div>

      <!-- Transform Controls -->
      {#if track.track_type === 'video'}
        <div class="section">
          <h4 class="section-title"><Move size={14} /> Video Transform</h4>
          
          <div class="field">
            <div class="field-label-row">
              <label>Scale</label>
              <span>{Math.round((seg.transform?.scale_x || 1) * 100)}%</span>
            </div>
            <input
              type="range"
              min="0.1"
              max="3.0"
              step="0.05"
              value={seg.transform?.scale_x || 1}
              on:input={(e) => updateTransform('scale_x', e.target.value)}
            />
          </div>

          <div class="field">
            <div class="field-label-row">
              <label>Opacity</label>
              <span>{Math.round((seg.transform?.opacity ?? 1) * 100)}%</span>
            </div>
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={seg.transform?.opacity ?? 1}
              on:input={(e) => updateTransform('opacity', e.target.value)}
            />
          </div>

          <div class="field">
            <div class="field-label-row">
              <label>Rotation</label>
              <span>{seg.transform?.rotation || 0}°</span>
            </div>
            <input
              type="range"
              min="-180"
              max="180"
              step="1"
              value={seg.transform?.rotation || 0}
              on:input={(e) => updateTransform('rotation', e.target.value)}
            />
          </div>
        </div>
      {/if}

      <!-- Audio Controls -->
      {#if track.track_type === 'video' || track.track_type === 'audio'}
        <div class="section">
          <h4 class="section-title"><Volume2 size={14} /> Audio & Volume</h4>
          <div class="field">
            <div class="field-label-row">
              <label>Volume</label>
              <span>{Math.round((seg.audio?.volume || 1) * 100)}%</span>
            </div>
            <input
              type="range"
              min="0"
              max="2"
              step="0.05"
              value={seg.audio?.volume || 1}
              on:input={(e) => updateVolume(e.target.value)}
            />
          </div>
        </div>
      {/if}

      <!-- Text Content Control -->
      {#if track.track_type === 'text'}
        <div class="section">
          <h4 class="section-title"><Type size={14} /> Text Content</h4>
          <div class="field">
            <label>Text Content</label>
            <input type="text" value={seg.text_content || ''} readonly class="readonly-input" />
          </div>
          <div class="field">
            <label>Font Color</label>
            <input type="color" value={seg.text_style?.color || '#FFFFFF'} disabled />
          </div>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .inspector {
    width: 290px;
    background: var(--bg-surface);
    border-left: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .panel-header {
    height: 38px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-main);
  }
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .section {
    background: var(--bg-panel);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .section-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-main);
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 4px;
  }
  .actions-bar {
    flex-direction: row;
    gap: 6px;
    padding: 8px;
  }
  .btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    font-size: 11px;
    font-weight: 600;
    padding: 6px 8px;
    border-radius: 4px;
    border: none;
    cursor: pointer;
  }
  .btn-split {
    background: var(--accent);
    color: #000;
  }
  .btn-delete {
    background: #e53e3e;
    color: #fff;
  }
  .btn-ripple {
    background: #d69e2e;
    color: #000;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .field-row {
    display: flex;
    gap: 8px;
  }
  .field-row .field {
    flex: 1;
  }
  .field-label-row {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--text-muted);
  }
  label {
    font-size: 11px;
    color: var(--text-muted);
  }
  input[type="range"] {
    accent-color: var(--accent);
    cursor: pointer;
  }
  .readonly-input {
    background: var(--bg-base);
    border: 1px solid var(--border-color);
    color: var(--text-main);
    border-radius: 4px;
    padding: 6px 8px;
    font-size: 12px;
    outline: none;
  }
  .speed-presets {
    display: flex;
    gap: 4px;
  }
  .preset-btn {
    flex: 1;
    background: var(--bg-base);
    border: 1px solid var(--border-color);
    color: var(--text-muted);
    padding: 5px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
  }
  .preset-btn:hover {
    color: var(--text-main);
    border-color: var(--border-active);
  }
  .preset-btn.active {
    background: var(--accent);
    color: #000;
    font-weight: 700;
    border-color: var(--accent);
  }
  .empty-hint {
    padding: 16px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.5;
  }
</style>
