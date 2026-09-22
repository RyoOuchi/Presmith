use crate::manifest::{Manifest, safe_file};
use anyhow::{Context, Result, bail};
use std::{collections::BTreeMap, fs, path::Path};
pub type Files = BTreeMap<String, Vec<u8>>;
pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn walk(root: &Path, dir: &Path, files: &mut Files) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for item in fs::read_dir(dir)? {
        let path = item?.path();
        let name = path.file_name().unwrap().to_string_lossy();
        if name.starts_with('.') || name == "node_modules" {
            continue;
        }
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            bail!("Symlinks are not served or exported: {}", path.display());
        }
        if path.is_dir() {
            walk(root, &path, files)?;
        } else if path.is_file() {
            let key = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            files.insert(key, fs::read(path)?);
        }
    }
    Ok(())
}
pub fn assemble(root: &Path, m: &Manifest, dev: bool) -> Result<Files> {
    let mut files = Files::new();
    for name in ["assets", "styles", "scripts", "lib"] {
        let path = root.join(name);
        if path.exists() && fs::symlink_metadata(&path)?.file_type().is_symlink() {
            bail!("Presentation directory cannot be a symlink: {name}");
        }
        walk(root, &path, &mut files)?;
    }
    let mut fragments = Files::new();
    for slide in &m.slides {
        fragments.insert(
            slide.source.clone(),
            fs::read(safe_file(root, &slide.source)?)?,
        );
    }
    assemble_sources(m, files, &fragments, dev)
}

/// Shared assembly for disk projects and validated editor snapshots.
pub fn assemble_sources(
    m: &Manifest,
    mut files: Files,
    fragments: &Files,
    dev: bool,
) -> Result<Files> {
    let data = serde_json::to_string(m)?
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    let mut html = format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><link rel=\"icon\" href=\"data:,\"><link rel=\"stylesheet\" href=\"lib/decksmith.css\">\n",
        escape(&m.title)
    );
    for style in &m.styles {
        if style != "lib/decksmith.css" {
            html += &format!("<link rel=\"stylesheet\" href=\"{}\">\n", escape(style));
        }
    }
    html += &format!(
        "<style>:root{{--slide-width:{}px;--slide-height:{}px}}@page{{size:{}px {}px;margin:0}}</style></head><body><main id=\"deck-stage\" aria-label=\"{}\">\n",
        m.width,
        m.height,
        m.width,
        m.height,
        escape(&m.title)
    );
    for (index, s) in m.slides.iter().enumerate() {
        let fragment = std::str::from_utf8(
            fragments
                .get(&s.source)
                .with_context(|| format!("Missing fragment {}", s.source))?,
        )?;
        html += &format!(
            "<section class=\"deck-slide\" id=\"slide-{}\" data-slide-id=\"{}\" data-source=\"{}\" aria-label=\"{}\" aria-roledescription=\"slide\" hidden inert>\n{}\n<footer class=\"slide-footer\"><span>{}</span><span>{:02} / {:02}</span></footer></section>\n",
            escape(&s.id),
            escape(&s.id),
            escape(&s.source),
            escape(s.title.as_deref().unwrap_or(&s.id)),
            fragment,
            escape(&m.title),
            index + 1,
            m.slides.len()
        );
    }
    html += "</main><nav id=\"deck-controls\" aria-label=\"Presentation controls\"><button id=\"deck-prev\" aria-label=\"Previous slide\">←</button><output id=\"deck-position\" aria-live=\"polite\"></output><button id=\"deck-next\" aria-label=\"Next slide\">→</button><button id=\"deck-fullscreen\">Fullscreen</button></nav>";
    html += &format!(
        "<script type=\"application/json\" id=\"deck-manifest\">{data}</script><script src=\"lib/decksmith.js\"></script>\n"
    );
    for script in &m.scripts {
        if script != "lib/decksmith.js" {
            html += &format!("<script src=\"{}\"></script>\n", escape(script));
        }
    }
    if dev {
        html += include_str!("dev-client.html");
    }
    html += "</body></html>";
    files.insert("index.html".into(), html.into_bytes());
    Ok(files)
}
pub fn write_files(path: &Path, files: &Files) -> Result<()> {
    fs::create_dir_all(path)?;
    for (name, data) in files {
        let target = path.join(name);
        fs::create_dir_all(target.parent().unwrap())?;
        fs::write(target, data)?;
    }
    Ok(())
}
