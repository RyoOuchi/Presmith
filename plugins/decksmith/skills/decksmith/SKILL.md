---
name: decksmith
description: Create, inspect, revise and export HTML presentations using the Decksmith CLI. Use when the user requests Decksmith or is editing a Decksmith project with deck.json and HTML slide fragments. Not for PowerPoint editing or general website development.
---

# Decksmith

Create presentations the user can edit and version as ordinary HTML, CSS and
JavaScript. Codex develops the narrative and edits source; the Rust CLI assembles
and serves the deck; Node.js and Chromium perform browser checks and rendering.
The CLI never calls a model.

## Inspect the project

Read the deck's `AGENTS.md`, `README.md`, `deck.json` and relevant slide sources.
Inspect existing changes with Git before editing. Identify whether the request is
for a new deck, a targeted revision, or inspection/export of an existing deck.

Use `decksmith --help` and `decksmith doctor --json` to establish available commands
and dependencies. If `decksmith` is not on PATH, check the repository's documented
build instructions and existing compiled binary. Do not assume it is globally
installed or substitute a different presentation framework.

Read [authoring conventions](references/authoring.md) before changing the manifest,
asset paths, or initialization hooks. Read other references only as needed.

## Create a deck

Infer audience, purpose, approximate duration or slide count, and desired style
from the request. Ask only for missing information that materially affects the
result; otherwise state reasonable assumptions. Draft a concise narrative outline
before creating slides. Each slide should advance one main point.

Initialize a new or empty directory; never initialize over an existing deck:

```sh
decksmith init my-talk
cd my-talk
decksmith setup
decksmith doctor --json
```

`setup` explicitly installs pinned project-local dependencies and Chromium. It
requires Node.js 22+, npm and download access. Reuse existing authorization for
necessary setup; if installation is outside the request's scope or cannot run,
report the exact prerequisite and continue independent source work. Do not claim
browser verification succeeded without those dependencies.

Use [layout patterns](references/layouts.md) for titles, explanations, comparisons,
processes, statistics, diagrams, images, code and closing slides. Reuse theme tokens
and typography; vary layouts to suit the content. Shorten crowded text before
shrinking it. Preserve supplied facts and source attribution. Clearly label
illustrative data, and do not invent evidence or citations.

Author one HTML fragment per slide. Let `deck.json` control order and speaker notes.
Use stable slide IDs and unique `data-element-id` values within each slide. Keep
assets local and paths relative. Register required asynchronous initialization and
repeatable static export states through `Decksmith.register` when needed.
Edit source files, not generated files under `.decksmith/` or `dist/`.

## Revise an existing deck

Follow the [revision workflow](references/revision.md). Identify the requested slide
and elements before making changes. Preserve unrelated slides, stable IDs, facts,
notes and ordering unless the request explicitly changes them.

Keep local CSS scoped to `[data-slide-id="ID"]`. Do not change global theme tokens
for a local request. Inspect the resulting diff against the starting state. Without
Git history, compare recorded hashes of unrelated slide sources.

Check and render the affected slide first, replacing `SLIDE_ID` with its actual ID:

```sh
decksmith check --slide SLIDE_ID --json
decksmith render --slide SLIDE_ID --json
```

When shared CSS, scripts or runtime behavior changes, check and render the whole
deck instead. Inspect every slide potentially affected by the shared change.

## Check, render and inspect

Run commands from the deck directory, or pass its path explicitly. For a full deck:

```sh
decksmith check --json
decksmith render --json
```

Use `decksmith dev --open` for an interactive preview; checks and exports start
their own temporary server and do not need preview running. See the [CLI reference](references/cli.md)
for options, output paths and JSON results, and [diagnostics](references/diagnostics.md)
for interpreting findings. Exit 1 means deck problems; exit 2 means a usage or
operational failure. Warning-only checks may succeed; still assess the warnings.

**Open the rendered contact sheet and relevant full-size slide PNGs with available
image-viewing tools. Generating screenshots is not visual inspection.** For a new
deck, inspect every slide. Check hierarchy, readable text, spacing, clipping,
diagram labels, chart units and consistency. If image-viewing tools are unavailable,
provide the image paths and explicitly mark visual review as unverified.

Use at most three check → render → inspect → repair passes per request. Stop sooner
when satisfactory. After the third pass, report unresolved issues and the next
specific action. Do not suppress a finding by marking meaningful content decorative;
the decorative-overflow convention is only for nonessential artwork. A clean check
does not establish narrative quality, factual accuracy or visual polish.

## Export and hand off

Follow [export behavior](references/export.md) and export the requested format:

```sh
decksmith export --format html --json
decksmith export --format pdf --json
```

Run both only when both are requested. Use artifact paths returned by the command.
PDF must report one page per slide. View exported HTML through an ordinary static
server; `file://` is not supported. Interactive content needs a defined export
state, and system-font differences can affect appearance across operating systems.
HTML exports include speaker notes in the embedded manifest.

Report what changed, artifact paths, diagnostics actually run, images actually
inspected, and unresolved limitations. If rendering or export failed, identify the
blocker rather than presenting an older output as the new result. Ordinary deck
authoring does not include installing the Codex plugin, changing global Codex
configuration, publishing packages, or deploying the presentation.
