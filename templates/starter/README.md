# Your Presmith presentation

Presmith is a Rust CLI. Node.js 22+, npm and Chromium are required for checks,
rendering and exports. Preview only needs the compiled CLI. No model API is called.

```sh
presmith setup
presmith doctor
presmith dev --open
# In another terminal:
presmith check
presmith render
presmith export --format html
presmith export --format pdf
presmith export --format pptx
```

The manifest controls order; slide files do not need numeric prefixes. Edit HTML
fragments directly. The starter uses the Ink theme. Switch styles/theme.css to
styles/paper.css in deck.json for the Paper theme.

- [Authoring and paths](docs/authoring.md)
- [Runtime and deterministic rendering](docs/runtime.md)
- [Diagnostics](docs/diagnostics.md)
- [Complete command reference, outputs and exit codes](docs/cli.md)
- [Use cases and step-by-step recipes](docs/use-cases.md)

Render output is .decksmith/render/contact-sheet.png plus slides/ID.png files.
Exports are dist/html/, dist/deck.pdf and dist/deck.pptx. Serve HTML with an ordinary static server,
for example `python3 -m http.server 8080 --directory dist/html`. Neither Rust nor
Node is needed to view that export. file:// is not a supported viewing method.

Setup explicitly installs or reuses a shared renderer cache. Matching projects share
Chromium and npm packages through links under tooling/renderer. Use `presmith setup
--local` for a self-contained installation. Commit package-lock.json, not
node_modules or .browsers. See [cache locations and migration](docs/cli.md#renderer-cache).
On Linux, install Playwright's OS libraries if doctor reports they are missing.

Open PPTX in PowerPoint or upload to Google Slides for visual editing. Text lines
and basic shapes stay editable; unsupported visuals become pictures with warnings.
PPTX includes speaker notes. Imported edits do not sync to HTML. For older decks,
run `presmith setup --upgrade-renderer`; replaced tooling is backed up first.

This deck includes the Presmith Codex skill in `.agents/skills/presmith/`, with
references and picker metadata copied from the CLI. Open the deck in Codex and use
`$presmith`; no separate download is needed. Restart Codex if the skill is not
visible. For availability across projects, `presmith skill install --global`
installs the same embedded skill in your user directory. See [skill installation](docs/cli.md#skill-install).

Read AGENTS.md when asking a coding agent to edit this deck. Keep facts, stable IDs,
and unrelated slides intact. Always visually inspect actual rendered images.
