//! Explicit-save visual editing. Source files remain the only persistent deck model.
pub mod source;
pub mod style;
mod transaction;
mod web;
use crate::{
    assemble::{self, Files},
    manifest::{self, Manifest},
};
use anyhow::{Context, Result, bail, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
pub use web::edit;
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[derive(Clone)]
pub struct Snapshot {
    pub root: PathBuf,
    pub files: Files,
    pub manifest: Manifest,
    pub revision: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Text {
        slide: String,
        element: String,
        html: String,
    },
    Style {
        slide: String,
        element: String,
        property: String,
        value: Option<String>,
    },
    Image {
        slide: String,
        element: String,
        data: String,
    },
    Alt {
        slide: String,
        element: String,
        text: String,
    },
    Reorder {
        order: Vec<String>,
    },
    Notes {
        slide: String,
        text: String,
    },
}
impl Operation {
    fn target(&self) -> Option<(&str, &str)> {
        match self {
            Self::Text { slide, element, .. }
            | Self::Style { slide, element, .. }
            | Self::Image { slide, element, .. }
            | Self::Alt { slide, element, .. } => Some((slide, element)),
            _ => None,
        }
    }
}
impl Snapshot {
    pub fn load(directory: &Path) -> Result<Self> {
        let (root, manifest) = manifest::load(directory)?;
        ensure!(
            !root.join(".decksmith-editor-transaction").exists(),
            "An interrupted editor save needs recovery. Close other editors and run presmith edit --recover, then reopen."
        );
        let mut files = assemble::assemble(&root, &manifest, false)?;
        files.remove("index.html");
        files.insert("deck.json".into(), fs::read(root.join("deck.json"))?);
        for slide in &manifest.slides {
            ensure!(
                slide.source.starts_with("slides/") && slide.source.ends_with(".html"),
                "Editor requires slide sources under slides/ with .html extension: {}",
                slide.source
            );
            ensure!(
                !manifest
                    .slides
                    .iter()
                    .any(|s| s.id != slide.id && s.source == slide.source),
                "Shared slide source is ambiguous: {}",
                slide.source
            );
            files.insert(
                slide.source.clone(),
                fs::read(manifest::safe_file(&root, &slide.source)?)?,
            );
        }
        for path in files.keys() {
            transaction::safe_target(&root, path)?;
        }
        let revision = hash(&serde_json::to_vec(&files)?);
        let s = Self {
            root,
            files,
            manifest,
            revision,
        };
        s.elements()?;
        s.overrides()?;
        Ok(s)
    }
    pub fn elements(&self) -> Result<BTreeMap<String, Vec<source::Element>>> {
        self.manifest
            .slides
            .iter()
            .map(|s| {
                Ok((
                    s.id.clone(),
                    source::parse(std::str::from_utf8(&self.files[&s.source])?)
                        .with_context(|| format!("{} ({})", s.source, s.id))?,
                ))
            })
            .collect()
    }
    pub fn overrides(&self) -> Result<style::Overrides> {
        match self.files.get(style::PATH) {
            Some(b) => style::parse(std::str::from_utf8(b)?),
            None => Ok(style::Overrides::new()),
        }
    }
    pub fn data(&self) -> Result<Value> {
        Ok(
            json!({"manifest":self.manifest,"revision":self.revision,"elements":self.elements()?,"overrides":self.overrides()?}),
        )
    }
    pub fn assembled(&self, annotate: bool) -> Result<Files> {
        let mut fragments = Files::new();
        for s in &self.manifest.slides {
            let mut html = String::from_utf8(self.files[&s.source].clone())?;
            if annotate {
                for n in source::parse(&html)?.into_iter().rev() {
                    source::attribute(&mut html, &n, "data-editor-key", &n.key)?;
                }
            }
            fragments.insert(s.source.clone(), html.into_bytes());
        }
        let mut public = self.files.clone();
        public.remove("deck.json");
        for s in &self.manifest.slides {
            public.remove(&s.source);
        }
        assemble::assemble_sources(&self.manifest, public, &fragments, false)
    }
    /// Replay session commands from the original snapshot. Save does not truncate undo history.
    pub fn apply(&self, operations: &[Operation]) -> Result<Self> {
        ensure!(
            operations.len() <= 10_000,
            "Too many pending operations; reopen after saving"
        );
        let mut next = self.clone();
        let baseline = self.elements()?;
        let targets: HashSet<_> = operations.iter().filter_map(Operation::target).collect();
        // Resolve every target before changing byte positions or inline structure.
        for (slide, element) in &targets {
            ensure!(
                baseline
                    .get(*slide)
                    .is_some_and(|nodes| nodes.iter().any(|n| n.key == *element)),
                "Unknown source target {slide}/{element}"
            );
        }
        for s in &self.manifest.slides {
            let mut html = String::from_utf8(next.files[&s.source].clone())?;
            for n in baseline[&s.id]
                .iter()
                .rev()
                .filter(|n| n.id.is_none() && targets.contains(&(s.id.as_str(), n.key.as_str())))
            {
                source::attribute(&mut html, n, "data-element-id", &n.key)?;
            }
            next.files.insert(s.source.clone(), html.into_bytes());
        }
        let mut raw: Value = serde_json::from_slice(&self.files["deck.json"])?;
        let mut overrides = self.overrides()?;
        let mut styles_changed = false;
        let mut manifest_changed = false;
        for op in operations {
            if let Some((slide, key)) = op.target() {
                let s = self.manifest.slides.iter().find(|s| s.id == slide).unwrap();
                let mut html = String::from_utf8(next.files[&s.source].clone())?;
                let nodes = source::parse(&html)?;
                let n = nodes
                    .iter()
                    .find(|n| n.id.as_deref() == Some(key))
                    .context(
                        "Target was removed by another edit; undo the parent text edit first",
                    )?;
                ensure!(
                    n.read_only.is_none(),
                    "{}",
                    n.read_only.as_deref().unwrap_or("Read-only source")
                );
                match op {
                    Operation::Text { html: value, .. } => {
                        ensure!(
                            n.text_editable,
                            "Unsupported nested content; edit this element in source"
                        );
                        source::inline(value)?;
                        html.replace_range(n.open_end..n.close_start, value);
                    }
                    Operation::Style {
                        property, value, ..
                    } => {
                        if let Some(value) = value {
                            style::validate(property, value)?;
                            ensure!(
                                !style::inline_conflict(
                                    n.attrs.get("style").map(String::as_str).unwrap_or(""),
                                    property
                                ),
                                "{property} is controlled by an inline style; edit or remove it in source first"
                            );
                        } else {
                            style::validate(
                                property,
                                match property.as_str() {
                                    "color" | "background-color" | "border-color" => "#000",
                                    "font-family" => "sans-serif",
                                    "font-weight" => "400",
                                    "font-style" => "normal",
                                    "text-align" => "left",
                                    "text-decoration-line" | "border-style" => "none",
                                    "object-fit" => "contain",
                                    "align-items" | "align-self" | "justify-content" => "center",
                                    "flex-direction" => "row",
                                    "order" => "0",
                                    _ => "1px",
                                },
                            )?;
                        }
                        let props = overrides
                            .entry(slide.into())
                            .or_default()
                            .entry(key.into())
                            .or_default();
                        if let Some(v) = value {
                            props.insert(property.clone(), v.clone());
                        } else {
                            props.remove(property);
                        }
                        styles_changed = true;
                    }
                    Operation::Alt { text, .. } => {
                        ensure!(n.tag == "img", "Alternative text requires an image");
                        ensure!(text.len() <= 10_000, "Alternative text too long");
                        source::attribute(&mut html, n, "alt", text)?;
                    }
                    Operation::Image { data, .. } => {
                        ensure!(n.tag == "img", "Image replacement requires an image");
                        ensure!(
                            !n.attrs.contains_key("srcset") && !n.attrs.contains_key("sizes"),
                            "Responsive images must be edited in source"
                        );
                        let (path, bytes) = image_asset(data)?;
                        if let Some(existing) = next.files.get(&path) {
                            ensure!(existing == &bytes, "Asset hash collision at {path}");
                        }
                        next.files.insert(path.clone(), bytes);
                        source::attribute(&mut html, n, "src", &path)?;
                    }
                    _ => unreachable!(),
                }
                next.files.insert(s.source.clone(), html.into_bytes());
            } else {
                match op {
                    Operation::Reorder { order } => {
                        let expected: HashSet<_> =
                            self.manifest.slides.iter().map(|s| s.id.as_str()).collect();
                        let got: HashSet<_> = order.iter().map(String::as_str).collect();
                        ensure!(
                            order.len() == expected.len() && got == expected,
                            "Reorder must contain every slide exactly once"
                        );
                        let slides = raw["slides"].as_array().unwrap();
                        raw["slides"] = Value::Array(
                            order
                                .iter()
                                .map(|id| slides.iter().find(|s| s["id"] == *id).unwrap().clone())
                                .collect(),
                        );
                        manifest_changed = true;
                    }
                    Operation::Notes { slide, text } => {
                        ensure!(text.len() <= 100_000, "Notes exceed 100 KB");
                        let s = raw["slides"]
                            .as_array_mut()
                            .unwrap()
                            .iter_mut()
                            .find(|s| s["id"] == *slide)
                            .context("Unknown slide")?;
                        s["notes"] = json!(text);
                        manifest_changed = true;
                    }
                    _ => unreachable!(),
                }
            }
        }
        if styles_changed {
            next.files
                .insert(style::PATH.into(), style::render(&overrides).into_bytes());
            let styles = raw["styles"].as_array_mut().unwrap();
            styles.retain(|p| p != style::PATH);
            styles.push(json!(style::PATH));
            manifest_changed = true;
        }
        if manifest_changed {
            let mut bytes = serde_json::to_vec_pretty(&raw)?;
            bytes.push(b'\n');
            next.files.insert("deck.json".into(), bytes);
        }
        next.manifest = serde_json::from_value(raw)?;
        next.elements()?;
        next.assembled(false)?;
        Ok(next)
    }
    pub fn save(&self, expected: &str, operations: &[Operation]) -> Result<String> {
        let current = Self::load(&self.root).context("CONFLICT: Current source cannot be loaded; repair the external change before reloading")?;
        ensure!(
            current.revision == expected,
            "CONFLICT: Project changed outside this editor. Download pending edits, then reload the latest source."
        );
        let next = self.apply(operations)?;
        // Recheck immediately before entering the transaction, including images and authored styles.
        ensure!(
            Self::load(&self.root)?.revision == expected,
            "CONFLICT: Source changed during validation"
        );
        transaction::commit(&self.root, &current.files, &next.files, None)?;
        Ok(Self::load(&self.root)?.revision)
    }
}
fn image_asset(data: &str) -> Result<(String, Vec<u8>)> {
    ensure!(data.len() <= 14_000_000, "Image exceeds 10 MB");
    let (prefix, encoded) = data
        .split_once(',')
        .context("Expected a base64 image data URL")?;
    let ext = match prefix {
        "data:image/png;base64" => "png",
        "data:image/jpeg;base64" => "jpg",
        "data:image/webp;base64" => "webp",
        "data:image/gif;base64" => "gif",
        _ => bail!("Use PNG, JPEG, WebP or GIF; SVG uploads are not supported"),
    };
    let bytes = STANDARD.decode(encoded)?;
    ensure!(bytes.len() <= 10_000_000, "Image exceeds 10 MB");
    let mut reader = image::ImageReader::new(std::io::Cursor::new(&bytes)).with_guessed_format()?;
    let format = reader.format().context("Invalid image")?;
    ensure!(
        matches!(
            (ext, format),
            ("png", image::ImageFormat::Png)
                | ("jpg", image::ImageFormat::Jpeg)
                | ("webp", image::ImageFormat::WebP)
                | ("gif", image::ImageFormat::Gif)
        ),
        "Image MIME type does not match its bytes"
    );
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128_000_000);
    reader.limits(limits);
    reader.decode().context("Invalid or oversized image")?;
    Ok((format!("assets/editor-{}.{}", hash(&bytes), ext), bytes))
}
