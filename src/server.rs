use crate::assemble::Files;
use anyhow::{Context, Result};
use axum::{
    Router,
    body::Body,
    extract::{Request, State as AppState},
    response::Response,
};
use serde_json::json;
use std::{
    net::TcpListener,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
#[derive(Default)]
pub struct State {
    pub files: Files,
    pub revision: u64,
    pub error: Option<String>,
    pub dev: bool,
    /// Only the editor presentation origin allows asset reads by its opaque sandbox.
    pub editor_assets: bool,
}
pub struct LocalServer {
    pub url: String,
    pub state: Arc<RwLock<State>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
fn mime(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "mp4" => "video/mp4",
        _ => "application/octet-stream",
    }
}
async fn respond(AppState(shared): AppState<Arc<RwLock<State>>>, request: Request) -> Response {
    let decoded = percent_encoding::percent_decode_str(request.uri().path()).decode_utf8_lossy();
    let key = if decoded == "/" {
        "index.html"
    } else {
        decoded.trim_start_matches('/')
    };
    let state = shared.read().unwrap();
    let (status, data, content_type) = if !matches!(request.method().as_str(), "GET" | "HEAD") {
        (405, b"Method not allowed".to_vec(), "text/plain")
    } else if key == "__decksmith/status" && state.dev {
        (
            200,
            serde_json::to_vec(&json!({"revision":state.revision,"error":state.error})).unwrap(),
            "application/json",
        )
    } else if let Some(data) = state.files.get(key) {
        let body = if state.dev && key == "index.html" {
            String::from_utf8_lossy(data)
                .replace("/*decksmith-revision*/0", &state.revision.to_string())
                .into_bytes()
        } else {
            data.clone()
        };
        (200, body, mime(key))
    } else {
        (404, b"Not found".to_vec(), "text/plain")
    };
    let mut response = Response::builder()
        .status(status)
        .header("Content-Type", content_type)
        .header("Content-Length", data.len())
        .header("Cache-Control", "no-store")
        .header("X-Content-Type-Options", "nosniff");
    if state.editor_assets {
        response = response.header("Access-Control-Allow-Origin", "null");
    }
    response
        .body(Body::from(if request.method() == "HEAD" {
            Vec::new()
        } else {
            data
        }))
        .unwrap()
}
impl LocalServer {
    pub fn start(files: Files, port: u16, dev: bool) -> Result<Self> {
        let socket =
            TcpListener::bind(("127.0.0.1", port)).context("Cannot bind loopback preview")?;
        socket.set_nonblocking(true)?;
        let url = format!("http://{}/", socket.local_addr()?);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let listener = {
            let _entered = runtime.enter();
            tokio::net::TcpListener::from_std(socket)?
        };
        let state = Arc::new(RwLock::new(State {
            files,
            dev,
            ..State::default()
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let router = Router::new().fallback(respond).with_state(state.clone());
        let stop2 = stop.clone();
        let thread = thread::spawn(move || {
            runtime.block_on(async move {
                let shutdown = stop2.clone();
                let mut task = tokio::spawn(async move {
                    let _ = axum::serve(listener, router)
                        .with_graceful_shutdown(async move {
                            while !shutdown.load(Ordering::Relaxed) {
                                tokio::time::sleep(Duration::from_millis(25)).await;
                            }
                        })
                        .await;
                });
                while !stop2.load(Ordering::Relaxed) {
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
                // Give requests a chance to complete, then tear down the owned runtime.
                if tokio::time::timeout(Duration::from_secs(2), &mut task)
                    .await
                    .is_err()
                {
                    task.abort();
                }
            });
            runtime.shutdown_timeout(Duration::from_secs(1));
        });
        Ok(Self {
            url,
            state,
            stop,
            thread: Some(thread),
        })
    }
}
impl Drop for LocalServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
