# Presmith showcase

A ten-slide, roughly 7–8 minute presentation for developers and technical product teams. It showcases the local Presmith CLI and its Codex authoring skill. All product claims come from this repository and the installed skill. Speaker notes in `deck.json` include source references and suggested timing.

## Narrative

1. Presmith and the promise of editable presentation source
2. Codex, Rust, and Chromium responsibilities
3. The authored files and stable IDs
4. The CLI workflow
5. The Codex skill and an example brief
6. A focused one-slide revision
7. Automated checks and visual review
8. A live typography demo with a repeatable export state
9. HTML and PDF delivery
10. Getting started

## Preview and export

Run from this directory with `presmith` available on PATH:

```sh
presmith setup # required only when local renderer dependencies are missing
presmith doctor --json
presmith dev --open
# In another terminal:
presmith check --json
presmith render --json
presmith export --format html --json
```

To view the static HTML export:

```sh
python3 -m http.server 4185 --bind 127.0.0.1 --directory dist/html
```

Open http://127.0.0.1:4185. Use arrows or on-screen controls to navigate. Slide `interactive` has a headline-size slider. Captures and PDF reset it to 72px. The HTML export retains interaction. Speaker notes are embedded in HTML and are public.

If you need PDF, run `presmith export --format pdf --json`.

## Editing

Edit `deck.json`, `slides/`, `styles/custom.css`, and `scripts/custom.js`. Generated outputs live in `.decksmith/` and `dist/`. Local assets and system fonts keep the deck independent of external web requests. Font rendering can differ across operating systems.
