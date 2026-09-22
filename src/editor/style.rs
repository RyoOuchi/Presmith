use anyhow::{Result, bail, ensure};
use std::collections::BTreeMap;
pub const PATH: &str = "styles/editor.css";
pub const HEADER: &str =
    "/* Decksmith GUI overrides. Reset a property in the editor to reveal authored CSS. */\n";
pub type Overrides = BTreeMap<String, BTreeMap<String, BTreeMap<String, String>>>;
pub fn validate(property: &str, value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value.len() < 200
            && !value.contains([';', '{', '}', '!', '\\', '\n', '\r', '<', '>']),
        "Unsafe or empty CSS value"
    );
    let choices: Option<&[&str]> = match property {
        "font-weight" => Some(&[
            "100", "200", "300", "400", "500", "600", "650", "700", "800", "900", "normal", "bold",
        ]),
        "font-style" => Some(&["normal", "italic"]),
        "text-decoration-line" => Some(&["none", "underline", "line-through"]),
        "text-align" => Some(&["left", "center", "right", "justify", "start", "end"]),
        "object-fit" => Some(&["contain", "cover", "fill", "none", "scale-down"]),
        "align-items" | "align-self" => Some(&[
            "normal",
            "stretch",
            "start",
            "end",
            "center",
            "baseline",
            "flex-start",
            "flex-end",
            "auto",
        ]),
        "justify-content" => Some(&[
            "normal",
            "start",
            "end",
            "center",
            "space-between",
            "space-around",
            "space-evenly",
            "flex-start",
            "flex-end",
        ]),
        "flex-direction" => Some(&["row", "column", "row-reverse", "column-reverse"]),
        _ => None,
    };
    if let Some(choices) = choices {
        ensure!(choices.contains(&value), "Unsupported {property} value");
        return Ok(());
    }
    match property {
        "color" | "background-color" | "border-color" => ensure!(
            value == "transparent"
                || (value.starts_with('#')
                    && [4, 5, 7, 9].contains(&value.len())
                    && value[1..].bytes().all(|c| c.is_ascii_hexdigit())),
            "Use a hexadecimal color or transparent"
        ),
        "font-family" => {
            for family in value.split(',') {
                let family = family.trim();
                ensure!(!family.is_empty(), "Font family names cannot be empty");
                let quoted = family.starts_with(['\"', '\'']);
                let name = if quoted {
                    ensure!(
                        family.len() > 2 && family.as_bytes().first() == family.as_bytes().last(),
                        "Unbalanced font family quotes"
                    );
                    &family[1..family.len() - 1]
                } else {
                    ensure!(
                        !family.starts_with(|c: char| c.is_ascii_digit()),
                        "Quote font family names that begin with a number"
                    );
                    family
                };
                ensure!(
                    !name.trim().is_empty()
                        && name
                            .chars()
                            .all(|c| c.is_alphanumeric() || " -".contains(c)),
                    "Unsupported font family"
                );
            }
        }
        "order" => {
            let n: i32 = value.parse()?;
            ensure!((-1000..=1000).contains(&n), "Order out of range");
        }
        "font-size" | "width" | "height" | "padding" | "gap" | "row-gap" | "column-gap"
        | "border-width" | "border-radius" | "left" | "top" => {
            let n: f64 = value
                .strip_suffix("px")
                .ok_or_else(|| anyhow::anyhow!("{property} requires px"))?
                .parse()?;
            ensure!(
                n.is_finite()
                    && n <= 8192.
                    && n >= if matches!(property, "left" | "top") {
                        -8192.
                    } else if matches!(property, "width" | "height" | "font-size") {
                        1.
                    } else {
                        0.
                    },
                "Invalid {property} dimension"
            );
        }
        "border-style" => ensure!(
            ["none", "solid", "dashed", "dotted", "double"].contains(&value),
            "Invalid border style"
        ),
        _ => bail!("Unsupported style property: {property}"),
    }
    Ok(())
}
pub fn selector(slide: &str, element: &str) -> String {
    format!("[data-slide-id=\"{slide}\"] [data-element-id=\"{element}\"]")
}
pub fn render(overrides: &Overrides) -> String {
    let mut css = HEADER.to_owned();
    for (s, elements) in overrides {
        for (e, props) in elements {
            if props.is_empty() {
                continue;
            }
            css += &format!("{} {{\n", selector(s, e));
            for (p, v) in props {
                css += &format!("  {p}: {v};\n");
            }
            css += "}\n";
        }
    }
    css
}
pub fn parse(css: &str) -> Result<Overrides> {
    ensure!(
        css.starts_with(HEADER),
        "styles/editor.css is not a Presmith override file; rename it and update deck.json before using GUI styles"
    );
    let mut result = Overrides::new();
    for rule in css[HEADER.len()..].split('}') {
        if rule.trim().is_empty() {
            continue;
        }
        let (sel, body) = rule
            .split_once('{')
            .ok_or_else(|| anyhow::anyhow!("Invalid editor.css rule"))?;
        let sel = sel.trim();
        let parts: Vec<_> = sel.split('"').collect();
        ensure!(
            parts.len() == 5
                && sel == selector(parts[1], parts[3])
                && super::source::valid_id(parts[1])
                && super::source::valid_id(parts[3]),
            "Unsupported editor.css selector"
        );
        let props = result
            .entry(parts[1].into())
            .or_default()
            .entry(parts[3].into())
            .or_default();
        for declaration in body.split(';') {
            if declaration.trim().is_empty() {
                continue;
            }
            let (p, v) = declaration
                .split_once(':')
                .ok_or_else(|| anyhow::anyhow!("Invalid CSS declaration"))?;
            validate(p.trim(), v.trim())?;
            ensure!(
                props.insert(p.trim().into(), v.trim().into()).is_none(),
                "Duplicate GUI declaration"
            );
        }
    }
    Ok(result)
}
pub fn inline_conflict(style: &str, property: &str) -> bool {
    style
        .split(';')
        .filter_map(|s| s.split_once(':'))
        .any(|(p, _)| {
            let p = p.trim().to_lowercase();
            p == property
                || p == "all"
                || (p == "font" && property.starts_with("font-"))
                || ([
                    "border",
                    "padding",
                    "background",
                    "flex",
                    "gap",
                    "inset",
                    "text-decoration",
                ]
                .contains(&p.as_str())
                    && (property.starts_with(&format!("{p}-"))
                        || (p == "inset" && ["left", "top"].contains(&property))
                        || (p == "background" && property == "background-color")))
        })
}
