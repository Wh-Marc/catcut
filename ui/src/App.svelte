<script>
  import { onMount, onDestroy } from 'svelte';
  import Header from './components/Header.svelte';
  import MediaBin from './components/MediaBin.svelte';
  import Preview from './components/Preview.svelte';
  import Inspector from './components/Inspector.svelte';
  import Timeline from './components/Timeline.svelte';
  import McpModal from './components/McpModal.svelte';
  import ExportModal from './components/ExportModal.svelte';
  import { fetchProject, subscribeToUpdates, callTool } from './lib/api.js';

  let project = null;
  let currentTimeSec = 0;
  let isPlaying = false;
  let selectedClipId = null;
  let aiLogs = [];
  let showMcpModal = false;
  let showExportModal = false;

  let animFrameId = null;
  let lastTimestamp = 0;
  let unsubscribeSse = null;

  $: totalDurationSec = (project?.duration || 0) / 1000000;

  async function loadProject() {
    try {
      project = await fetchProject();
    } catch (e) {
      console.warn("Could not fetch project:", e);
    }
  }

  function handlePlayheadTick(timestamp) {
    if (!lastTimestamp) lastTimestamp = timestamp;
    const dt = (timestamp - lastTimestamp) / 1000;
    lastTimestamp = timestamp;

    if (isPlaying) {
      currentTimeSec += dt;
      if (currentTimeSec >= totalDurationSec) {
        currentTimeSec = 0; // loop
      }
      animFrameId = requestAnimationFrame(handlePlayheadTick);
    }
  }

  function togglePlay() {
    isPlaying = !isPlaying;
    if (isPlaying) {
      lastTimestamp = 0;
      animFrameId = requestAnimationFrame(handlePlayheadTick);
    } else {
      if (animFrameId) cancelAnimationFrame(animFrameId);
    }
  }

  function handleSeek(newTimeSec) {
    currentTimeSec = Math.max(0, newTimeSec);
  }

  async function handleAddClip(materialId) {
    try {
      await callTool('add_clip', {
        material_id: materialId,
        target_start_sec: currentTimeSec,
      });
      await loadProject();
    } catch (e) {
      alert("Add clip error: " + e.message);
    }
  }

  function handleKeyDown(e) {
    // Ignore key shortcuts if focus is inside an input
    if (['INPUT', 'SELECT', 'TEXTAREA'].includes(document.activeElement?.tagName)) {
      return;
    }

    if (e.code === 'Space') {
      e.preventDefault();
      togglePlay();
    } else if (e.code === 'ArrowLeft') {
      e.preventDefault();
      handleSeek(Math.max(0, currentTimeSec - 1 / (project?.fps || 30)));
    } else if (e.code === 'ArrowRight') {
      e.preventDefault();
      handleSeek(currentTimeSec + 1 / (project?.fps || 30));
    } else if (e.key === 'b' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      if (selectedClipId) {
        callTool('split_clip', { clip_id: selectedClipId, split_time_sec: currentTimeSec })
          .then(loadProject)
          .catch(err => alert(err.message));
      }
    } else if (e.key === 'Delete' || e.key === 'Backspace') {
      if (selectedClipId) {
        e.preventDefault();
        callTool('remove_clip', { clip_id: selectedClipId, ripple: false })
          .then(() => {
            selectedClipId = null;
            loadProject();
          })
          .catch(err => alert(err.message));
      }
    } else if (e.key === 'z' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      callTool('undo').then(loadProject);
    } else if (e.key === 'y' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      callTool('redo').then(loadProject);
    }
  }

  onMount(async () => {
    await loadProject();

    // Listen for real-time SSE updates from MCP
    unsubscribeSse = subscribeToUpdates((data) => {
      const now = new Date();
      const timeStr = now.toTimeString().split(' ')[0];
      aiLogs = [{ time: timeStr, message: `AI modified timeline (${data})` }, ...aiLogs.slice(0, 49)];
      loadProject();
    });

    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    if (animFrameId) cancelAnimationFrame(animFrameId);
    if (unsubscribeSse) unsubscribeSse();
    window.removeEventListener('keydown', handleKeyDown);
  });
</script>

<main class="app-layout">
  <Header
    {project}
    onRefresh={loadProject}
    onShowMcpModal={() => showMcpModal = true}
    onShowExportModal={() => showExportModal = true}
  />

  <div class="workspace-main">
    <MediaBin
      {project}
      onRefresh={loadProject}
      onAddClip={handleAddClip}
      {aiLogs}
    />

    <Preview
      {project}
      {currentTimeSec}
      {isPlaying}
      onTogglePlay={togglePlay}
      onSeek={handleSeek}
    />

    <Inspector
      {project}
      bind:selectedClipId
      onRefresh={loadProject}
      {currentTimeSec}
    />
  </div>

  <Timeline
    {project}
    {currentTimeSec}
    {selectedClipId}
    onSeek={handleSeek}
    onRefresh={loadProject}
    onSelectClip={(id) => selectedClipId = id}
  />

  {#if showMcpModal}
    <McpModal onClose={() => showMcpModal = false} />
  {/if}

  {#if showExportModal}
    <ExportModal {project} onClose={() => showExportModal = false} />
  {/if}
</main>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background: var(--bg-base);
    color: var(--text-main);
  }
  .workspace-main {
    flex: 1;
    display: flex;
    min-height: 0;
    overflow: hidden;
  }
</style>
