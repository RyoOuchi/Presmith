pub mod assemble;
pub mod editor;
pub mod manifest;
pub mod process;
mod renderer_cache;
pub mod server;
pub mod skill;
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
        "Created {} with the Codex skill in .agents/skills/presmith/. No rendering dependencies installed.\nOpen this deck in Codex and use $presmith.\nNext steps:\n  cd '{}'\n  presmith setup\n  presmith doctor\n  presmith dev --open\n  presmith check\n  presmith render\n  presmith export --format html\n  presmith export --format pdf\n  presmith export --format pptx",
        directory.display(),
        directory.display()
    );
    Ok(())
}
pub fn setup(directory: &Path) -> Result<()> {
    setup_with_upgrade(directory, false)
}
/// Refresh only bundled renderer files. Preserve a backup of every replaced file,
/// custom renderer files, installed dependencies, and all authored deck sources.
pub fn upgrade_renderer(root: &Path) -> Result<PathBuf> {
    let root = root.canonicalize()?;
    for relative in [
        ".decksmith",
        ".decksmith/renderer-backups",
        "tooling",
        "tooling/renderer",
    ] {
        if root.join(relative).is_symlink() {
            bail!("Refusing to upgrade through a symlink: {relative}");
        }
    }
    let backup_root = root.join(".decksmith/renderer-backups");
    fs::create_dir_all(&backup_root)?;
    if !backup_root.canonicalize()?.starts_with(&root) {
        bail!("Renderer backup directory must remain inside the deck");
    }
    let backup = tempfile::Builder::new()
        .prefix("upgrade-")
        .tempdir_in(backup_root)?
        .keep();
    let renderer = root.join("tooling/renderer");
    fs::create_dir_all(&renderer)?;
    if !renderer.canonicalize()?.starts_with(&root) {
        bail!("Renderer directory must remain inside the deck");
    }
    // Back up all replacements before changing any renderer files.
    for (name, _) in ASSETS {
        if let Some(relative) = name.strip_prefix("tooling/renderer/") {
            let existing = renderer.join(relative);
            if existing.is_symlink() {
                bail!(
                    "Refusing to replace symlink renderer asset: {}",
                    existing.display()
                );
            }
            if existing.exists() {
                if !existing.is_file() {
                    bail!(
                        "Refusing to replace non-file renderer asset: {}",
                        existing.display()
                    );
                }
                let target = backup.join(relative);
                fs::create_dir_all(target.parent().unwrap())?;
                fs::copy(existing, target)?;
            }
        }
    }
    for (name, data) in ASSETS {
        if let Some(relative) = name.strip_prefix("tooling/renderer/") {
            let target = renderer.join(relative);
            fs::create_dir_all(target.parent().unwrap())?;
            fs::write(target, data)?;
        }
    }
    Ok(backup)
}
pub fn setup_with_upgrade(directory: &Path, upgrade: bool) -> Result<()> {
    setup_with_options(directory, upgrade, false)
}

pub fn setup_with_options(directory: &Path, upgrade: bool, local: bool) -> Result<()> {
    let (root, _) = manifest::load(directory)?;
    if upgrade {
        let backup = upgrade_renderer(&root)?;
        eprintln!(
            "Updated renderer; previous files backed up to {}",
            backup.display()
        );
    }
    let probe = renderer_cache::setup(&root, local)?;
    if probe["capabilities"]["pptx_export"] == true {
        eprintln!("Ready: preview, checks, screenshots, HTML, PDF and PPTX export.");
    } else {
        eprintln!(
            "Ready: preview, checks, screenshots, HTML and PDF export. For PPTX, run presmith setup --upgrade-renderer."
        );
    }
    Ok(())
}
pub fn envelope(command: &str) -> Value {
    json!({"schema_version":1,"command":command,"success":true,"findings":[],"artifacts":[],"error":null})
}
pub fn doctor(directory: &Path) -> (Value, i32) {
    let mut result = envelope("doctor");
    let loaded = manifest::load(directory);
    let mut capabilities = json!({"preview":false,"check":false,"render":false,"html_export":false,"pdf_export":false,"pptx_export":false});
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
                    capabilities["pptx_export"] = json!(v["capabilities"]["pptx_export"] == true);
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
    if format == Some("pptx") && !root.join("tooling/renderer/pptx.mjs").is_file() {
        bail!(
            "This deck's renderer predates PPTX export. Run presmith setup --upgrade-renderer in the deck directory; old renderer files will be backed up."
        );
    }
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
        "pptx" => Some(output_path(&root, out, "dist/deck.pptx")?),
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
        let single_file = action == "pdf" || action == "pptx";
        let source = if single_file {
            scratch.path().join(format!("deck.{action}"))
        } else {
            scratch.path().to_owned()
        };
        install_output(&source, &target, !single_file)?;
        if let Some(artifacts) = result["artifacts"].as_array_mut() {
            for artifact in artifacts {
                let relative = artifact["path"].as_str().unwrap_or("");
                artifact["path"] = json!(if single_file || action == "html" {
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
