---
name: decksmith
description: Create, inspect, revise and export source-first HTML presentations in a Decksmith project using its Rust CLI. Use for decks with deck.json or an explicit request to use Decksmith; not for PowerPoint editing or general websites.
---

# Decksmith

Work in the user's local deck. The CLI does not call a model. Read its AGENTS.md,
deck.json and relevant source files before editing. Check `decksmith --help` and
`decksmith doctor --json` when command/runtime availability is unclear. If the CLI
is not installed, report the specific missing command; do not silently substitute
a different slide framework.

For new decks, infer audience, purpose, approximate length and style from the
request. Ask only about missing information that would materially change the
result; otherwise state reasonable assumptions. Draft a concise narrative outline
before writing slides. Preserve supplied facts and cite their supplied sources.
Label any invented example numbers as illustrative; do not manufacture evidence.

Use [authoring](references/authoring.md) for source/asset conventions and
[layouts](references/layouts.md) for HTML patterns. Reuse the shared spacing and
typography. Keep one point per slide and vary layout by content. Author HTML/CSS/JS
directly; never modify generated outputs to implement a source revision.

Use [CLI](references/cli.md) for commands. Initialize only a new directory. Setup
installs local dependencies and Chromium; run it when the request authorizes
installation, otherwise state the exact prerequisite. Do not change global Codex
configuration, install a plugin, publish packages or deploy a deck as part of
ordinary authoring.

Use [revision workflow](references/revision.md) for targeted changes. Preserve
stable slide and element IDs, unrelated slides, facts and notes. Scope CSS to the
requested slide; do not change theme tokens for a local request. Inspect the diff
and check affected slides first. If shared CSS or runtime changes, check the whole
deck and inspect all rendered slides.

Run `decksmith check --json` (or `--slide ID`) and interpret findings with
[diagnostics](references/diagnostics.md). Render PNGs and open the contact sheet
**and relevant full-size slides using available image-viewing tools**. Generating
screenshots is not visual inspection. Check readability, hierarchy, whitespace,
clipping, labels and consistency; fix actual problems. If image tools are absent,
report that visual review is unverified and provide the image paths.

Use at most three check → render → inspect → repair passes for one request. Stop
sooner when satisfactory. After three passes report unresolved findings and the
specific next action; do not conceal failures or keep changing unrelated content.

Export the requested format following [exports](references/export.md). Report
artifact paths, checks actually run, visual review actually performed and remaining
limitations. A successful diagnostic check alone is not a design-quality guarantee.
