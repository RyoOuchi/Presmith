//! Tokenizer spans let us patch authored bytes without round-tripping a browser DOM.
use super::hash;
use anyhow::{Result, bail, ensure};
use html5gum::{DefaultEmitter, Token, Tokenizer};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Debug, Serialize)]
pub struct Element {
    pub key: String,
    pub tag: String,
    pub id: Option<String>,
    pub html: String,
    pub text_editable: bool,
    pub read_only: Option<String>,
    pub attrs: BTreeMap<String, String>,
    #[serde(skip)]
    pub start: usize,
    #[serde(skip)]
    pub open_end: usize,
    #[serde(skip)]
    pub close_start: usize,
    #[serde(skip)]
    pub end: usize,
}
fn void(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
fn editable(tag: &str) -> bool {
    matches!(
        tag,
        "div"
            | "section"
            | "article"
            | "header"
            | "footer"
            | "aside"
            | "figure"
            | "figcaption"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "p"
            | "span"
            | "strong"
            | "em"
            | "b"
            | "i"
            | "u"
            | "s"
            | "code"
            | "pre"
            | "ul"
            | "ol"
            | "li"
            | "img"
            | "blockquote"
    )
}
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 100
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
}
pub fn parse(source: &str) -> Result<Vec<Element>> {
    let mut emitter = DefaultEmitter::<usize>::new_with_span();
    emitter.naively_switch_states(true);
    let mut nodes: Vec<Element> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut ids = HashSet::new();
    for token in Tokenizer::new_with_emitter(source, emitter) {
        match token? {
            Token::StartTag(t) => {
                let tag = String::from_utf8(t.name.to_vec())?;
                ensure!(
                    !matches!(tag.as_str(), "html" | "head" | "body"),
                    "Slide fragments cannot contain document wrappers"
                );
                let attrs: BTreeMap<String, String> = t
                    .attributes
                    .into_iter()
                    .map(|(k, v)| {
                        Ok((
                            String::from_utf8(k.to_vec())?,
                            String::from_utf8(v.to_vec())?,
                        ))
                    })
                    .collect::<Result<_>>()?;
                ensure!(
                    !attrs.contains_key("data-editor-key") && !attrs.contains_key("data-slide-id"),
                    "Reserved editor/slide attribute in fragment"
                );
                let id = attrs.get("data-element-id").cloned();
                if let Some(id) = &id {
                    ensure!(
                        valid_id(id),
                        "Element ID must use ASCII letters, digits, - or _: {id}"
                    );
                    ensure!(
                        ids.insert(id.clone()),
                        "Ambiguous duplicate element ID: {id}"
                    );
                }
                let key = id.clone().unwrap_or_else(|| {
                    format!(
                        "editor-{}",
                        &hash(format!("{}:{}", hash(source.as_bytes()), t.span.start).as_bytes())
                            [..16]
                    )
                });
                let index = nodes.len();
                nodes.push(Element {
                    key,
                    tag: tag.clone(),
                    id,
                    html: String::new(),
                    text_editable: false,
                    read_only: None,
                    attrs,
                    start: t.span.start,
                    open_end: t.span.end,
                    close_start: t.span.end,
                    end: t.span.end,
                });
                if !void(&tag) && !t.self_closing {
                    stack.push(index);
                }
            }
            Token::EndTag(t) => {
                let tag = String::from_utf8(t.name.to_vec())?;
                let Some(index) = stack.pop() else {
                    bail!(
                        "Unmatched closing tag </{tag}>; use explicitly balanced HTML for editing"
                    )
                };
                ensure!(
                    nodes[index].tag == tag,
                    "Unbalanced HTML near </{tag}>; explicitly close each element before editing"
                );
                nodes[index].close_start = t.span.start;
                nodes[index].end = t.span.end;
            }
            Token::Error(e) => bail!("Invalid HTML near byte {}: {:?}", e.span.start, e.value),
            _ => {}
        }
    }
    ensure!(
        stack.is_empty(),
        "Unclosed HTML element; explicitly close tags before editing"
    );
    // Fragment scripts and event attributes can run before the source bridge captures
    // parser-created node identities. Render them, but refuse ambiguous source writes.
    let early_script = nodes.iter().any(|n| {
        n.tag == "script"
            && !matches!(
                n.attrs.get("type").map(String::as_str),
                Some("application/json" | "application/ld+json")
            )
            || n.attrs.keys().any(|a| a.starts_with("on"))
    });
    for n in &mut nodes {
        if early_script {
            n.read_only = Some("This slide has a fragment script or inline event handler that runs before source mapping. Move initialization to a deck.json script with Decksmith.register to enable element editing.".into());
        }

        n.html = source[n.open_end..n.close_start].to_owned();
        n.text_editable = n.tag != "img" && editable(&n.tag) && inline(&n.html).is_ok();
    }
    // Inert/template and foreign content cannot be reliably mapped to live HTML.
    let excluded: Vec<_> = nodes
        .iter()
        .filter(|n| {
            matches!(
                n.tag.as_str(),
                "script"
                    | "style"
                    | "template"
                    | "svg"
                    | "math"
                    | "textarea"
                    | "select"
                    | "button"
                    | "a"
            )
        })
        .map(|n| (n.start, n.end))
        .collect();
    Ok(nodes
        .into_iter()
        .filter(|n| editable(&n.tag) && !excluded.iter().any(|(s, e)| n.start >= *s && n.end <= *e))
        .collect())
}
/// Only a small, attribute-free inline vocabulary is accepted, never arbitrary HTML.
pub fn inline(source: &str) -> Result<()> {
    ensure!(source.len() <= 100_000, "Text exceeds 100 KB");
    let mut stack = Vec::new();
    for t in Tokenizer::new(source) {
        match t? {
            Token::StartTag(t) => {
                let tag = String::from_utf8(t.name.to_vec())?;
                ensure!(
                    matches!(
                        tag.as_str(),
                        "strong" | "em" | "b" | "i" | "u" | "s" | "code" | "br"
                    ) && t.attributes.is_empty(),
                    "Text supports only attribute-free strong, em, b, i, u, s, code and br; nested content is read-only"
                );
                if tag != "br" {
                    stack.push(tag);
                }
            }
            Token::EndTag(t) => ensure!(
                stack.pop().as_deref() == Some(std::str::from_utf8(&t.name)?),
                "Unbalanced inline formatting"
            ),
            Token::String(_) => {}
            _ => {
                bail!("Comments, scripts and unsupported markup are read-only; edit source instead")
            }
        }
    }
    ensure!(stack.is_empty(), "Unclosed inline formatting");
    Ok(())
}
pub fn attribute(source: &mut String, node: &Element, name: &str, value: &str) -> Result<()> {
    // Locate attributes only within the parser-verified start tag. Preserve every other byte.
    let raw = &source[node.start..node.open_end];
    let bytes = raw.as_bytes();
    let mut i = 1;
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
        i += 1;
    }
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || b">/".contains(&bytes[i]) {
            break;
        }
        let start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && !b"=>/".contains(&bytes[i]) {
            i += 1;
        }
        let attr = &raw[start..i];
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && b"\"'".contains(&bytes[i]) {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    i += 1;
                }
                i += 1;
            } else {
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                    i += 1;
                }
            }
        }
        if attr.eq_ignore_ascii_case(name) {
            source.replace_range(
                node.start + start..node.start + i,
                &format!("{name}=\"{}\"", crate::assemble::escape(value)),
            );
            return Ok(());
        }
    }
    let pos = node.open_end - if raw.ends_with("/>") { 2 } else { 1 };
    source.insert_str(
        pos,
        &format!(" {name}=\"{}\"", crate::assemble::escape(value)),
    );
    Ok(())
}
