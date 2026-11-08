<script>
  import { ZoomIn, ZoomOut, Scissors, Trash2, Plus, Volume2, Eye, Lock } from 'lucide-svelte';
  import { callTool } from '../lib/api.js';

  export let project;
  export let currentTimeSec = 0;
  export let selectedClipId = null;
  export let onSeek;
  export let onRefresh;
  export let onSelectClip;

  let zoomPxPerSec = 50; // default 50px = 1s
  let isDraggingPlayhead = false;
  let timelineContainer;

  $: totalDurationSec = Math.max(30, (project?.duration || 0) / 1000000);
  $: timelineWidthPx = Math.max(800, totalDurationSec * zoomPxPerSec);
  $: playheadLeftPx = currentTimeSec * zoomPxPerSec;

  function handleRulerClick(e) {
    const rect = e.currentTarget.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const newTime = Math.max(0, clickX / zoomPxPerSec);
    onSeek(newTime);
  }

  function handleRulerMouseDown(e) {
    isDraggingPlayhead = true;
    handleRulerClick(e);

    const onMouseMove = (ev) => {
      if (!isDraggingPlayhead) return;
      const rect = timelineContainer.getBoundingClientRect();
      const clickX = ev.clientX - rect.left - 180; // 180px track header offset
      const newTime = Math.max(0, clickX / zoomPxPerSec);
      onSeek(newTime);
    };

    const onMouseUp = () => {
      isDraggingPlayhead = false;
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    };

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  function handleClipClick(e, clipId) {
    e.stopPropagation();
    onSelectClip(clipId);
  }

  async function handleSplitClip() {
    if (!selectedClipId) return;
    try {
      await callTool('split_clip', {
        clip_id: selectedClipId,
        split_time_sec: currentTimeSec,
      });
      onRefresh();
    } catch (e) {
      alert("Split error: " + e.message);
    }
  }

  async function handleDeleteClip(ripple = false) {
    if (!selectedClipId) return;
    try {
      await callTool('remove_clip', {
        clip_id: selectedClipId,
        ripple,
      });
      selectedClipId = null;
      onRefresh();
    } catch (e) {
      alert("Delete error: " + e.message);
    }
  }

  function formatTime(sec) {
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }

  // Calculate ticks for ruler
  $: ticks = Array.from({ length: Math.ceil(totalDurationSec) + 1 }, (_, i) => i);
</script>

<div class="timeline-panel" bind:this={timelineContainer}>
  <!-- Timeline Toolbar -->
  <div class="timeline-toolbar">
    <div class="toolbar-left">
      <button class="tool-action-btn" on:click={handleSplitClip} disabled={!selectedClipId} title="Split Selected Clip at Playhead (Ctrl+B)">
        <Scissors size={14} />
        <span>Split</span>
      </button>
      <button class="tool-action-btn" on:click={() => handleDeleteClip(false)} disabled={!selectedClipId} title="Delete Selected Clip (Del)">
        <Trash2 size={14} />
        <span>Delete</span>
      </button>
      <button class="tool-action-btn" on:click={() => handleDeleteClip(true)} disabled={!selectedClipId} title="Ripple Delete Selected Clip">
        <span>Ripple Del</span>
      </button>
    </div>

    <div class="toolbar-right">
      <button class="zoom-btn" on:click={() => (zoomPxPerSec = Math.max(10, zoomPxPerSec - 15))} title="Zoom Out">
        <ZoomOut size={15} />
      </button>
      <span class="zoom-label">{Math.round((zoomPxPerSec / 50) * 100)}%</span>
      <button class="zoom-btn" on:click={() => (zoomPxPerSec = Math.min(200, zoomPxPerSec + 15))} title="Zoom In">
        <ZoomIn size={15} />
      </button>
    </div>
  </div>

  <div class="timeline-body">
    <!-- Track Headers Column -->
    <div class="track-headers">
      <div class="ruler-header">Tracks</div>
      {#if project?.tracks}
        {#each project.tracks as track}
          <div class="track-header-item">
            <span class="track-title">{track.name}</span>
            <div class="track-controls">
              <Eye size={13} opacity={0.6} />
              <Volume2 size={13} opacity={0.6} />
              <Lock size={13} opacity={0.6} />
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <!-- Scrollable Tracks & Ruler Area -->
    <div class="track-content-scroll" on:mousedown={handleRulerMouseDown}>
      <div class="track-content-container" style="width: {timelineWidthPx}px;">
        <!-- Time Ruler -->
        <div class="time-ruler" on:click={handleRulerClick}>
          {#each ticks as sec}
            {#if (zoomPxPerSec >= 40 && sec % 5 === 0) || (zoomPxPerSec < 40 && sec % 10 === 0) || sec === 0}
              <div class="ruler-mark major" style="left: {sec * zoomPxPerSec}px;">
                <span class="ruler-time">{formatTime(sec)}</span>
              </div>
            {:else if zoomPxPerSec >= 50}
              <div class="ruler-mark minor" style="left: {sec * zoomPxPerSec}px;"></div>
            {/if}
          {/each}
        </div>

        <!-- Tracks Canvas -->
        <div class="tracks-list">
          {#if project?.tracks}
            {#each project.tracks as track}
              <div class="track-lane" class:track-video={track.track_type === 'video'} class:track-audio={track.track_type === 'audio'} class:track-text={track.track_type === 'text'}>
                {#each track.segments as seg}
                  {@const startSec = seg.target_timerange.start / 1000000}
                  {@const durSec = seg.target_timerange.duration / 1000000}
                  {@const leftPx = startSec * zoomPxPerSec}
                  {@const widthPx = Math.max(15, durSec * zoomPxPerSec)}
                  {@const label = seg.text_content || project?.materials?.find(m => m.id === seg.material_id)?.name || 'Clip'}

                  <div
                    class="clip"
                    class:selected={selectedClipId === seg.id}
                    class:clip-video={track.track_type === 'video'}
                    class:clip-audio={track.track_type === 'audio'}
                    class:clip-text={track.track_type === 'text'}
                    style="left: {leftPx}px; width: {widthPx}px;"
                    on:click={(e) => handleClipClick(e, seg.id)}
                    title="{label} ({durSec.toFixed(2)}s, speed: {seg.speed}x)"
                  >
                    <span class="clip-name">{label}</span>
                    <span class="clip-dur">{durSec.toFixed(1)}s</span>
                  </div>
                {/each}
              </div>
            {/each}
          {/if}
        </div>

        <!-- Vertical Red Playhead Line -->
        <div class="playhead" style="left: {playheadLeftPx}px;">
          <div class="playhead-handle"></div>
          <div class="playhead-line"></div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .timeline-panel {
    height: 260px;
    background: var(--bg-surface);
    border-top: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .timeline-toolbar {
    height: 36px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
  }
  .toolbar-left, .toolbar-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .tool-action-btn {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    color: var(--text-main);
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 11px;
    display: flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
  }
  .tool-action-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    border-color: var(--border-active);
  }
  .tool-action-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .zoom-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .zoom-btn:hover {
    color: var(--text-main);
  }
  .zoom-label {
    font-size: 11px;
    font-family: 'JetBrains Mono', monospace;
    color: var(--text-muted);
  }
  .timeline-body {
    flex: 1;
    display: flex;
    overflow: hidden;
    position: relative;
  }
  .track-headers {
    width: 180px;
    background: var(--bg-surface);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    z-index: 10;
  }
  .ruler-header {
    height: 28px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-color);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    padding: 0 10px;
  }
  .track-header-item {
    height: 48px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 10px;
    background: var(--bg-surface);
  }
  .track-title {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-main);
  }
  .track-controls {
    display: flex;
    gap: 6px;
    color: var(--text-muted);
  }
  .track-content-scroll {
    flex: 1;
    overflow-x: auto;
    overflow-y: hidden;
    background: var(--bg-base);
    position: relative;
  }
  .track-content-container {
    height: 100%;
    position: relative;
  }
  .time-ruler {
    height: 28px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-color);
    position: relative;
    cursor: pointer;
  }
  .ruler-mark {
    position: absolute;
    bottom: 0;
    width: 1px;
    background: rgba(255, 255, 255, 0.2);
  }
  .ruler-mark.major {
    height: 14px;
    background: rgba(255, 255, 255, 0.4);
  }
  .ruler-mark.minor {
    height: 6px;
  }
  .ruler-time {
    position: absolute;
    bottom: 12px;
    left: 4px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
  }
  .tracks-list {
    display: flex;
    flex-direction: column;
  }
  .track-lane {
    height: 48px;
    border-bottom: 1px solid var(--border-color);
    position: relative;
    background: rgba(255, 255, 255, 0.01);
  }
  .clip {
    position: absolute;
    top: 4px;
    bottom: 4px;
    border-radius: 4px;
    padding: 4px 8px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    cursor: pointer;
    overflow: hidden;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    transition: transform 0.1s, box-shadow 0.1s;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  .clip:hover {
    filter: brightness(1.1);
  }
  .clip.selected {
    border: 2px solid var(--accent);
    box-shadow: 0 0 10px rgba(0, 210, 180, 0.4);
    z-index: 5;
  }
  .clip-video {
    background: linear-gradient(135deg, #1e3c72, #2a5298);
  }
  .clip-audio {
    background: linear-gradient(135deg, #11998e, #38ef7d);
    color: #0b1a13;
  }
  .clip-text {
    background: linear-gradient(135deg, #8a2387, #e94057);
  }
  .clip-name {
    font-size: 11px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .clip-dur {
    font-size: 9px;
    opacity: 0.8;
    align-self: flex-end;
  }
  .playhead {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    pointer-events: none;
    z-index: 30;
  }
  .playhead-handle {
    width: 12px;
    height: 14px;
    background: var(--playhead);
    position: absolute;
    top: 0;
    left: -5px;
    clip-path: polygon(0 0, 100% 0, 100% 70%, 50% 100%, 0 70%);
  }
  .playhead-line {
    width: 2px;
    height: 100%;
    background: var(--playhead);
    position: absolute;
    left: 0;
  }
</style>
