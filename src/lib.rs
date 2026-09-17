pub mod assemble;
pub mod manifest;
pub mod process;
pub mod server;
use anyhow::{Context, Result, bail};
use manifest::Problems;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::atomic::Ordering,
    thread,
    time::Duration,
};
include!(concat!(env!("OUT_DIR"), "/assets.rs"));

pub fn init(directory: &Path) -> Result<()> {
    if directory.exists() && (!directory.is_dir() || fs::read_dir(directory)?.next().is_some()) {
        bail!(
            "Refusing to overwrite nonempty directory: {}",
            directory.display()
        );
    }
    fs::create_dir_all(directory)?;
    for (name, data) in ASSETS {
        let target = directory.join(name);
        fs::create_dir_all(target.parent().unwrap())?;
        fs::write(target, data)?;
    }
    fs::create_dir_all(directory.join("assets"))?;
    eprintln!(
        "Created {}. No dependencies installed.\nNext steps:\n  cd '{}'\n  decksmith setup\n  decksmith doctor\n  decksmith dev --open\n  decksmith check\n  decksmith render\n  decksmith export --format html\n  decksmith export --format pdf",
        directory.display(),
        directory.display()
    );
    Ok(())
}
pub fn setup(directory: &Path) -> Result<()> {
    let (root, _) = manifest::load(directory)?;
    process::run(
        Command::new("node").args([
            "-e",
            "if(Number(process.versions.node.split('.')[0])<22)process.exit(1)",
        ]),
        None,
        Duration::from_secs(15),
    )
    .context("Node.js 22+ is required. Install Node.js with npm, then rerun decksmith setup")?;
    let dir = root.join("tooling/renderer");
    if !dir.join("package-lock.json").is_file() {
        bail!(
            "Missing tooling/renderer/package-lock.json; restore renderer files from a fresh decksmith init project"
        );
    }
    eprintln!("Installing pinned renderer dependencies locally…");
    let output = process::run(
        Command::new(if cfg!(windows) { "npm.cmd" } else { "npm" })
            .args(["ci", "--no-audit", "--no-fund", "--ignore-scripts"])
            .env("npm_config_cache", dir.join(".npm-cache"))
            .current_dir(&dir),
        None,
        Duration::from_secs(600),
    )
    .context("npm ci failed. Check npm and network access, then rerun decksmith setup")?;
    eprintln!("{output}Installing project-local Chromium…");
    let output = process::run(Command::new("node").args(["node_modules/playwright/cli.js", "install", "chromium"]).env("PLAYWRIGHT_BROWSERS_PATH", dir.join(".browsers")).current_dir(&dir), None, Duration::from_secs(600)).context("Chromium installation failed. Check network/disk space. On Linux install Playwright system libraries; see README")?;
    eprintln!("{output}");
    let probe = process::helper(&root, &json!({"action":"probe"}))?;
    if probe["success"] != true {
        bail!("Chromium could not launch: {}", probe["error"]);
    }
    eprintln!("Ready: preview, checks, screenshots, HTML verification and PDF export.");
    Ok(())
}
pub fn envelope(command: &str) -> Value {
    json!({"schema_version":1,"command":command,"success":true,"findings":[],"artifacts":[],"error":null})
}
pub fn doctor(directory: &Path) -> (Value, i32) {
    let mut result = envelope("doctor");
    let loaded = manifest::load(directory);
    let mut capabilities = json!({"preview":false,"check":false,"render":false,"html_export":false,"pdf_export":false});
    let mut code = 0;
    match loaded {
        Ok((root, m)) => {
            match assemble::assemble(&root, &m, false) {
                Ok(_) => capabilities["preview"] = json!(true),
                Err(e) => {
                    result["error"] = json!({"kind":"operational","message":e.to_string()});
                    code = 2;
                }
            }
            let probe = process::helper(&root, &json!({"action":"probe"}));
            match probe {
                Ok(v) if v["success"] == true && code == 0 => {
                    for key in ["check", "render", "html_export", "pdf_export"] {
                        capabilities[key] = json!(true);
                    }
                    result["runtime"] = v["runtime"].clone();
                }
                other => {
                    code = 2;
                    result["error"] = json!({"kind":"operational","message":match other { Ok(v) => v["error"]["message"].as_str().unwrap_or("Browser probe failed").to_string(), Err(e) => format!("{e:#}") }});
                }
            }
        }
        Err(e) => {
            let (failure, c) = failure("doctor", &e);
            result = failure;
            code = c;
        }
    }
    result["capabilities"] = capabilities;
    result["success"] = json!(code == 0);
    (result, code)
}
pub fn failure(command: &str, e: &anyhow::Error) -> (Value, i32) {
    let mut value = envelope(command);
    value["success"] = json!(false);
    let code = if let Some(p) = e.downcast_ref::<Problems>() {
        value["findings"] = json!(p.0);
        1
    } else {
        2
    };
    value["error"] =
        json!({"kind":if code == 1 {"deck"} else {"operational"},"message":format!("{e:#}")});
    (value, code)
}
fn output_path(root: &Path, provided: Option<&Path>, default: &str) -> Result<PathBuf> {
    let path = match provided {
        Some(p) if p.is_absolute() => p.to_path_buf(),
        Some(p) => std::env::current_dir()?.join(p),
        None => root.join(default),
    };
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        bail!("Output path cannot contain '..'");
    }
    // Resolve the nearest existing ancestor before checking containment, including symlinks.
    let mut ancestor = path.as_path();
    let mut suffix = Vec::new();
    while !ancestor.exists() {
        suffix.push(
            ancestor
                .file_name()
                .context("Invalid output path")?
                .to_owned(),
        );
        ancestor = ancestor.parent().context("Invalid output path")?;
    }
    let mut resolved = ancestor.canonicalize()?;
    for component in suffix.iter().rev() {
        resolved.push(component);
    }
    if root.starts_with(&resolved)
        || (resolved.starts_with(root)
            && !resolved.starts_with(root.join("dist"))
            && !resolved.starts_with(root.join(".decksmith")))
    {
        bail!(
            "Outputs inside the project must live in dist/ or .decksmith/; refusing to overwrite authored files"
        );
    }
    Ok(resolved)
}
fn install_output(temp: &Path, out: &Path, directory: bool) -> Result<()> {
    if directory {
        if out.exists() {
            if !out.is_dir() {
                bail!("Output exists and is not a directory: {}", out.display());
            }
            if fs::read_dir(out)?.next().is_some()
                && fs::read_to_string(out.join(".decksmith-output")).unwrap_or_default()
                    != "decksmith-output-v1\n"
            {
                bail!(
                    "Refusing to overwrite an unmanaged output directory: {}",
                    out.display()
                );
            }
            fs::remove_dir_all(out)?;
        }
        fs::create_dir_all(out)?;
        fn copy_dir(from: &Path, to: &Path) -> Result<()> {
            for item in fs::read_dir(from)? {
                let p = item?.path();
                let t = to.join(p.file_name().unwrap());
                if p.is_dir() {
                    fs::create_dir_all(&t)?;
                    copy_dir(&p, &t)?;
                } else {
                    fs::copy(&p, &t)?;
                }
            }
            Ok(())
        }
        copy_dir(temp, out)?;
        fs::write(out.join(".decksmith-output"), "decksmith-output-v1\n")?;
    } else {
        fs::create_dir_all(out.parent().context("Output needs a parent directory")?)?;
        fs::copy(temp, out)?;
    }
    Ok(())
}
pub fn browser_command(
    command: &str,
    directory: &Path,
    slide: Option<&str>,
    out: Option<&Path>,
    format: Option<&str>,
) -> Result<Value> {
    let (root, m) = manifest::load(directory)?;
    if let Some(id) = slide
        && !m.slides.iter().any(|s| s.id == id)
    {
        return Err(Problems(vec![manifest::Finding::error(
            "manifest.unknown_slide",
            None,
            "deck.json",
            format!("Unknown slide ID: {id}"),
        )])
        .into());
    }
    let files = assemble::assemble(&root, &m, false)?;
    let scratch = tempfile::tempdir()?;
    let action = if command == "export" {
        format.unwrap()
    } else {
        command
    };
    let target = match action {
        "render" => Some(output_path(&root, out, ".decksmith/render")?),
        "html" => Some(output_path(&root, out, "dist/html")?),
        "pdf" => Some(output_path(&root, out, "dist/deck.pdf")?),
        _ => None,
    };
    let server = server::LocalServer::start(files.clone(), 0, false)?;
    let mut result = process::helper(
        &root,
        &json!({"action":action,"url":server.url,"manifest":m,"slide":slide,"out":scratch.path()}),
    )?;
    drop(server);
    result["command"] = json!(command);
    if result["success"] != true {
        return Ok(result);
    }
    if let Some(target) = target {
        if action == "html" {
            assemble::write_files(scratch.path(), &files)?;
        }
        let source = if action == "pdf" {
            scratch.path().join("deck.pdf")
        } else {
            scratch.path().to_owned()
        };
        install_output(&source, &target, action != "pdf")?;
        if let Some(artifacts) = result["artifacts"].as_array_mut() {
            for artifact in artifacts {
                let relative = artifact["path"].as_str().unwrap_or("");
                artifact["path"] = json!(if action == "pdf" || action == "html" {
                    target.clone()
                } else {
                    target.join(relative)
                });
            }
        }
    }
    Ok(result)
}
pub fn dev(directory: &Path, port: u16, open: bool) -> Result<()> {
    let (root, m) = manifest::load(directory)?;
    let server = server::LocalServer::start(assemble::assemble(&root, &m, true)?, port, true)?;
    eprintln!("Preview: {} (Ctrl-C to stop)", server.url);
    if open {
        let program = if cfg!(target_os = "macos") {
            "open"
        } else if cfg!(windows) {
            "explorer"
        } else {
            "xdg-open"
        };
        if let Err(e) = Command::new(program).arg(&server.url).status() {
            eprintln!("Could not open browser: {e}. Open {} manually.", server.url);
        }
    }
    while !process::INTERRUPTED.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(500));
        let rebuilt = manifest::load(&root).and_then(|(_, m)| assemble::assemble(&root, &m, true));
        let mut state = server.state.write().unwrap();
        match rebuilt {
            Ok(files) => {
                if state.files != files || state.error.is_some() {
                    state.files = files;
                    state.revision += 1;
                    state.error = None;
                    eprintln!("Rebuilt preview (revision {})", state.revision);
                }
            }
            Err(e) => {
                let text = format!("{e:#}");
                if state.error.as_ref() != Some(&text) {
                    eprintln!("Build error: {text}");
                    state.error = Some(text);
                }
            }
        }
    }
    Ok(())
}
pub fn result_code(value: &Value) -> i32 {
    if value["success"] == true {
        0
    } else if value["error"]["kind"] == "operational" {
        2
    } else {
        1
    }
}
