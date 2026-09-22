use presmith::editor::{Operation, Snapshot, source, style};
use serde_json::json;
use std::fs;
fn fresh() -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    presmith::init(t.path()).unwrap();
    t
}
fn text(value: &str) -> Operation {
    Operation::Text {
        slide: "intro".into(),
        element: "headline".into(),
        html: value.into(),
    }
}
fn style_op(p: &str, v: Option<&str>) -> Operation {
    Operation::Style {
        slide: "intro".into(),
        element: "headline".into(),
        property: p.into(),
        value: v.map(str::to_owned),
    }
}
#[test]
fn open_is_read_only_and_source_patches_preserve_bytes() {
    let t = fresh();
    let before = Snapshot::load(t.path()).unwrap();
    assert_eq!(before.files, Snapshot::load(t.path()).unwrap().files);
    let content = "日本語 <strong>clear</strong><br>مرحبا &amp; 🌏";
    let operations = vec![
        text(content),
        style_op("font-size", Some("64px")),
        style_op("color", Some("#ff0000")),
    ];
    let revision = before.save(&before.revision, &operations).unwrap();
    let after = Snapshot::load(t.path()).unwrap();
    assert_eq!(after.revision, revision);
    let old = String::from_utf8(before.files["slides/intro.html"].clone()).unwrap();
    let expected = old.replace("Good ideas deserve<br>a <em>clear story.</em>", content);
    assert_eq!(after.files["slides/intro.html"], expected.as_bytes());
    for p in [
        "slides/workflow.html",
        "slides/next.html",
        "scripts/custom.js",
        "styles/theme.css",
        "styles/custom.css",
    ] {
        assert_eq!(before.files[p], after.files[p], "{p}");
    }
    assert_eq!(
        after.overrides().unwrap()["intro"]["headline"]["font-size"],
        "64px"
    );
    assert_eq!(after.manifest.styles.last().unwrap(), style::PATH);
}
#[test]
fn repeated_save_reset_and_undo_after_save() {
    let t = fresh();
    let base = Snapshot::load(t.path()).unwrap();
    let ops = vec![
        text("First <em>edit</em>"),
        style_op("font-size", Some("60px")),
        style_op("font-size", Some("64px")),
    ];
    let r = base.save(&base.revision, &ops).unwrap();
    let css = fs::read_to_string(t.path().join(style::PATH)).unwrap();
    assert_eq!(css.matches("font-size:").count(), 1);
    let mut reset = ops.clone();
    reset.push(style_op("font-size", None));
    let r = base.save(&r, &reset).unwrap();
    assert!(
        !fs::read_to_string(t.path().join(style::PATH))
            .unwrap()
            .contains("font-size:")
    );
    base.save(&r, &[]).unwrap();
    assert_eq!(base.files, Snapshot::load(t.path()).unwrap().files);
}
#[test]
fn ids_only_on_edit_and_nested_targets() {
    let t = fresh();
    let path = t.path().join("slides/intro.html");
    fs::write(
        &path,
        "<!-- untouched -->\n<div class='box'><h1>Hello <em>world</em></h1><p>Keep me</p></div>\n",
    )
    .unwrap();
    let base = Snapshot::load(t.path()).unwrap();
    let nodes = base.elements().unwrap();
    let h = nodes["intro"].iter().find(|n| n.tag == "h1").unwrap();
    assert!(h.id.is_none());
    let op = Operation::Text {
        slide: "intro".into(),
        element: h.key.clone(),
        html: "New <em>world</em>".into(),
    };
    base.save(&base.revision, &[op]).unwrap();
    let source = fs::read_to_string(path).unwrap();
    assert_eq!(source.matches("data-element-id").count(), 1);
    assert!(source.contains("<!-- untouched -->\n<div class='box'><h1 data-element-id="));
    assert!(source.contains("<p>Keep me</p>"));
    assert_eq!(
        Snapshot::load(t.path()).unwrap().elements().unwrap()["intro"]
            .iter()
            .find(|n| n.tag == "h1")
            .unwrap()
            .key,
        h.key
    );
}
#[test]
fn conflicts_do_not_overwrite_anything() {
    let t = fresh();
    let base = Snapshot::load(t.path()).unwrap();
    fs::write(t.path().join("slides/next.html"), "<p>External edit</p>").unwrap();
    let latest = Snapshot::load(t.path()).unwrap();
    let err = base.save(&base.revision, &[text("Oops")]).unwrap_err();
    assert!(err.to_string().contains("CONFLICT"));
    assert_eq!(latest.files, Snapshot::load(t.path()).unwrap().files);
}
#[test]
fn validates_all_operations_before_write() {
    let t = fresh();
    let base = Snapshot::load(t.path()).unwrap();
    for ops in [
        vec![text("Changed"), style_op("position", Some("absolute"))],
        vec![text("<img src=x onerror=alert(1)>")],
        vec![text("<span>Unsupported</span>")],
        vec![style_op("font-size", Some("-10px"))],
        vec![style_op("color", Some("red;display:none"))],
        vec![Operation::Reorder {
            order: vec!["intro".into()],
        }],
        vec![Operation::Alt {
            slide: "intro".into(),
            element: "headline".into(),
            text: "wrong target".into(),
        }],
    ] {
        assert!(base.save(&base.revision, &ops).is_err());
        assert_eq!(base.files, Snapshot::load(t.path()).unwrap().files);
    }
}
#[test]
fn ambiguous_invalid_html_and_inline_css_are_rejected() {
    assert!(source::parse("<p data-element-id='a'>x</p><p data-element-id='a'>y</p>").is_err());
    assert!(source::parse("<div><p>x</div>").is_err());
    assert!(!source::parse("<div><p>x</p></div>").unwrap()[0].text_editable);
    assert!(source::parse("<script>if(a<b){console.log('<p>')}</script><h1>Good</h1>").is_ok());
    let t = fresh();
    fs::write(
        t.path().join("slides/intro.html"),
        "<h1 data-element-id='headline' style='font-size: 50px'>X</h1>",
    )
    .unwrap();
    let base = Snapshot::load(t.path()).unwrap();
    assert!(
        base.save(&base.revision, &[style_op("font-size", Some("64px"))])
            .unwrap_err()
            .to_string()
            .contains("inline style")
    );
}
#[test]
fn order_notes_and_extra_metadata_survive() {
    let t = fresh();
    let path = t.path().join("deck.json");
    let mut m: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    m["custom_metadata"] = json!({"retain":true});
    m["slides"][0]["custom"] = json!(42);
    fs::write(&path, serde_json::to_vec_pretty(&m).unwrap()).unwrap();
    let base = Snapshot::load(t.path()).unwrap();
    base.save(
        &base.revision,
        &[
            Operation::Reorder {
                order: vec!["next".into(), "intro".into(), "workflow".into()],
            },
            Operation::Notes {
                slide: "intro".into(),
                text: "Notes 日本語\nLine two".into(),
            },
        ],
    )
    .unwrap();
    let after = Snapshot::load(t.path()).unwrap();
    assert_eq!(
        after.manifest.slides[1].notes.as_deref(),
        Some("Notes 日本語\nLine two")
    );
    let raw: serde_json::Value = serde_json::from_slice(&after.files["deck.json"]).unwrap();
    assert_eq!(raw["custom_metadata"], m["custom_metadata"]);
    assert_eq!(raw["slides"][1]["custom"], 42);
    let html = String::from_utf8(after.assembled(false).unwrap()["index.html"].clone()).unwrap();
    assert!(
        html.find("data-slide-id=\"next\"").unwrap()
            < html.find("data-slide-id=\"intro\"").unwrap()
    );
    for forbidden in [
        "data-editor-key",
        "editor-selection",
        "x-decksmith-session",
        "bridge.js",
    ] {
        assert!(!html.contains(forbidden));
    }
}
#[cfg(unix)]
#[test]
fn reject_symlink_and_traversal_targets() {
    let t = fresh();
    let base = Snapshot::load(t.path()).unwrap();
    for id in ["../headline", "headline\" onclick=oops"] {
        assert!(
            base.save(
                &base.revision,
                &[Operation::Text {
                    slide: "intro".into(),
                    element: id.into(),
                    html: "x".into()
                }]
            )
            .is_err()
        );
    }
    let external = tempfile::NamedTempFile::new().unwrap();
    let target = t.path().join("styles/editor.css");
    std::os::unix::fs::symlink(external.path(), target).unwrap();
    assert!(
        base.save(&base.revision, &[style_op("color", Some("#ff0000"))])
            .is_err()
    );
    assert_eq!(fs::read(external.path()).unwrap(), b"");
}
#[test]
fn assets_are_validated_and_content_addressed() {
    use base64::Engine;
    let t = fresh();
    fs::write(
        t.path().join("slides/intro.html"),
        "<img data-element-id='image' src='assets/old.png' alt='Old'>",
    )
    .unwrap();
    let base = Snapshot::load(t.path()).unwrap();
    let image = image::DynamicImage::new_rgb8(2, 2);
    let mut out = std::io::Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).unwrap();
    let data = format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(out.into_inner())
    );
    let op = Operation::Image {
        slide: "intro".into(),
        element: "image".into(),
        data,
    };
    let r = base
        .save(&base.revision, std::slice::from_ref(&op))
        .unwrap();
    base.save(&r, &[op.clone(), op]).unwrap();
    let files = Snapshot::load(t.path()).unwrap().files;
    assert_eq!(
        files
            .keys()
            .filter(|p| p.starts_with("assets/editor-"))
            .count(),
        1
    );
    for data in [
        "data:image/svg+xml;base64,PHN2Zz4=",
        "data:image/png;base64,bm90IGEgcG5n",
        "../../../etc/passwd",
    ] {
        let latest = Snapshot::load(t.path()).unwrap();
        assert!(
            latest
                .save(
                    &latest.revision,
                    &[Operation::Image {
                        slide: "intro".into(),
                        element: "image".into(),
                        data: data.into()
                    }]
                )
                .is_err()
        );
    }
}

