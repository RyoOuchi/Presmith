use decksmith::{assemble, manifest};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
use tempfile::TempDir;
fn cli(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_decksmith"))
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}
fn fresh() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    decksmith::init(temp.path()).unwrap();
    temp
}
fn edit(root: &Path, f: impl FnOnce(&mut Value)) {
    let path = root.join("deck.json");
    let mut v = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    f(&mut v);
    fs::write(path, serde_json::to_vec_pretty(&v).unwrap()).unwrap();
}
#[test]
fn compiled_binary_initializes_outside_checkout_with_spaces() {
    let temp = tempfile::tempdir().unwrap();
    let result = cli(temp.path(), &["init", "a talk with spaces"]);
    assert!(result.status.success(), "{:?}", result);
    let project = temp.path().join("a talk with spaces");
    let (_, m) = manifest::load(&project).unwrap();
    assert_eq!(m.slides.len(), 3);
    for file in [
        "lib/decksmith.js",
        "tooling/renderer/package-lock.json",
        "tooling/renderer/render.mjs",
        "AGENTS.md",
        "README.md",
        "docs/runtime.md",
    ] {
        assert!(
            project.join(file).is_file(),
            "Missing embedded asset: {file}"
        );
    }
    assert!(!project.join("tooling/renderer/node_modules").exists());
}
#[test]
fn never_overwrites_nonempty_directory() {
    let t = tempfile::tempdir().unwrap();
    fs::write(t.path().join("keep"), "original").unwrap();
    let out = cli(t.path(), &["init", "."]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        fs::read_to_string(t.path().join("keep")).unwrap(),
        "original"
    );
    assert!(!t.path().join("deck.json").exists());
}
#[test]
fn empty_existing_directory_is_supported() {
    let t = tempfile::tempdir().unwrap();
    assert!(cli(t.path(), &["init", "."]).status.success());
}
#[test]
fn manifest_order_and_script_data_are_preserved_safely() {
    let t = fresh();
    edit(t.path(), |m| {
        m["slides"].as_array_mut().unwrap().reverse();
        m["title"] = json!("A </script><script>alert(1)</script> & B");
    });
    let (root, m) = manifest::load(t.path()).unwrap();
    let files = assemble::assemble(&root, &m, false).unwrap();
    let html = String::from_utf8(files["index.html"].clone()).unwrap();
    assert!(
        html.find("data-slide-id=\"next\"").unwrap()
            < html.find("data-slide-id=\"intro\"").unwrap()
    );
    assert!(!html.contains("</script><script>alert(1)"));
    assert!(html.contains("\\u003c/script\\u003e"));
}
#[test]
fn malformed_manifest_is_json_and_a_deck_error() {
    let t = fresh();
    fs::write(t.path().join("deck.json"), "{").unwrap();
    let out = cli(t.path(), &["check", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["success"], false);
    assert_eq!(value["findings"][0]["rule_id"], "manifest.invalid");
}
#[test]
fn duplicate_ids_and_missing_files_are_actionable() {
    let t = fresh();
    edit(t.path(), |m| {
        m["slides"][1]["id"] = json!("intro");
        m["slides"][1]["source"] = json!("slides/not-here.html");
    });
    let out = cli(t.path(), &["check", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let rules: Vec<_> = v["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["rule_id"].as_str().unwrap())
        .collect();
    assert!(rules.contains(&"manifest.duplicate_id"));
    assert!(rules.contains(&"manifest.source"));
}
#[test]
fn rejects_traversal_absolute_and_hidden_paths() {
    for path in [
        "../outside.html",
        "/etc/passwd",
        "slides/../../secret",
        "slides\\intro.html",
        ".git/config",
        "slides/%2e%2e/foo",
        "https://example.com/x",
    ] {
        let t = fresh();
        edit(t.path(), |m| m["slides"][0]["source"] = json!(path));
        assert!(manifest::load(t.path()).is_err(), "Accepted {path}");
    }
}
#[cfg(unix)]
#[test]
fn rejects_symlink_escape_and_presentation_symlinks() {
    let t = fresh();
    let outside = tempfile::NamedTempFile::new().unwrap();
    std::os::unix::fs::symlink(outside.path(), t.path().join("slides/escape.html")).unwrap();
    edit(t.path(), |m| {
        m["slides"][0]["source"] = json!("slides/escape.html")
    });
    assert!(manifest::load(t.path()).is_err());
    let t = fresh();
    std::os::unix::fs::symlink(outside.path(), t.path().join("assets/escape.txt")).unwrap();
    let (root, m) = manifest::load(t.path()).unwrap();
    assert!(assemble::assemble(&root, &m, false).is_err());
}
#[test]
fn missing_dependencies_are_not_reported_as_success() {
    let t = fresh();
    let out = cli(t.path(), &["doctor", "--json"]);
    assert_eq!(out.status.code(), Some(2));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["capabilities"]["preview"], true);
    assert_eq!(v["capabilities"]["render"], false);
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("decksmith setup")
    );
}
#[test]
fn json_usage_errors_are_parseable() {
    let t = tempfile::tempdir().unwrap();
    let out = cli(t.path(), &["export", "--format", "pptx", "--json"]);
    assert_eq!(out.status.code(), Some(2));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["error"]["kind"], "usage");
}
#[test]
fn generated_html_excludes_project_secrets_and_tooling() {
    let t = fresh();
    fs::write(t.path().join("secret.txt"), "not public").unwrap();
    fs::write(t.path().join("assets/.env"), "not public").unwrap();
    let (root, m) = manifest::load(t.path()).unwrap();
    let files = assemble::assemble(&root, &m, false).unwrap();
    assert!(!files.keys().any(|k| k.contains("tooling")
        || k.contains("secret")
        || k.contains(".env")
        || k.starts_with("slides/")));
    assert!(files.contains_key("index.html"));
}
#[test]
fn output_cannot_replace_authored_sources() {
    let t = fresh();
    let out = cli(t.path(), &["render", "--out", "slides", "--json"]);
    assert_eq!(out.status.code(), Some(2));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["error"]["message"].as_str().unwrap().contains("authored"));
}

