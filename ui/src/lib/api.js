// API Client for CatCut - supports both Tauri IPC and HTTP/SSE MCP Backend

const BASE_URL = 'http://localhost:8765';
let reqId = 1;

export async function sendRpc(method, params = {}) {
  try {
    const res = await fetch(`${BASE_URL}/messages`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: reqId++,
        method,
        params,
      }),
    });
    const data = await res.json();
    if (data.error) {
      throw new Error(data.error.message || 'RPC Error');
    }
    return data.result;
  } catch (err) {
    console.error('sendRpc error:', err);
    throw err;
  }
}

export async function callTool(name, args = {}) {
  const result = await sendRpc('tools/call', {
    name,
    arguments: args,
  });
  return result;
}

export async function fetchProject() {
  const res = await fetch(`${BASE_URL}/api/project`);
  if (!res.ok) throw new Error('Failed to fetch project');
  return await res.json();
}

export function subscribeToUpdates(onUpdate) {
  try {
    const sse = new EventSource(`${BASE_URL}/sse`);
    sse.onmessage = (e) => {
      onUpdate(e.data);
    };
    sse.onerror = (err) => {
      console.warn('SSE connection waiting for server...', err);
    };
    return () => sse.close();
  } catch (e) {
    console.warn('Could not connect to SSE:', e);
    return () => {};
  }
}
