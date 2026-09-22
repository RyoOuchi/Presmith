# Presmith

A working local prototype for creating presentations with a coding agent. Write one
HTML fragment per slide, preview immediately, get browser diagnostics, inspect PNGs,
and export HTML, PDF or editable PowerPoint files. Your project stays readable and versionable.

**Initialize → ask Codex → preview → check → render and inspect → revise → export.**

Presmith is available as an MIT-licensed macOS Apple Silicon prerelease.
Install it with `brew install ryoouchi/tap/presmith`, or download it from
[GitHub releases](https://github.com/RyoOuchi/Presmith/releases).
Follow the [installation guide](docs/install.md). See the
[v0.2.0 release notes](docs/releases/v0.2.0.md) for features and preview limitations.
There is no model API integration, chat UI, account, cloud service, deployment
feature or MCP server.

The CLI is now named `presmith` (formerly `decksmith`). Existing decks remain
compatible: `deck.json`, `lib/decksmith.css`, `lib/decksmith.js`, the
`window.Decksmith` browser API, and `.decksmith/` output paths retain their existing
names. Use `presmith` for CLI commands and `$presmith` for the bundled Codex skill.

See the [documentation index](docs/README.md), [complete command reference](docs/project/cli.md),
and [use cases and recipes](docs/project/use-cases.md) for detailed workflows and options.

## Quick start

On an Apple Silicon Mac with **macOS 14 or newer** and
[Homebrew](https://brew.sh/) installed:

```sh
brew install ryoouchi/tap/presmith

presmith init my-talk
cd my-talk
presmith setup
presmith doctor
presmith dev --open
# Or edit content, appearance, slides and notes visually:
presmith edit --open
```

Homebrew installs Node.js 24 and npm and selects that runtime for Presmith.
`setup` reuses a shared, versioned cache of pinned Playwright packages and Chromium.
Matching decks share one installation; `setup --local` opts into a self-contained
installation. See [renderer cache and migration](docs/project/cli.md#renderer-cache)
for paths, cleanup and existing decks. No global npm packages are needed. See
[Install Presmith](docs/install.md) for manual and source installation, or run
`cargo install --path . --locked` from a source checkout with stable Rust/Cargo.
Browser operations require **Node.js 22+ and npm**. Linux may require
[Playwright system libraries](https://playwright.dev/docs/browsers#install-system-dependencies).

Keep preview running; in another terminal inside my-talk:

```sh
presmith check --json
presmith render
# Open .decksmith/render/contact-sheet.png and individual slide PNGs.
presmith export --format html
presmith export --format pdf
presmith export --format pptx
```

Without installing the binary, use `/absolute/path/to/Presmith/target/release/presmith`
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

## Edit in PowerPoint or Google Slides

```sh
# For a deck initialized with an older Presmith version:
presmith setup my-talk --upgrade-renderer
presmith export my-talk --format pptx --json
```

Open `my-talk/dist/deck.pptx` in PowerPoint, or upload it to Google Drive and choose
**Open with → Google Slides**. Text exports as editable line-sized text boxes;
fills, borders, rectangles, ellipses and simple SVG geometry remain separate shapes.
Images remain replaceable pictures. Slide order, canvas dimensions and speaker
notes carry over. Each JSON artifact includes per-slide `editability` counts.

Complex SVG, controls, canvas, transforms and unsupported CSS effects become static
pictures and produce `pptx.rasterized` warnings. `data-pptx="raster"` can explicitly
request a picture for a complex element. Fonts may substitute and text metrics can
shift between applications. Review the imported deck before presenting. Tables
export as editable text and borders, not semantic Office tables; HTML chart shapes
do not become data-backed Office charts. Interactive state follows export hooks.
Changes in PowerPoint/Slides do not sync back to HTML. Notes are included in PPTX.

`setup --upgrade-renderer` backs up replaced renderer files under
`.decksmith/renderer-backups/`, updates the bundled renderer, and installs its pinned
dependencies. It preserves authored slides, styles, scripts and custom extra tooling
files. Without this flag, setup retains the existing renderer. Doctor reports
`pptx_export` separately so older decks do not claim support.

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
| `editor/` | React/TypeScript visual editor, pinned build tools and embedded production assets |
| `renderer/` | Node helper; exact package versions and package-lock.json |
| `plugins/presmith/` | Portable Codex plugin, compatibility manifest, skill/references |
| `examples/product/` | Eight authored slides explaining the product |
| `docs/project/` | Project documentation embedded by init |
| `tests/` | CLI/browser integration checks and broken fixtures |
| `scripts/` | Example materialization and fresh-workflow verification |

`build.rs` embeds the canonical assets, omitting installed dependencies. The example
materialization script copies shared assets from those canonical sources; the copies
are ignored by Git. Edit canonical library/helper assets, then rerun the script.
Cargo.lock and renderer/package-lock.json are versioned. Cargo publishing and npm
publishing are disabled by the package manifests. The visual editor uses React and TypeScript with an esbuild bundle; slides remain
ordinary HTML/CSS, with no framework project format.

## The example

```sh
cargo build --locked
python3 scripts/prepare-example.py
./target/debug/presmith setup examples/product
./target/debug/presmith dev examples/product --open
./target/debug/presmith check examples/product --json
./target/debug/presmith render examples/product
./target/debug/presmith export examples/product --format html
./target/debug/presmith export examples/product --format pdf
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
Current source builds embed the complete skill in the CLI. `presmith init my-talk`
automatically includes `.agents/skills/presmith/` with its references and picker
metadata. Open the deck in Codex and use `$presmith`; no separate download or
plugin registration is needed for this project-local skill.

To install the same embedded skill for all your projects:

```sh
presmith skill install --global
# After updating the CLI, back up and replace a different global copy:
presmith skill install --global --force
```

Both operations work offline. An identical global copy is left untouched. A
different copy requires `--force`, which backs it up outside the skill discovery
folder. The global command does not change Codex settings or marketplace plugins.
These commands require a build containing this feature; the published v0.2.0
binary predates it. See [skill installation](docs/plugin.md#bundled-skill).

The skill supports
[seven creation workflows](plugins/presmith/skills/presmith/references/creation-workflows.md):
briefs, scripts, supplied structures, documents, data, existing decks and demos.
It infers the workflow from your material and can combine inputs. Example prompts:

1. “Use $presmith to create an eight-slide engineering proposal for replacing our
   nightly batch pipeline. Audience: backend leads. Use these incident notes as facts,
   compare two options, and end with the decision needed. Draft the outline first.”
2. “On slide solution, shorten the headline to seven words and make the diagram
   labels easier to read. Preserve the evidence, slide IDs and all other slides.
   Check and visually inspect the revised slide.”
3. “Prepare this deck for a ten-minute customer demo. Keep all supplied metrics and
   citations. Check the whole deck, open the rendered images, repair up to three
   passes, then export HTML and PDF and report unresolved issues.”
4. “Use $presmith to turn this keynote script into 12 slides. Keep my spoken wording
   in speaker notes, follow the existing sequence, and use minimal text on screen.”
5. “Use $presmith to build exactly the six slides in this outline. Keep the titles
   and order, use this report for evidence, and do not add a cover or closing slide.”

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

For packaging and publishing, see [Releasing Presmith](docs/releasing.md).

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
Output directories with the Presmith ownership marker are replaced on successful
reruns; keep source out of them. Artifact installation is not atomic across filesystems.
Browser checks require local process/network permissions even though they bind only
loopback. macOS is exercised here; Linux/Windows portability is not fully verified.
The CLI is designed for trusted local project source, not hostile web content.

## Visual editor

`presmith edit [DIRECTORY] --open` launches the local source editor. Select slides
and elements, edit text/typography/colors/spacing/borders, resize supported blocks,
move supported positioned elements, replace images, edit notes, reorder slides,
and use undo/redo. Explicit Save updates HTML, `deck.json`, and scoped overrides in
`styles/editor.css`. Reset buttons remove individual overrides. Save retains undo
history. External changes trigger a conflict with pending-edit download and an
explicit reload/discard action. Opening a deck does not change its source files.

See [supported operations, source rules, save/recovery and limitations](docs/project/editor.md).
Ordinary preview and exports have no editor mutation API. No plugin installation,
account or cloud service is involved.

### Frontend development and production build

The production assets in `editor/build/` are versioned and embedded by Rust. A
normal `cargo build --locked` therefore needs no frontend tooling and the resulting
binary serves the editor without a separate development server. After editing
frontend source, regenerate and commit the bundle:

```sh
npm ci --ignore-scripts --prefix editor
npm run typecheck --prefix editor
npm run build --prefix editor
cargo build --locked
./target/debug/presmith edit examples/product --open
```

`npm run dev --prefix editor` watches and rebuilds both frontend entry points.
Rebuild/restart the Rust editor after a bundle change; generated assets are embedded
at compile time. This avoids a frontend dev server becoming a runtime dependency.
All new dependencies are pinned, and `editor/package-lock.json` is committed.

### Editor verification

```sh
cargo test --locked
npm run typecheck --prefix editor
npm run build --prefix editor
cargo build --locked
# Requires an installed renderer/Chromium (shared or local; see setup above):
node tests/browser.mjs
node tests/editor-browser.mjs
```

Editor integration tests use real Chromium, persist and reopen source edits, check
computed styles/geometry and write boundaries, exercise conflicts/history/assets,
and export edited decks through HTML/PNG/PDF/PPTX. Screenshots and test results are
written to `verification/editor/` for actual visual inspection. Synthetic browser
composition events cover input handling; operating-system IME and Office import
verification are separate manual checks.

See the [recorded editor verification and visual inspection](docs/editor-verification.md)
for tested behavior and explicit verification limits.

Shared-cache verification (uses the example’s existing browser and npm downloads offline):

```sh
cargo build --locked
node tests/cache.mjs
```

Shared-cache support is in this source checkout; older published binaries may
still use per-project installations.