#[test]
fn setup_with_missing_node_is_actionable() {
    let t = fresh();
    let out = Command::new(env!("CARGO_BIN_EXE_decksmith"))
        .args(["setup", "."])
        .current_dir(t.path())
        .env("PATH", "")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Node.js 22+"));
}

#[test]
fn loopback_server_handles_burst_keepalive_connections_and_closes() {
    use std::{
        collections::BTreeMap,
        io::{BufRead, BufReader, Read, Write},
        net::TcpStream,
        sync::{Arc, Barrier},
        thread,
        time::Duration,
    };
    let server = decksmith::server::LocalServer::start(
        BTreeMap::from([("index.html".into(), b"hello".to_vec())]),
        0,
        false,
    )
    .unwrap();
    let address = server
        .url
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string();
    let barrier = Arc::new(Barrier::new(32));
    let tasks: Vec<_> = (0..32)
        .map(|_| {
            let address = address.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                let mut stream = TcpStream::connect(address).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                barrier.wait();
                stream
                    .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
                    .unwrap();
                let mut reader = BufReader::new(stream);
                let mut status = String::new();
                reader.read_line(&mut status).unwrap();
                assert!(status.contains("200"));
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                }
                let mut body = [0; 5];
                reader.read_exact(&mut body).unwrap();
                assert_eq!(&body, b"hello");
                reader.into_inner()
            })
        })
        .collect();
    let sockets: Vec<_> = tasks.into_iter().map(|t| t.join().unwrap()).collect();
    drop(server);
    assert!(TcpStream::connect(&address).is_err());
    drop(sockets);
}
