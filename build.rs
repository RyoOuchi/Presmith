use std::{env, fs, path::Path};
fn collect(base: &Path, path: &Path, prefix: &str, lines: &mut Vec<String>) {
    let mut entries: Vec<_> = fs::read_dir(path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for p in entries {
        let name = p.file_name().unwrap().to_string_lossy();
        if matches!(
            name.as_ref(),
            "node_modules" | ".browsers" | ".npm-cache" | ".DS_Store"
        ) {
            continue;
        }
        if p.is_dir() {
            collect(base, &p, prefix, lines);
        } else {
            let target = format!(
                "{prefix}{}",
                p.strip_prefix(base).unwrap().to_string_lossy()
            );
            lines.push(format!(
                "({target:?}, include_bytes!({:?})),",
                fs::canonicalize(p).unwrap()
            ));
        }
    }
}
fn main() {
    let mut lines = Vec::new();
    for (source, target) in [
        ("templates/starter", ""),
        ("renderer", "tooling/renderer/"),
        ("docs/project", "docs/"),
    ] {
        println!("cargo:rerun-if-changed={source}");
        if Path::new(source).exists() {
            collect(Path::new(source), Path::new(source), target, &mut lines);
        }
    }
    for (source, target) in [
        ("library/decksmith.css", "lib/decksmith.css"),
        ("library/decksmith.js", "lib/decksmith.js"),
        ("library/themes/ink.css", "styles/theme.css"),
        ("library/themes/paper.css", "styles/paper.css"),
    ] {
        println!("cargo:rerun-if-changed={source}");
        lines.push(format!(
            "({target:?}, include_bytes!({:?})),",
            fs::canonicalize(source).unwrap()
        ));
    }
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("assets.rs"),
        format!(
            "pub const ASSETS: &[(&str, &[u8])] = &[\n{}\n];",
            lines.join("\n")
        ),
    )
    .unwrap();
}
