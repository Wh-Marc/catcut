<script>
  import { Play, Pause, SkipBack, SkipForward, Maximize2, Camera, Ratio } from 'lucide-svelte';
  import { callTool } from '../lib/api.js';

  export let project;
  export let currentTimeSec = 0;
  export let isPlaying = false;
  export let onTogglePlay;
  export let onSeek;

  let aspectRatio = '16:9'; // '16:9' | '9:16' | '1:1'
  let isExtractingFrame = false;
  let previewSnapshotUrl = null;

  $: totalDurationSec = (project?.duration || 0) / 1000000;

  // Find active video and text clips at currentTimeSec
  $: currentUs = currentTimeSec * 1000000;
  $: activeVideoClip = project?.tracks
    ?.find(t => t.track_type === 'video')
    ?.segments?.find(s => currentUs >= s.target_timerange.start && currentUs < (s.target_timerange.start + s.target_timerange.duration));

  $: activeTextClip = project?.tracks
    ?.find(t => t.track_type === 'text')
    ?.segments?.find(s => currentUs >= s.target_timerange.start && currentUs < (s.target_timerange.start + s.target_timerange.duration));

  $: activeMaterial = activeVideoClip ? project?.materials?.find(m => m.id === activeVideoClip.material_id) : null;

  let videoEl;
  let lastLoadedSrc = '';

  $: videoSrc = activeMaterial ? `http://localhost:8765/api/media?path=${encodeURIComponent(activeMaterial.path)}` : null;

  $: if (videoEl && videoSrc) {
    if (lastLoadedSrc !== videoSrc) {
      lastLoadedSrc = videoSrc;
      videoEl.src = videoSrc;
      videoEl.load();
    }

    if (activeVideoClip) {
      const targetStartSec = activeVideoClip.target_timerange.start / 1000000;
      const sourceStartSec = activeVideoClip.source_timerange.start / 1000000;
      const offsetInTarget = currentTimeSec - targetStartSec;
      const targetMediaTime = sourceStartSec + (offsetInTarget * (activeVideoClip.speed || 1.0));

      if (Math.abs(videoEl.currentTime - targetMediaTime) > 0.25) {
        videoEl.currentTime = Math.max(0, targetMediaTime);
      }

      videoEl.playbackRate = activeVideoClip.speed || 1.0;
      videoEl.volume = Math.min(1.0, activeVideoClip.audio?.volume ?? 1.0);
      videoEl.muted = activeVideoClip.audio?.is_muted || false;

      if (isPlaying && videoEl.paused) {
        videoEl.play().catch(() => {});
      } else if (!isPlaying && !videoEl.paused) {
        videoEl.pause();
      }
    }
  } else if (videoEl && !videoSrc) {
    if (!videoEl.paused) videoEl.pause();
  }

  function formatTimecode(sec) {
    const sTotal = Math.max(0, sec || 0);
    const m = Math.floor(sTotal / 60);
    const s = Math.floor(sTotal % 60);
    const ms = Math.floor((sTotal % 1) * 1000);
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}.${Math.floor(ms / 10).toString().padStart(2, '0')}`;
  }
  async function handleSnapshot() {
    isExtractingFrame = true;
    try {
      const res = await callTool('inspect_frame', { timestamp_sec: currentTimeSec, max_width: 640 });
      const imgContent = res.content?.find(c => c.type === 'image');
      if (imgContent) {
        previewSnapshotUrl = `data:${imgContent.mimeType};base64,${imgContent.data}`;
      } else {
        alert(res.content?.[0]?.text || "No frame data returned");
      }
    } catch (e) {
      alert("Snapshot error: " + e.message);
    } finally {
      isExtractingFrame = false;
    }
  }

  function stepFrame(frames) {
    const fps = project?.fps || 30;
    const dt = frames / fps;
    onSeek(Math.max(0, Math.min(totalDurationSec, currentTimeSec + dt)));
  }
</script>

<div class="preview-panel">
  <div class="viewport-container" class:aspect-16-9={aspectRatio === '16:9'} class:aspect-9-16={aspectRatio === '9:16'} class:aspect-1-1={aspectRatio === '1:1'}>
    <div class="screen">
      {#if previewSnapshotUrl}
        <div class="snapshot-wrapper">
          <img src={previewSnapshotUrl} alt="Preview Snapshot" class="snapshot-img" />
          <button class="close-snapshot" on:click={() => previewSnapshotUrl = null}>Back to Video ✕</button>
        </div>
      {:else if activeMaterial}
        <div class="canvas-content" style="
          transform: scale({activeVideoClip?.transform?.scale_x || 1}) rotate({activeVideoClip?.transform?.rotation || 0}deg) translate({(activeVideoClip?.transform?.position_x || 0) * 100}px, {(activeVideoClip?.transform?.position_y || 0) * 100}px);
          opacity: {activeVideoClip?.transform?.opacity ?? 1};
        ">
          <video
            bind:this={videoEl}
            class="active-video-element"
            playsinline
            preload="auto"
          ></video>
        </div>
      {:else}
        <div class="no-clip-state">
          <span>No video active at playhead</span>
        </div>
      {/if}

      <!-- Text Overlay Layer -->
      {#if activeTextClip && activeTextClip.text_content}
        <div class="text-overlay" style="
          font-size: {(activeTextClip.text_style?.font_size || 48) / 2}px;
          color: {activeTextClip.text_style?.color || '#FFFFFF'};
          transform: translate({(activeTextClip.transform?.position_x || 0) * 100}px, {(activeTextClip.transform?.position_y || 0) * 100}px);
        ">
          {activeTextClip.text_content}
        </div>
      {/if}
    </div>
  </div>

  <!-- Transport Controls Bar -->
  <div class="controls-bar">
    <div class="left-controls">
      <div class="timecode">
        <span class="current">{formatTimecode(currentTimeSec)}</span>
        <span class="divider">/</span>
        <span class="total">{formatTimecode(totalDurationSec)}</span>
      </div>
    </div>

    <div class="playback-controls">
      <button class="control-btn" on:click={() => stepFrame(-1)} title="Previous Frame (Left Arrow)">
        <SkipBack size={16} />
      </button>

      <button class="play-btn" on:click={onTogglePlay} title="Play / Pause (Space)">
        {#if isPlaying}
          <Pause size={18} />
        {:else}
          <Play size={18} fill="currentColor" />
        {/if}
      </button>

      <button class="control-btn" on:click={() => stepFrame(1)} title="Next Frame (Right Arrow)">
        <SkipForward size={16} />
      </button>
    </div>

    <div class="right-controls">
      <button class="tool-btn" on:click={handleSnapshot} disabled={isExtractingFrame} title="Inspect Frame (MCP Multimodal capture)">
        <Camera size={15} />
        <span>{isExtractingFrame ? 'Extracting...' : 'Inspect Frame'}</span>
      </button>

      <div class="ratio-select">
        <button class="tool-btn" on:click={() => {
          if (aspectRatio === '16:9') aspectRatio = '9:16';
          else if (aspectRatio === '9:16') aspectRatio = '1:1';
          else aspectRatio = '16:9';
        }}>
          <Ratio size={14} />
          <span>{aspectRatio}</span>
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .preview-panel {
    flex: 1;
    background: var(--bg-base);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }
  .viewport-container {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    padding: 24px;
    box-sizing: border-box;
  }
  .screen {
    background: #000;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.6);
    border-radius: 6px;
    position: relative;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: width 0.3s, height 0.3s;
  }
  .aspect-16-9 .screen {
    width: 640px;
    height: 360px;
  }
  .aspect-9-16 .screen {
    width: 270px;
    height: 480px;
  }
  .aspect-1-1 .screen {
    width: 380px;
    height: 380px;
  }
  .snapshot-img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
  .canvas-content {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.1s ease-out;
  }
  .active-video-element {
    width: 100%;
    height: 100%;
    object-fit: contain;
    background: #000;
  }
  .snapshot-wrapper {
    width: 100%;
    height: 100%;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .close-snapshot {
    position: absolute;
    top: 10px;
    right: 10px;
    background: rgba(0, 0, 0, 0.7);
    color: var(--accent);
    border: 1px solid var(--accent);
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 11px;
    cursor: pointer;
  }
  .no-clip-state {
    color: #4b5563;
    font-size: 13px;
    font-style: italic;
  }
  .text-overlay {
    position: absolute;
    bottom: 20%;
    left: 0;
    right: 0;
    text-align: center;
    font-weight: 700;
    text-shadow: 0 2px 8px rgba(0, 0, 0, 0.8);
    pointer-events: none;
    z-index: 10;
  }
  .controls-bar {
    height: 44px;
    width: 100%;
    background: var(--bg-surface);
    border-top: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
  }
  .left-controls, .playback-controls, .right-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .timecode {
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px;
    background: var(--bg-panel);
    padding: 4px 8px;
    border-radius: 4px;
    border: 1px solid var(--border-color);
  }
  .current {
    color: var(--text-main);
    font-weight: 600;
  }
  .divider {
    color: var(--border-active);
    margin: 0 4px;
  }
  .total {
    color: var(--text-muted);
  }
  .control-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }
  .control-btn:hover {
    background: var(--bg-hover);
    color: var(--text-main);
  }
  .play-btn {
    background: var(--accent);
    color: #000;
    border: none;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    box-shadow: 0 2px 10px rgba(0, 210, 180, 0.3);
  }
  .play-btn:hover {
    background: var(--accent-hover);
    transform: scale(1.05);
  }
  .tool-btn {
    background: var(--bg-panel);
    border: 1px solid var(--border-color);
    color: var(--text-main);
    font-size: 12px;
    padding: 5px 10px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }
  .tool-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    border-color: var(--border-active);
  }
  .tool-btn:disabled {
    opacity: 0.5;
  }
</style>
