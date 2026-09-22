use super::*;
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{Request, State},
    response::Response,
};
use std::{
    net::TcpListener,
    sync::{Arc, Mutex, atomic::Ordering},
    time::Duration,
};
struct Session {
    base: Snapshot,
    baselines: BTreeMap<String, Snapshot>,
    preview: crate::server::LocalServer,
    token: String,
    host: String,
}
fn response(status: u16, data: impl Into<Vec<u8>>, mime: &str) -> Response {
    Response::builder().status(status).header("Content-Type",mime).header("Cache-Control","no-store").header("X-Content-Type-Options","nosniff").header("Referrer-Policy","no-referrer").header("Content-Security-Policy","default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; frame-src http://127.0.0.1:*; frame-ancestors 'none'; object-src 'none'; base-uri 'none'").body(Body::from(data.into())).unwrap()
}
fn reply(status: u16, v: Value) -> Response {
    response(status, serde_json::to_vec(&v).unwrap(), "application/json")
}
fn data(session: &mut Session) -> Result<Value> {
    session.base = Snapshot::load(&session.base.root)?;
    session
        .baselines
        .entry(session.base.revision.clone())
        .or_insert_with(|| session.base.clone());
    session
        .preview
        .state
        .write()
        .unwrap()
        .files
        .extend(session.base.assembled(false)?);
    let mut v = session.base.data()?;
    let files = session.base.assembled(true)?;
    let mut html = String::from_utf8(files["index.html"].clone())?;
    html = html.replacen(
        "<head>",
        &format!("<head><base href=\"{}\">", session.preview.url),
        1,
    );
    // Capture source node references before any authored scripts execute.
    let bridge = "<script src=\"__editor/bridge.js\"></script>".to_owned();
    html = html.replacen(
        "<script src=\"lib/decksmith.js\">",
        &(bridge + "<script src=\"lib/decksmith.js\">"),
        1,
    );
    let mut state = session.preview.state.write().unwrap();
    state.files.insert(
        format!("__editor/frame-{}.html", session.base.revision),
        html.into_bytes(),
    );
    state.files.insert(
        "__editor/bridge.js".into(),
        include_bytes!("../../editor/build/bridge.js").to_vec(),
    );
    v["frame_url"] = json!(format!(
        "{}__editor/frame-{}.html",
        session.preview.url, session.base.revision
    ));
    Ok(v)
}
async fn respond(State(shared): State<Arc<Mutex<Session>>>, request: Request) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();
    {
        let s = shared.lock().unwrap();
        if request.headers().get("host").and_then(|v| v.to_str().ok()) != Some(&s.host) {
            return reply(403, json!({"error":"Invalid editor Host"}));
        }
        if let Some(origin) = request.headers().get("origin")
            && origin.to_str().ok() != Some(&format!("http://{}", s.host))
        {
            return reply(403, json!({"error":"Disallowed editor origin"}));
        }
        if path.starts_with("/api/") {
            if request
                .headers()
                .get("x-decksmith-session")
                .and_then(|v| v.to_str().ok())
                != Some(&s.token)
            {
                return reply(
                    401,
                    json!({"error":"Editor session authorization required; reopen the CLI URL"}),
                );
            }
            if method != "GET" && request.headers().get("origin").is_none() {
                return reply(403, json!({"error":"Mutation requires the editor origin"}));
            }
            if method != "GET"
                && request
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    != Some("application/json")
            {
                return reply(415, json!({"error":"Use application/json"}));
            }
        }
    }
    if !path.starts_with("/api/") {
        if method != "GET" {
            return reply(405, json!({"error":"Method not allowed"}));
        }
        return match path.as_str() {
            "/" => response(
                200,
                include_bytes!("../../editor/build/index.html").to_vec(),
                "text/html; charset=utf-8",
            ),
            "/editor.js" => response(
                200,
                include_bytes!("../../editor/build/editor.js").to_vec(),
                "text/javascript; charset=utf-8",
            ),
            "/licenses.txt" => response(
                200,
                include_bytes!("../../editor/build/licenses.txt").to_vec(),
                "text/plain; charset=utf-8",
            ),
            "/editor.js.LEGAL.txt" => response(
                200,
                include_bytes!("../../editor/build/editor.js.LEGAL.txt").to_vec(),
                "text/plain; charset=utf-8",
            ),
            "/editor.css" => response(
                200,
                include_bytes!("../../editor/build/editor.css").to_vec(),
                "text/css; charset=utf-8",
            ),
            _ => reply(404, json!({"error":"Not found"})),
        };
    }
    let body = match to_bytes(request.into_body(), 30_000_000).await {
        Ok(b) => b,
        Err(_) => return reply(413, json!({"error":"Request too large (30 MB maximum)"})),
    };
    // Blocking disk work and existing synchronous exports must not block Axum's event loop.
    let result = tokio::task::spawn_blocking(move || -> Result<Value> {
        let mut s = shared.lock().unwrap();
        match (method.as_str(), path.as_str()) {
            ("GET", "/api/project") => data(&mut s),
            ("GET", "/api/status") => {
                Ok(json!({"revision":Snapshot::load(&s.base.root).context("CONFLICT: Current source is invalid; repair it before reloading")?.revision}))
            }
            ("POST", "/api/save") => {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Save {
                    revision: String,
                    baseline: String,
                    operations: Vec<Operation>,
                }
                let payload: Save = serde_json::from_slice(&body)?;
                let baseline = s.baselines.get(&payload.baseline).context("Unknown editing baseline; reload the project")?;
                let revision = baseline.save(&payload.revision, &payload.operations)?;
                let files = Snapshot::load(&s.base.root)?.assembled(false)?;
                s.preview.state.write().unwrap().files.extend(files);
                Ok(json!({"revision":revision}))
            }
            ("POST", "/api/reload") => {
                let v: Value = serde_json::from_slice(&body)?;
                ensure!(
                    v == json!({"discard":true}),
                    "Reload explicitly requires discarding the pending session"
                );
                s.base = Snapshot::load(&s.base.root)?;
                s.preview.state.write().unwrap().files.extend(s.base.assembled(false)?);
                data(&mut s)
            }
            ("POST", "/api/export") => {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Export {
                    format: String,
                    revision: String,
                }
                let payload: Export = serde_json::from_slice(&body)?;
                ensure!(
                    ["html", "png", "pdf", "pptx"].contains(&payload.format.as_str()),
                    "Unsupported export format"
                );
                ensure!(
                    Snapshot::load(&s.base.root)?.revision == payload.revision,
                    "CONFLICT: Source changed; reload before exporting"
                );
                let v = if payload.format == "png" {
                    crate::browser_command("render", &s.base.root, None, None, None)?
                } else {
                    crate::browser_command(
                        "export",
                        &s.base.root,
                        None,
                        None,
                        Some(&payload.format),
                    )?
                };
                ensure!(v["success"] == true, "Export failed: {}", v);
                Ok(v)
            }
            _ => bail!("Unknown editor endpoint or method"),
        }
    })
    .await;
    match result {
        Ok(Ok(v)) => reply(200, v),
        Ok(Err(e)) => {
            let message = format!("{e:#}");
            reply(
                if message.contains("CONFLICT:") {
                    409
                } else {
                    422
                },
                json!({"error":message}),
            )
        }
        Err(e) => reply(500, json!({"error":format!("Editor worker failed: {e}")})),
    }
}
pub fn edit(directory: &Path, port: u16, open: bool, recover: bool) -> Result<()> {
    if recover {
        let root = directory.canonicalize()?;
        transaction::recover(&root)?;
        eprintln!("Recovered interrupted save.");
    }
    let base = Snapshot::load(directory)?;
    let preview = crate::server::LocalServer::start(base.assembled(false)?, 0, false)?;
    preview.state.write().unwrap().editor_assets = true;
    let socket = TcpListener::bind(("127.0.0.1", port)).context("Cannot bind loopback editor")?;
    socket.set_nonblocking(true)?;
    let host = socket.local_addr()?.to_string();
    let mut secret = [0u8; 32];
    getrandom::getrandom(&mut secret)
        .map_err(|e| anyhow::anyhow!("Cannot create editor session token: {e}"))?;
    let token = hash(&secret);
    let url = format!("http://{host}/#session={token}");
    let shared = Arc::new(Mutex::new(Session {
        baselines: BTreeMap::new(),
        base,
        preview,
        token,
        host,
    }));
    eprintln!("Editor: {url} (Ctrl-C to stop)");
    if open {
        let program = if cfg!(target_os = "macos") {
            "open"
        } else if cfg!(windows) {
            "explorer"
        } else {
            "xdg-open"
        };
        if let Err(e) = std::process::Command::new(program).arg(&url).status() {
            eprintln!("Could not open browser: {e}. Open the Editor URL manually.");
        }
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        let listener = tokio::net::TcpListener::from_std(socket)?;
        axum::serve(listener, Router::new().fallback(respond).with_state(shared))
            .with_graceful_shutdown(async {
                while !crate::process::INTERRUPTED.load(Ordering::Relaxed) {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            })
            .await
            .map_err(anyhow::Error::from)
    })
}
