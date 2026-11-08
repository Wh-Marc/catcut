mod server;
mod tools;

use clap::Parser;
use std::io::IsTerminal;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "catcut", about = "CatCut - Ultra-lightweight AI Video Editor with Native MCP")]
struct Cli {
    /// Server mode: "ui" / "sse" for web editor, or "stdio" for AI chat clients
    #[arg(short, long)]
    mode: Option<String>,

    /// HTTP port for UI / SSE mode
    #[arg(long, default_value_t = 8765)]
    port: u16,

    /// Do not automatically open the browser when launching in UI mode
    #[arg(long)]
    no_open: bool,

    /// Path to project JSON file to load and auto-save
    #[arg(short, long)]
    project: Option<String>,

    /// Path to CapCut draft_content.json to import initially
    #[arg(long)]
    capcut_draft: Option<String>,
}

fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/c", "start", url]).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "linux")]
    {
        if Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists() {
            let _ = std::process::Command::new("cmd.exe").args(["/c", "start", url]).spawn();
        } else {
            let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Write logs to stderr so stdio stdout is pure JSON-RPC
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    let cli = Cli::parse();

    // Smart mode detection: if mode is specified, use it.
    // If not specified: if stdin is a terminal, run UI mode; otherwise run stdio MCP mode.
    let mode = match cli.mode.as_deref() {
        Some(m) => m.to_lowercase(),
        None => {
            if std::io::stdin().is_terminal() {
                "ui".to_string()
            } else {
                "stdio".to_string()
            }
        }
    };

    let engine = if let Some(draft_path) = &cli.capcut_draft {
        let proj = catcut_core::import_from_capcut_file(Path::new(draft_path))?;
        eprintln!("📁 Loaded CapCut draft from: {}", draft_path);
        catcut_core::TimelineEngine::from_project(proj)
    } else if let Some(proj_path) = &cli.project {
        if Path::new(proj_path).exists() {
            let data = std::fs::read_to_string(proj_path)?;
            let proj: catcut_core::Project = serde_json::from_str(&data)?;
            eprintln!("📁 Loaded project from: {}", proj_path);
            catcut_core::TimelineEngine::from_project(proj)
        } else {
            catcut_core::TimelineEngine::new("CatCut Project", 1920, 1080, 30.0)
        }
    } else {
        catcut_core::TimelineEngine::new("CatCut Project", 1920, 1080, 30.0)
    };

    let project_file = cli.project.or_else(|| Some("catcut_project.json".to_string()));
    let (sse_tx, _) = broadcast::channel(100);

    let ctx = Arc::new(server::ServerContext {
        engine: Arc::new(Mutex::new(engine)),
        project_path: project_file,
        sse_sender: sse_tx,
    });

    if mode == "ui" || mode == "sse" {
        let url = format!("http://localhost:{}", cli.port);
        eprintln!("\n=======================================================");
        eprintln!("🎬 CatCut - Lightweight AI Video Editor");
        eprintln!("🌐 Web Interface & MCP SSE: {}", url);
        eprintln!("🤖 MCP Endpoint: {}/sse", url);
        eprintln!("=======================================================\n");

        if !cli.no_open {
            open_browser(&url);
        }

        server::run_http(ctx, cli.port).await?;
    } else {
        server::run_stdio(ctx).await?;
    }

    Ok(())
}
