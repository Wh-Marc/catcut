use crate::tools::{execute_tool, get_tools_list};
use axum::{
    body::Bytes,
    extract::{Query, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use tower::ServiceExt;
use tower_http::services::ServeFile;
use catcut_core::*;
use futures::stream::Stream;
use serde::Deserialize;
use serde_json::{json, Value};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::CorsLayer;
use tracing::info;

pub struct ServerContext {
    pub engine: Arc<Mutex<TimelineEngine>>,
    pub project_path: Option<String>,
    pub sse_sender: broadcast::Sender<String>,
}

pub async fn handle_jsonrpc(
    ctx: &ServerContext,
    req: Value,
) -> Option<Value> {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(Value::Null);

    match method {
        "initialize" => {
            let res = json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "catcut-mcp",
                        "version": "0.1.0"
                    },
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        },
                        "resources": {
                            "subscribe": false,
                            "listChanged": false
                        }
                    }
                }
            });
            Some(res)
        }
        "notifications/initialized" => {
            // Notifications do not have response
            None
        }
        "ping" => {
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {}
            }))
        }
        "tools/list" => {
            let tools = get_tools_list();
            let tools_json: Vec<Value> = tools
                .into_iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "inputSchema": t.input_schema
                    })
                })
                .collect();

            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": tools_json
                }
            }))
        }
        "tools/call" => {
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            let res = execute_tool(
                ctx.engine.clone(),
                ctx.project_path.clone(),
                name,
                arguments,
            )
            .await;

            match res {
                Ok(content) => {
                    // Broadcast state update to any connected GUI clients
                    let _ = ctx.sse_sender.send("timeline_updated".to_string());

                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": content
                    }))
                }
                Err(err) => Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": err
                    }
                })),
            }
        }
        "resources/list" => {
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "resources": [
                        {
                            "uri": "catcut://timeline/summary",
                            "name": "Timeline Summary",
                            "mimeType": "text/markdown",
                            "description": "Current state and summary of CatCut timeline"
                        },
                        {
                            "uri": "catcut://project/json",
                            "name": "Project JSON",
                            "mimeType": "application/json",
                            "description": "Full serialized CatCut project schema"
                        }
                    ]
                }
            }))
        }
        "resources/read" => {
            let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            let eng = ctx.engine.lock().await;

            let (text, mime) = match uri {
                "catcut://timeline/summary" => (eng.summary().markdown, "text/markdown"),
                "catcut://project/json" => (
                    serde_json::to_string_pretty(&eng.project).unwrap_or_default(),
                    "application/json",
                ),
                _ => {
                    return Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32602,
                            "message": format!("Resource not found: {}", uri)
                        }
                    }));
                }
            };

            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "contents": [
                        {
                            "uri": uri,
                            "mimeType": mime,
                            "text": text
                        }
                    ]
                }
            }))
        }
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {}", method)
            }
        })),
    }
}

/// Run MCP Server over standard I/O (stdio)
pub async fn run_stdio(ctx: Arc<ServerContext>) -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    let mut stdout = tokio::io::stdout();

    info!("CatCut MCP Server listening on stdio");

    while let Ok(Some(line)) = reader.next_line().await {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(req) = serde_json::from_str::<Value>(line) {
            if let Some(resp) = handle_jsonrpc(&ctx, req).await {
                let resp_str = serde_json::to_string(&resp)? + "\n";
                stdout.write_all(resp_str.as_bytes()).await?;
                stdout.flush().await?;
            }
        } else {
            let err_resp = json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {
                    "code": -32700,
                    "message": "Parse error"
                }
            });
            let resp_str = serde_json::to_string(&err_resp)? + "\n";
            stdout.write_all(resp_str.as_bytes()).await?;
            stdout.flush().await?;
        }
    }

    Ok(())
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct SseQuery {
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Run MCP HTTP / SSE server (for web or remote GUI clients)
pub async fn run_http(ctx: Arc<ServerContext>, port: u16) -> anyhow::Result<()> {
    let dist_path = std::path::Path::new("dist");
    let fallback_service = if dist_path.exists() {
        tower_http::services::ServeDir::new("dist")
    } else {
        tower_http::services::ServeDir::new("../dist")
    };

    let app = Router::new()
        .route("/sse", get(sse_handler).post(messages_handler).delete(delete_handler))
        .route("/mcp", get(sse_handler).post(messages_handler).delete(delete_handler))
        .route("/messages", post(messages_handler).get(sse_handler))
        .route("/api/project", get(get_project_handler))
        .route("/api/media", get(media_stream_handler))
        .fallback_service(fallback_service)
        .layer(CorsLayer::permissive())
        .with_state(ctx);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("CatCut Web & MCP Server running at http://localhost:{}", port);
    eprintln!("✨ CatCut Web UI & MCP Server running at: http://localhost:{}", port);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn sse_handler(
    State(ctx): State<Arc<ServerContext>>,
    Query(_query): Query<SseQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = ctx.sse_sender.subscribe();

    let stream = async_stream::stream! {
        // Send initial endpoint event as required by MCP SSE transport
        let endpoint_event = Event::default()
            .event("endpoint")
            .data("/messages");
        yield Ok(endpoint_event);

        while let Ok(msg) = rx.recv().await {
            let event = Event::default()
                .event("message")
                .data(msg);
            yield Ok(event);
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

async fn delete_handler() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

async fn messages_handler(
    State(ctx): State<Arc<ServerContext>>,
    body: Bytes,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Mcp-Session-Id", HeaderValue::from_static("catcut-session-1"));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    if body.is_empty() {
        return (StatusCode::OK, headers, Json(json!({"jsonrpc": "2.0", "result": {}})));
    }

    let req: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                headers,
                Json(json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                })),
            );
        }
    };

    if let Some(resp) = handle_jsonrpc(&ctx, req).await {
        (StatusCode::OK, headers, Json(resp))
    } else {
        (StatusCode::OK, headers, Json(json!({"jsonrpc": "2.0", "result": "ok"})))
    }
}

#[derive(Deserialize)]
pub struct MediaPathQuery {
    pub path: String,
}

async fn media_stream_handler(
    Query(query): Query<MediaPathQuery>,
    req: Request,
) -> Result<impl IntoResponse, StatusCode> {
    let p = std::path::Path::new(&query.path);
    if !p.exists() || !p.is_file() {
        return Err(StatusCode::NOT_FOUND);
    }
    let service = ServeFile::new(p);
    match service.oneshot(req).await {
        Ok(res) => Ok(res.into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
async fn get_project_handler(State(ctx): State<Arc<ServerContext>>) -> Json<Project> {
    let eng = ctx.engine.lock().await;
    Json(eng.project.clone())
}
