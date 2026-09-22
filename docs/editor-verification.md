# Visual editor verification — 2026-09-21

Verified locally on macOS with the real project-local Playwright Chromium. The
editor tests launch the compiled Rust binary against temporary authored projects;
they interact with the React UI and sandboxed canvas, inspect persisted source,
reopen projects, and call the existing rendering/export pipeline.

| Check | Result |
|---|---|
| `cargo test --locked` | 29 passed: 17 existing CLI, 11 editor source/persistence, 1 transaction rollback |
| `cargo clippy --locked --all-targets -- -D warnings` | Passed |
| `npm run typecheck --prefix editor` | Passed |
| `npm run build --prefix editor` | Passed; production assets embedded in the Rust binary |
| `npm run format:check --prefix editor` | Passed |
| `node tests/browser.mjs` | All 17 existing checks passed, including PPTX structure and PDF page count |
| `node tests/editor-browser.mjs` | All 19 editor integration checks passed |
| `git diff --check` | Passed |

The editor checks cover read-only opening, targeted HTML preservation, supported
emphasis and Unicode/line breaks, computed typography and reset, repeated CSS
updates, undo/redo after Save, nested selection, 50%/75% zoom alignment, resized
logical dimensions, containing-block movement with preserved transforms, flex/grid
layout, inline/specificity restrictions, runtime-node read-only behavior, synthetic
composition events, images/alt text, notes, keyboard and drag reordering, browser-tab
reopening without restarting the server, server restart/reopening, external-change
conflicts, explicit reload/discard, authorization/origin checks, ordinary-preview
write isolation, pending-edit survival through preview/network failure, and the
GUI export control. A FontFace load and viewport resize verify selection alignment;
a missing image produces an actionable UI error.

An edited three-slide fixture was exported through HTML, PNG, PDF and PPTX. HTML
source, order and notes were inspected, PNGs were generated and visually reviewed,
PDF was parsed to confirm three pages, and PPTX XML/notes/order were checked. The
existing PPTX native-text/shape/image/fallback checks were retained and passed.

Screenshots in `verification/editor/` were opened and inspected, not merely
captured:

- `starter-editor.png`: saved source edits, properties, thumbnails and selection.
- `product-editor.png`: the representative existing eight-slide product deck.
- `small-desktop.png`: all three panels at 1024 × 800; usable selection handles.
- `conflict.png`: retained pending text, blocked stale save, recovery actions.
- `edited-slide.png`: edited exported slide with no editor overlay.

The final screenshots show readable controls, distinct panels, correctly aligned
selection and accurate slide rendering. The fixture includes deliberate geometry,
image, runtime and specificity test elements; these are not added to the starter
or product deck sources. JSON test records are in `verification/browser-results.json`
and `verification/editor/results.json` (generated verification files are ignored).

Operating-system IME was **not** tested; composition coverage uses browser events.
PowerPoint, Keynote and Google Slides imports were **not** opened or inspected.
Windows/Linux, very large decks and concurrent external writers during the brief
multi-file commit window were not exercised. The save journal provides rollback
and explicit recovery, not a filesystem-wide transaction or automatic merge.

See [the editor guide](project/editor.md) for supported source structures, geometry
restrictions, inline-script handling, explicit Save, undo and conflict behavior.
No publishing, pushing or globally installed plugin changes were performed.
