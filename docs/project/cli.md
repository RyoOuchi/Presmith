# CLI and result schema v1

All optional directory arguments default to `.`. Paths containing spaces are
supported; quote them in the shell. `--out` resolves relative to the current working
directory, not the deck. Omitted outputs resolve inside the deck.

| Command | Behavior |
|---|---|
| `presmith init DIRECTORY` | Three-slide source project; no installs; refuses nonempty directory |
| `presmith setup [DIRECTORY]` | npm ci, then local Chromium download and launch probe; rerunnable |
| `presmith setup [DIRECTORY] --upgrade-renderer` | Back up and refresh bundled renderer files, then install dependencies |
| `presmith doctor [DIRECTORY] --json` | Manifest/assembly + actual Chromium launch; no installation |
| `presmith edit [DIRECTORY] --port 4174 --open` | Local GUI with explicit source saves, undo, conflicts and existing exports; see [editor guide](editor.md) |
| `presmith edit [DIRECTORY] --recover` | Recover an interrupted editor save before opening; close other editors first |
| `presmith dev [DIRECTORY] --port 4173 --open` | Loopback-only preview; port 0 selects a free port |
| `presmith check [DIRECTORY] --slide ID --json` | Manifest + active-slide browser findings |
| `presmith render [DIRECTORY] --slide ID --out PATH --json` | Slide PNG(s) + labeled contact sheet |
| `presmith export [DIRECTORY] --format html --out PATH --json` | Portable static directory, verified with external requests blocked |
| `presmith export [DIRECTORY] --format pdf --out PATH --json` | PDF with verified one page per slide |
| `presmith export [DIRECTORY] --format pptx --out PATH --json` | Editable text/shapes/pictures, notes and verified slide count |

Preview polls relevant manifest/fragments and public source directories every
500ms. It ignores tooling, generated output and node_modules. Valid changes reload
while retaining `#slide-id`. Build errors leave the last working deck visible with
an error overlay. Recovery rebuilds and reloads. Add new assets under assets/.

Defaults: `.decksmith/render/slides/ID.png`, `.decksmith/render/contact-sheet.png`,
`dist/html/`, `dist/deck.pdf`, `dist/deck.pptx`. Slide PNG names depend on stable IDs, not order. A
filtered render has only that PNG plus its contact sheet. Managed output directories
contain `.decksmith-output`; subsequent successful exports to the same directory
replace its contents (including any files you added). Do not author files there.
Nonempty unmanaged directories are refused. Inside a project output paths must be
under dist/ or .decksmith/; authored directories and project ancestors are protected.
Render into temporary storage first; failed browser checks do not install partial
artifacts. Copy failures while installing a completed artifact may leave partial
outputs; atomic cross-filesystem publishing is outside this prototype.

For JSON-capable commands stdout contains exactly one object, including operational
and CLI argument errors. Progress and human diagnostics go to stderr. Example:

```json
{"schema_version":1,"command":"check","success":true,"findings":[],"artifacts":[],"error":null}
```

Artifact records contain `kind`, absolute `path`, and optional `slide_id`, `width`,
`height`, `pages`, `slides`, `external_network`, `navigation_verified`. Doctor adds
`capabilities` (preview/check/render/html_export/pdf_export/pptx_export) and, when available,
`runtime` versions. Error objects contain `kind` (`deck`, `operational`, `usage`) and
`message`. Additional fields may be added within version 1; consumers should ignore
unknown fields. Help/version are normal clap output, not JSON results.

PPTX artifacts also include per-slide `editability` counts: `text_boxes`, `shapes`,
`images`, and `rasterized_elements` (a subset of images). Text boxes contain visible
line segments. Unsupported elements produce `pptx.rasterized` warnings and remain
static pictures. The export includes speaker notes. PowerPoint/Google Slides edits
are independent of HTML source; check fonts/layout after import.

For old decks, use `setup --upgrade-renderer`. It backs up every replaced renderer
file under `.decksmith/renderer-backups/` before refreshing bundled tooling.
Authored slides, styles, scripts and extra renderer files remain intact.

Exit codes: 0 success (including warning-only checks); 1 detected deck problems;
2 usage/operational failure; 130 interrupted. On SIGINT/SIGTERM, Rust closes its
in-process server and terminates/waits for its child group; the Node helper closes
Chromium. Parent pipe closure also triggers cleanup. Forced OS termination and
machine failure cannot guarantee deletion of temporary files. Process cleanup is
integration-tested on macOS; Windows is unverified.

Setup requires Node.js 22+, npm and network access. All browser packages/cache stay
in tooling/renderer. It does not require a global npm package and ignores npm
lifecycle scripts. Linux may need OS libraries: follow Playwright's install-deps
instructions, with administrator access if necessary; setup never invokes sudo.
