# Your Decksmith presentation

Decksmith is a Rust CLI. Node.js 22+, npm and Chromium are required for checks,
rendering and exports. Preview only needs the compiled CLI. No model API is called.

```sh
decksmith setup
decksmith doctor
decksmith dev --open
# In another terminal:
decksmith check
decksmith render
decksmith export --format html
decksmith export --format pdf
```

The manifest controls order; slide files do not need numeric prefixes. Edit HTML
fragments directly. The starter uses the Ink theme. Switch styles/theme.css to
styles/paper.css in deck.json for the Paper theme.

- [Authoring and paths](docs/authoring.md)
- [Runtime and deterministic rendering](docs/runtime.md)
- [Diagnostics](docs/diagnostics.md)
- [CLI, outputs and exit codes](docs/cli.md)

Render output is .decksmith/render/contact-sheet.png plus slides/ID.png files.
Exports are dist/html/ and dist/deck.pdf. Serve HTML with an ordinary static server,
for example `python3 -m http.server 8080 --directory dist/html`. Neither Rust nor
Node is needed to view that export. file:// is not a supported viewing method.

setup downloads dependencies only when explicitly invoked. Chromium and npm packages
stay under tooling/renderer; commit package-lock.json but not node_modules or .browsers.
On Linux, install Playwright's OS libraries if doctor reports they are missing.

Read AGENTS.md when asking a coding agent to edit this deck. Keep facts, stable IDs,
and unrelated slides intact. Always visually inspect actual rendered images.
