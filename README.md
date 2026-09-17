# Decksmith

A working local prototype for creating presentations with a coding agent. Write one
HTML fragment per slide, preview immediately, get browser diagnostics, inspect PNGs,
and export static HTML or PDF. Your project stays readable and versionable.

**Initialize → ask Codex → preview → check → render and inspect → revise → export.**

Decksmith and `decksmith` are provisional local development names. Nothing is
published and no name availability is claimed. There is no model API integration,
chat UI, account, cloud service, deployment feature, MCP server or PowerPoint format.

## Quick start

Prerequisites: stable Rust/Cargo to build, plus **Node.js 22+ and npm** for browser
operations. `setup` downloads pinned Playwright packages and Chromium into the deck.
No global npm packages are needed. Linux may require [Playwright system libraries](https://playwright.dev/docs/browsers#install-system-dependencies).

```sh
cargo build --release --locked
# Optional local CLI installation, explicitly initiated by you:
cargo install --path . --locked

decksmith init my-talk
cd my-talk
decksmith setup
decksmith doctor
decksmith dev --open
```

Keep preview running; in another terminal inside my-talk:

```sh
decksmith check --json
decksmith render
# Open .decksmith/render/contact-sheet.png and individual slide PNGs.
decksmith export --format html
decksmith export --format pdf
```

Without installing the binary, use `/absolute/path/to/Desksmith/target/release/decksmith`
in these commands. Init embeds every scaffold/library/helper asset: the compiled
binary works outside this checkout. Init and doctor never silently install packages.
Setup is explicit and rerunnable. On a dependency failure, follow doctor's guidance;
a skipped render is never called a success.

View exported HTML using any static web server:

```sh
python3 -m http.server 8080 --directory dist/html
```

Open localhost:8080. **Generated presentations need neither Rust nor Node to be
viewed through an ordinary static server.** file:// is not a supported viewing mode.

## Responsibilities and repository layout

Rust handles arguments, manifest validation, project scaffolding, HTML assembly,
loopback serving, source-change polling, output ownership and child-process cleanup.
A small Node helper uses Playwright/Chromium for browser layout, asset/runtime checks,
PNG capture and PDF printing. It parses the PDF to verify the final page count.
Codex supplies narrative and editing judgment through the plugin's skill.

| Path | Purpose |
|---|---|
| `src/` | Single Rust CLI/library crate |
| `library/` | Canonical vanilla CSS/JS and Ink/Paper themes |
| `templates/starter/` | Authored three-slide scaffold and short agent instructions |
| `renderer/` | Node helper; exact package versions and package-lock.json |
| `plugins/decksmith/` | Portable Codex plugin, compatibility manifest, skill/references |
| `examples/product/` | Eight authored slides explaining the product |
| `docs/project/` | Project documentation embedded by init |
| `tests/` | CLI/browser integration checks and broken fixtures |
| `scripts/` | Example materialization and fresh-workflow verification |

`build.rs` embeds the canonical assets, omitting installed dependencies. The example
materialization script copies shared assets from those canonical sources; the copies
are ignored by Git. Edit canonical library/helper assets, then rerun the script.
Cargo.lock and renderer/package-lock.json are versioned. Cargo publishing and npm
publishing are disabled by the package manifests. There is no frontend framework,
bundler or custom slide language.

## The example

```sh
cargo build --locked
python3 scripts/prepare-example.py
./target/debug/decksmith setup examples/product
./target/debug/decksmith dev examples/product --open
./target/debug/decksmith check examples/product --json
./target/debug/decksmith render examples/product
./target/debug/decksmith export examples/product --format html
./target/debug/decksmith export examples/product --format pdf
```

Source: examples/product/deck.json and slides/. Overview:
examples/product/.decksmith/render/contact-sheet.png. Exports:
examples/product/dist/html/ and examples/product/dist/deck.pdf.
The example contains a process layout, comparison, SVG diagram, HTML code sample,
large statistic, closing slide and live slider chart. Chart values are explicitly
illustrative; export resets the slider to 60%.

## Codex plugin and prompts

See [verified local installation instructions and validation boundaries](docs/plugin.md).
No global Codex settings or personal marketplace were changed by this implementation.
Use the repository-local plugin or copy its self-contained skill into a deck's
`.agents/skills/decksmith/`. Install the CLI separately. Three realistic prompts:

1. “Use $decksmith to create an eight-slide engineering proposal for replacing our
   nightly batch pipeline. Audience: backend leads. Use these incident notes as facts,
   compare two options, and end with the decision needed. Draft the outline first.”
2. “On slide solution, shorten the headline to seven words and make the diagram
   labels easier to read. Preserve the evidence, slide IDs and all other slides.
   Check and visually inspect the revised slide.”
3. “Prepare this deck for a ten-minute customer demo. Keep all supplied metrics and
   citations. Check the whole deck, open the rendered images, repair up to three
   passes, then export HTML and PDF and report unresolved issues.”

The skill requires actual image inspection when available, localized revisions,
diff review, and an explicit bounded repair loop. It never equates generating a
screenshot with visually reviewing it.

## Project format and runtime

A deck owns deck.json, slides/, styles/, scripts/, assets/, lib/, tooling/renderer/,
AGENTS.md, README.md and docs/. The versioned manifest orders stable slide IDs and
stores title, dimensions, source paths, optional titles/notes, styles and scripts.
Default logical size is 1280×720. Fragments are wrapped in predictable containers;
`data-element-id` identifies content within a slide. Speaker notes remain embedded
in exported HTML, so they are not private.

- [Authoring, paths, CSS scoping and semantic layout patterns](docs/project/authoring.md)
- [Runtime API, readiness, interaction, PDF and HTML contracts](docs/project/runtime.md)
- [Commands, JSON schema, output ownership and exit codes](docs/project/cli.md)
- [Diagnostic rules and limitations](docs/project/diagnostics.md)

`window.Decksmith` exposes `listSlides`, `select`, `current`, `setExportMode`, `ready`
and `register`. Async init/export hooks, font readiness and required images have
bounded waits. Exports suppress controls and CSS motion and preserve logical size.
Every slide is checked while active. Hidden slides are inert; navigation ignores
editable controls and respects stable hashes. Preview reloads preserve the slide.

## Verification

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
# After setting up the example's renderer:
node tests/browser.mjs examples/product
# Full init → setup → doctor → preview → check → render → HTML → PDF:
python3 scripts/verify-workflow.py
```

The fresh-workflow script requires a new empty target (default `verification/workflow talk`) and explicitly downloads dependencies. Pass another directory as its argument
to rerun without overwriting evidence. It demonstrates a one-slide revision and
records SHA-256 hashes proving unrelated slide sources remain unchanged.
The browser suite uses temporary projects outside the repository, a normal Python
static server, blocked external requests, broken fixtures, and descendant-process
checks. It is separate from cargo test so missing browser setup is not silently skipped.
See [recorded verification](docs/verification.md) and the [comparison evaluation
protocol](docs/evaluation.md). No comparative evaluation results are fabricated.

## Prototype limits

Diagnostics cover configuration, duplicate IDs, failed assets/images, uncaught JS,
bounds, simple text overflow/clipping and small text. They do not judge narrative,
contrast, semantic accessibility, overlapping objects, SVG internals or canvas pixels.
A clean check is not a guarantee of good design. Custom timers, complex widgets and
print CSS need explicit author care. Fonts/OS rendering can differ despite browser
pinning; universal pixel identity is not promised.

HTML export is verified with external network requests blocked. It includes only
presentation resources, not development dependencies. PDF is static, preserves
selectable browser text where possible, and checks exactly one page per slide.
Output directories with the Decksmith ownership marker are replaced on successful
reruns; keep source out of them. Artifact installation is not atomic across filesystems.
Browser checks require local process/network permissions even though they bind only
loopback. macOS is exercised here; Linux/Windows portability is not fully verified.
The CLI is designed for trusted local project source, not hostile web content.
