# Verification record

Verified on 2026-09-17, macOS arm64, Rust/Cargo 1.98.1, Node.js 26.0.0,
Playwright 1.58.2 and its Chromium 145.0.7632.6. These are observed local results,
not a claim of cross-platform certification or comparative productivity.

## Automated results

- `cargo fmt --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test --locked`: 14 integration tests passed, including compiled-binary
  initialization outside the checkout, safe paths, JSON/exit codes, missing Node,
  and 32 simultaneous keep-alive HTTP connections plus server shutdown.
- `node tests/browser.mjs examples/product`: 16 end-to-end checks passed.
- Fresh compiled-CLI workflow: init → setup → doctor → preview (HTTP 200) → check
  → render → HTML export → PDF export: passed in `verification/final talk`.
- Product example: zero check findings; eight 1280×720 slide PNGs; contact sheet;
  portable HTML; PDF independently parsed as exactly eight pages.
- PDF text extraction: nonempty selectable text on every page; page text follows
  the eight manifest topics in order. The cover PDF page was also rasterized and viewed.
- Skill validator: passed. Official portable plugin JSON schema: passed.

The browser checks cover:

- compiled binary initializes outside repository, including spaces
- doctor launches Chromium and returns JSON capabilities
- preview: navigation, scaling, isolation, reload, error recovery and shutdown
- starter check is clean; warning-only returns zero
- screenshots: count, dimensions, manifest order, selected stable filename
- contact-sheet slide ID cannot collide with the overview filename
- PDF: exact page count and logical aspect ratio
- HTML: ordinary static server, subdirectory, navigation, no external requests/dependencies
- broken fixtures fire the important rules; decorative overflow is exempt
- required external requests are blocked and reported
- targeted revision preserves every unrelated slide source
- Paper theme uses the same runtime and checks
- registered async state is awaited and interactive state resets on export
- missing Chromium gives actionable JSON without false success
- interruption closes helper and browser descendants
- successful and failed export commands leave no child processes

## Actual visual review

Opened the example contact sheet and all eight full-size PNGs with an image viewer.
Checked hierarchy, whitespace, comparison rows, process columns, diagram arrows and
labels, code readability, chart labels and footers. No unresolved visible clipping
was found. Opened the rasterized PDF cover and the revised starter workflow slide.
This was image inspection, not merely a claim based on successful render commands.
The two themes passed browser checks; only the Ink example received full manual
visual review here.

Final example artifacts (generated and Git-ignored):

- `examples/product/.decksmith/render/contact-sheet.png`
- `examples/product/.decksmith/render/slides/ID.png`
- `examples/product/dist/html/`
- `examples/product/dist/deck.pdf`

## Targeted revision evidence

In the fresh deck, changed only the workflow headline from “Three moves. One clear
message.” to “One story. Three deliberate moves.” Then checked and rendered that
slide and opened its image. The other two slide sources were byte-for-byte unchanged.
SHA-256 prefixes (full hashes are in the machine-readable record):

| Source | Before | After | Preserved? |
|---|---|---|---|
| intro.html | `977aa0cc655ab612` | `977aa0cc655ab612` | yes |
| next.html | `04f5366a82127172` | `04f5366a82127172` | yes |
| workflow.html | `bb9c5c8978bd6a68` | `fe84a83f0f5686fa` | requested change |

Machine-readable evidence is in `verification/fresh-workflow.json` and
`verification/browser-results.json` (local, generated, Git-ignored). The fresh deck's
full-deck exports precede the demonstration revision; the revised PNG is separately
stored at `verification/final talk/.decksmith/revision/slides/workflow.png`.

## Verification boundaries

No runtime dependency blockers remain on this machine. Sandbox network/process
restrictions required approved local tool execution for downloads and Chromium.
No global npm package, personal Codex marketplace or global Codex setting was changed.
The final HTTP server uses Axum/Tokio and owns its runtime and shutdown; burst browser
loads and keep-alive sockets are exercised after resolving intermittent load stalls.

The available older plugin ingestion validator reports two absent publisher fields:
`author` and `interface.developerName`. No real identity was supplied, so none was
invented. The current portable Agent Plugins 1.0.0 manifest schema validates without
those optional fields. See docs/plugin.md for exact installation/validation boundaries.
The plugin was not installed into the user's Codex account; activation/UI discovery
was not tested. Linux/Windows execution and cross-OS font identity were not tested.
The proposed toolkit-vs-unassisted-agent evaluation has not been run.