#[test]
fn early_fragment_scripts_are_read_only_but_still_rendered() {
    let t = fresh();
    fs::write(t.path().join("slides/intro.html"), "<h1 data-element-id='headline'>Source</h1><script>document.querySelector('h1').textContent='Runtime';</script>").unwrap();
    let base = Snapshot::load(t.path()).unwrap();
    assert!(base.elements().unwrap()["intro"][0].read_only.is_some());
    assert!(
        base.save(&base.revision, &[text("Changed")])
            .unwrap_err()
            .to_string()
            .contains("before source mapping")
    );
    assert!(
        String::from_utf8(base.assembled(false).unwrap()["index.html"].clone())
            .unwrap()
            .contains("textContent='Runtime'")
    );
}

#[test]
fn invalid_external_source_is_a_conflict_and_remains_untouched() {
    let t = fresh();
    let base = Snapshot::load(t.path()).unwrap();
    fs::write(
        t.path().join("slides/intro.html"),
        "<div><h1>External broken source",
    )
    .unwrap();
    assert!(
        base.save(&base.revision, &[text("Changed")])
            .unwrap_err()
            .to_string()
            .contains("CONFLICT")
    );
    assert_eq!(
        fs::read_to_string(t.path().join("slides/intro.html")).unwrap(),
        "<div><h1>External broken source"
    );
}
