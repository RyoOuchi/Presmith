# CLI

`presmith init my-talk`; `cd my-talk`; `presmith setup`; `presmith doctor --json`.
Setup requires Node 22+/npm and downloads pinned local Chromium. Nothing installs
on init or doctor. Preview needs only the Rust binary: `presmith dev --open`.
Use `--port 4173` or `--port 0`. All optional project paths default to `.`.

`presmith check [directory] --slide ID --json`
`presmith render [directory] --slide ID --out PATH --json`
`presmith export [directory] --format html|pdf|pptx --out PATH --json`

Omit --slide for the full deck. These commands start their own temporary server.
Default renders: .decksmith/render/slides/ID.png and contact-sheet.png. Defaults for
exports: dist/html/, dist/deck.pdf and dist/deck.pptx. --out paths resolve from the current working
directory; quote paths with spaces. Reusing a managed output directory replaces its
contents. Keep authored files out of outputs.

JSON: schema_version 1, command, success, findings[], artifacts[], error. Error has
kind/message. Exit 0 = success (warnings allowed), 1 = deck problem, 2 = dependency,
usage or operational failure, 130 = interrupted. Do not call a skipped render a success.

Doctor reports `pptx_export` separately. For decks with older tooling, use
`presmith setup [directory] --upgrade-renderer`; replaced renderer files are backed
up under .decksmith/renderer-backups/ before the refresh and dependency installation.
