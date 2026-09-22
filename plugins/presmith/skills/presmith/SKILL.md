---
name: presmith
description: Create, inspect, revise and export HTML presentations using the Presmith CLI, starting from briefs, scripts, outlines or source material. Use when the user requests Presmith or is editing a Presmith project with deck.json and HTML slide fragments. Not for direct PowerPoint editing or general website development.
---

# Presmith

Create presentations the user can edit and version as ordinary HTML, CSS and
JavaScript. Codex develops the narrative and edits source; the Rust CLI assembles
and serves the deck; Node.js and Chromium perform browser checks and rendering.
The CLI never calls a model.

## Inspect the project

Read the deck's `AGENTS.md`, `README.md`, `deck.json` and relevant slide sources.
Inspect existing changes with Git before editing. Identify whether the request is
for a new deck, a targeted revision, or inspection/export of an existing deck.

Use `presmith --help` and `presmith doctor --json` to establish available commands
and dependencies. If `presmith` is not on PATH, check the repository's documented
build instructions and existing compiled binary. Do not assume it is globally
installed or substitute a different presentation framework.

Read [authoring conventions](references/authoring.md) before changing the manifest,
asset paths, or initialization hooks. Read other references only as needed.

## Create a deck

### Choose the starting point

Infer the workflow from the user's request and supplied material. Do not make the
user pick a mode when the input is clear. If they ask what is possible, use these
starting points and the example prompts in the linked reference.

| Starting point | Read when using it | Main constraint |
|---|---|---|
| Topic or brief | [Brief](references/creation-workflows.md#brief) | Develop the narrative around the audience and purpose. |
| Script or transcript | [Script](references/creation-workflows.md#script) | Map spoken beats to slides and preserve narration in notes. |
| Outline or slide specification | [Structure](references/creation-workflows.md#structure) | Preserve the supplied order, required content and slide count. |
| Documents or research notes | [Documents](references/creation-workflows.md#documents) | Distinguish source evidence from interpretation. |
| Data or results | [Data](references/creation-workflows.md#data) | Preserve units, definitions and traceable calculations. |
| Existing deck or visual reference | [Existing deck](references/creation-workflows.md#existing-deck) | Establish whether to preserve content, design, or both. |
| Demo steps or storyboard | [Demo](references/creation-workflows.md#demo) | Map each step to a visible state and a repeatable export. |

Read only the relevant sections of [creation workflows](references/creation-workflows.md).
These are authoring workflows for Codex, not additional CLI commands or import flags.
Inputs can combine: an outline can set order, a script supply narration, a report
provide evidence, and a reference establish style. Follow explicit user priorities.
Resolve material conflicts rather than silently dropping content or changing scope.

### Plan and author

Infer audience, purpose, approximate duration or slide count, and desired style
from the request. Ask only for missing information that materially affects the
result; otherwise state reasonable assumptions. Draft a concise slide plan before
creating slides, or map the supplied structure without replacing it. For each
slide, identify its main point, source section when applicable, on-slide content,
visual treatment and relevant notes or timing. Keep the plan lightweight; do not
require a separate file or approval unless the user requests it. If the user asks
for an outline only or an outline for approval, stop at that requested deliverable.

Honor an exact slide count, including cover and closing slides. Check source
coverage and sequence before authoring. Keep spoken explanation in speaker notes
and use concise content on the slide, unless a script or reading-deck request
specifies otherwise. An infeasible combination of timing, count and verbatim text
needs a focused clarification, not silent cuts or unreadably small type.

Initialize a new or empty directory; never initialize over an existing deck:

```sh
presmith init my-talk
cd my-talk
presmith setup
presmith doctor --json
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
presmith check --slide SLIDE_ID --json
presmith render --slide SLIDE_ID --json
```

When shared CSS, scripts or runtime behavior changes, check and render the whole
deck instead. Inspect every slide potentially affected by the shared change.

## Check, render and inspect

Run commands from the deck directory, or pass its path explicitly. For a full deck:

```sh
presmith check --json
presmith render --json
```

Use `presmith dev --open` for an interactive preview; checks and exports start
their own temporary server and do not need preview running. See the [CLI reference](references/cli.md)
for options, output paths and JSON results, and [diagnostics](references/diagnostics.md)
for interpreting findings. Exit 1 means deck problems; exit 2 means a usage or
operational failure. Warning-only checks may succeed; still assess the warnings.

**Open the rendered contact sheet and relevant full-size slide PNGs with available
image-viewing tools. Generating screenshots is not visual inspection.** For a new
deck, inspect every slide. Check hierarchy, readable text, spacing, clipping,
diagram labels, chart units and consistency. If image-viewing tools are unavailable,
provide the image paths and explicitly mark visual review as unverified.

Also check the selected workflow's content constraints: script coverage, supplied
order/count, evidence and units, or demo states. Layout diagnostics cannot verify
these. Report material omissions, estimates and unavailable source material.

Use at most three check → render → inspect → repair passes per request. Stop sooner
when satisfactory. After the third pass, report unresolved issues and the next
specific action. Do not suppress a finding by marking meaningful content decorative;
the decorative-overflow convention is only for nonessential artwork. A clean check
does not establish narrative quality, factual accuracy or visual polish.

## Export and hand off

Follow [export behavior](references/export.md) and export the requested format:

```sh
presmith export --format html --json
presmith export --format pdf --json
presmith export --format pptx --json
```

Run only the requested formats. Use artifact paths returned by the command.
PDF must report one page per slide. View exported HTML through an ordinary static
server; `file://` is not supported. Interactive content needs a defined export
state, and system-font differences can affect appearance across operating systems.
HTML exports include speaker notes in the embedded manifest.

For visual editing in PowerPoint or Google Slides, export PPTX. Text lines and basic
shapes remain editable; read `editability` counts and `pptx.rasterized` warnings to
explain which elements became pictures. PPTX includes speaker notes. Inspect an
Office-rendered preview when available; HTML screenshots do not verify PPTX layout.
Fonts can substitute, and imported edits do not synchronize back to HTML. If the
deck has an older renderer, use `presmith setup --upgrade-renderer` within existing
setup authorization; it backs up replaced tooling and preserves authored sources.

Report what changed, artifact paths, diagnostics actually run, images actually
inspected, and unresolved limitations. If rendering or export failed, identify the
blocker rather than presenting an older output as the new result. Ordinary deck
authoring does not include installing the Codex plugin, changing global Codex
configuration, publishing packages, or deploying the presentation.
