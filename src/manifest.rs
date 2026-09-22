use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, Value>,
    pub schema_version: u32,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub slides: Vec<Slide>,
    pub styles: Vec<String>,
    pub scripts: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Slide {
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, Value>,
    pub id: String,
    pub source: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Finding {
    pub severity: String,
    pub rule_id: String,
    pub slide_id: Option<String>,
    pub element_id: Option<String>,
    pub source: Option<String>,
    pub message: String,
    pub measurements: Value,
}
impl Finding {
    pub fn error(rule: &str, slide: Option<&Slide>, source: &str, message: String) -> Self {
        Self {
            severity: "error".into(),
            rule_id: rule.into(),
            slide_id: slide.map(|s| s.id.clone()),
            element_id: None,
            source: Some(source.into()),
            message,
            measurements: json!({}),
        }
    }
}
#[derive(Debug)]
pub struct Problems(pub Vec<Finding>);
impl std::fmt::Display for Problems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Deck validation failed: {}",
            self.0
                .iter()
                .map(|x| x.message.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        )
    }
}
impl std::error::Error for Problems {}

pub fn relative_path(value: &str) -> Result<&Path> {
    let p = Path::new(value);
    if value.is_empty()
        || value.contains(['\\', '?', '#', ':', '\0', '%'])
        || p.components().any(|c| !matches!(c, Component::Normal(_)))
        || value.split('/').any(|s| s.starts_with('.') || s.is_empty())
    {
        bail!("Path must be a plain relative project path (no traversal or URL): {value}");
    }
    Ok(p)
}
pub fn safe_file(root: &Path, value: &str) -> Result<PathBuf> {
    let p = root.join(relative_path(value)?);
    let canonical = p
        .canonicalize()
        .with_context(|| format!("Missing file: {value}"))?;
    if !canonical.starts_with(root) || !canonical.is_file() {
        bail!("File escapes project root or is not a file: {value}");
    }
    Ok(canonical)
}
pub fn load(directory: &Path) -> Result<(PathBuf, Manifest)> {
    let root = directory
        .canonicalize()
        .context("Project directory does not exist; run presmith init <directory>")?;
    let raw = fs::read_to_string(root.join("deck.json")).map_err(|e| {
        Problems(vec![Finding::error(
            "manifest.invalid",
            None,
            "deck.json",
            format!("Cannot read deck.json: {e}"),
        )])
    })?;
    let m: Manifest = serde_json::from_str(&raw).map_err(|e| {
        Problems(vec![Finding::error(
            "manifest.invalid",
            None,
            "deck.json",
            e.to_string(),
        )])
    })?;
    let mut findings = Vec::new();
    if m.schema_version != 1
        || m.title.trim().is_empty()
        || !(320..=4096).contains(&m.width)
        || !(240..=4096).contains(&m.height)
        || m.slides.is_empty()
        || m.slides.len() > 200
    {
        findings.push(Finding::error(
            "manifest.invalid",
            None,
            "deck.json",
            "Require schema_version 1, a title, width 320–4096, height 240–4096, and 1–200 slides"
                .into(),
        ));
    }
    let mut ids = HashSet::new();
    for slide in &m.slides {
        if slide.id.is_empty()
            || slide.id.len() > 80
            || !slide
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
        {
            findings.push(Finding::error(
                "manifest.invalid_id",
                Some(slide),
                "deck.json",
                "Slide ID must contain 1–80 ASCII letters, digits, hyphens or underscores".into(),
            ));
        }
        if !ids.insert(slide.id.to_ascii_lowercase()) {
            findings.push(Finding::error(
                "manifest.duplicate_id",
                Some(slide),
                "deck.json",
                format!("Duplicate slide ID (case-insensitive): {}", slide.id),
            ));
        }
        if let Err(e) = safe_file(&root, &slide.source) {
            findings.push(Finding::error(
                "manifest.source",
                Some(slide),
                &slide.source,
                e.to_string(),
            ));
        }
    }
    for (paths, allowed) in [
        (&m.styles, ["styles", "lib", "assets"]),
        (&m.scripts, ["scripts", "lib", "assets"]),
    ] {
        for path in paths {
            let first = path.split('/').next().unwrap_or("");
            if !allowed.contains(&first) {
                findings.push(Finding::error(
                    "manifest.path",
                    None,
                    path,
                    format!("Shared resource must be in {}: {path}", allowed.join(", ")),
                ));
            }
            if let Err(e) = safe_file(&root, path) {
                findings.push(Finding::error("manifest.path", None, path, e.to_string()));
            }
        }
    }
    for required in ["lib/decksmith.css", "lib/decksmith.js"] {
        if let Err(e) = safe_file(&root, required) {
            findings.push(Finding::error(
                "manifest.runtime",
                None,
                required,
                e.to_string(),
            ));
        }
    }
    if findings.is_empty() {
        Ok((root, m))
    } else {
        Err(Problems(findings).into())
    }
}
