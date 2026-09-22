# Use cases and recipes

Use this guide to choose a workflow and carry it through authoring, review and
handoff. The [command reference](cli.md) documents every argument, option, output
and exit code. These recipes use the installed `presmith` command.

Presmith stores the presentation as local HTML/CSS/JS plus `deck.json`. You can
write those files yourself, work with a coding agent, or use the visual editor
for supported changes. The CLI has no model API, document-import command or
natural-language generation command. The optional `$presmith` skill helps a coding
agent plan and edit presentations; the CLI is installed separately.

## Contents

- [Choose a workflow](#choose-a-workflow)
- [Create and deliver your first deck](#create-and-deliver-your-first-deck)
- [Create from different source material](#create-from-different-source-material)
- [Revise one slide](#revise-one-slide)
- [Edit visually](#edit-visually)
- [Change the theme or slide structure](#change-the-theme-or-slide-structure)
- [Build an interactive demo with useful static exports](#build-an-interactive-demo-with-useful-static-exports)
- [Choose a delivery format](#choose-a-delivery-format)
- [Upgrade an existing deck](#upgrade-an-existing-deck)
- [Automate checks and exports](#automate-checks-and-exports)
- [Recover from editing or rendering problems](#recover-from-editing-or-rendering-problems)
- [Review before handoff](#review-before-handoff)

## Choose a workflow

| Goal | Typical input | Workflow |
|---|---|---|
| Explain an idea or propose a decision | Brief, audience, supporting facts | Init → outline → author → check → render → export |
| Support a talk | Script/transcript and timing | Map narrative beats to slides, preserve speech in notes |
| Follow an exact requested structure | Outline or slide-by-slide specification | Map slots to stable IDs and preserve count/order |
| Summarize evidence | Reports, meeting notes, research | Extract traceable claims, retain qualifications, author slides |
| Present results | CSV, spreadsheet, metrics | Verify units/calculations, build labeled charts, inspect exports |
| Adapt an existing presentation | Source deck or visual reference | Revise source or reconstruct with available tools |
| Demonstrate a product or process | Screenshots, steps, local interaction | Build sequence and static export states |
| Fix one slide | Existing Presmith project and feedback | Edit → targeted check/render → full check |
| Adjust appearance without writing HTML | Existing Presmith project | `edit` → Save → check/render |
| Share a web presentation | Validated deck | HTML export → static server/hosting |
| Send a printable or fixed-layout copy | Validated deck | PDF export → inspect every page |
| Hand off editable slides | Validated deck | PPTX export → inspect in PowerPoint/Google Slides |
| Run an automated quality gate | Deck sources and prepared runtime | Doctor → check JSON → render/export |

## Create and deliver your first deck

Start in the directory where the new project should live:

```sh
presmith init my-talk
cd my-talk
presmith setup
presmith doctor
presmith dev --open
```

The last command stays running. Use a second terminal in `my-talk` for subsequent
commands. The scaffold has three slides: `intro`, `workflow` and `next`, ordered
by `deck.json`. It starts at 1280 × 720 with the Ink theme.

Author the content by editing `slides/*.html`, `styles/custom.css` and
`scripts/custom.js`, or launch `presmith edit --open` in another terminal. Add
local assets under `assets/`. Keep titles, slide order and notes in `deck.json`.
See [Authoring](authoring.md) for valid fragments, paths and CSS scoping.

When the content is ready:

```sh
presmith check --json
presmith render
```

Read the diagnostics and open `.decksmith/render/contact-sheet.png`, then inspect
individual PNGs under `.decksmith/render/slides/`. Repair issues and repeat the
relevant check/render steps. The contact sheet helps compare consistency; full-size
images reveal small text, clipping and awkward spacing.

Export the formats the audience needs:

```sh
presmith export --format html
presmith export --format pdf
presmith export --format pptx
```

Outputs are `dist/html/`, `dist/deck.pdf` and `dist/deck.pptx`. Inspect the final
format as well as the PNGs. Preview/edit can be used before setup; browser checks
and exports need the renderer installation.

## Create from different source material

The following are authoring workflows, not additional CLI subcommands. With the
optional skill installed, give the coding agent the matching prompt. Without the
skill, the same planning steps apply when editing source manually. Combine input
roles when useful: an outline can define order, a script can supply notes, a report
can support facts, and an existing deck can guide the visual style.

### A brief or idea

Use for proposals, explanations, lessons, project updates or introductions. Start
with the audience, purpose, desired decision and approximate duration. Draft a
slide plan, then turn each idea into a stable slide ID and an HTML fragment.
Distinguish supplied facts from assumptions and placeholders.

> Use $presmith to create an eight-slide engineering proposal from these incident
> notes. Audience: backend leads. Compare the two proposed options and end with
> the decision needed. Preserve supplied metrics and label missing evidence.

Validate both the story and the layout: `check` cannot decide whether a conclusion
follows from the evidence.

### A speech, narration script or transcript

Map semantic beats and explicit slide cues to slides. Keep on-screen wording
short and preserve the corresponding spoken wording in `deck.json` speaker notes
when requested. Retain useful timestamps, speaker labels and stage directions
in notes. Read through the narration with the slide sequence to check coverage.

> Use $presmith to turn this keynote script into 12 slides. Keep my spoken wording
> in speaker notes, follow the sequence and use minimal text on screen.

HTML and PPTX include notes; PNG and PDF do not. Script length alone does not
prove presentation duration, so rehearse the result.

### A supplied outline or exact slide structure

Treat numbered slide slots and exact titles as constraints. Map each slot to one
stable ID, preserve order, and check the manifest against the requested count.
An exact six-slide request includes any cover or closing slide within those six.
A broad section outline can span multiple slides if no fixed count was requested.

> Use $presmith to build exactly the six slides in this outline. Keep the titles
> and order. Use this report for supporting facts and add no extra slides.

A Markdown table or JSON/YAML brief is planning input; it is not automatically a
valid `deck.json` file.

### Reports, articles or meeting notes

Extract the audience's main questions, the relevant claims and their evidence.
Keep filename/page/section references in notes and visible citations where needed.
Preserve qualifications and explain conflicting sources. Summarize around the
presentation's purpose rather than mechanically assigning one slide per page.

> Use $presmith to turn these incident reports into a seven-slide leadership
> briefing. Explain the common causes and end with the decisions needed. Keep
> source references and material caveats.

Reading a PDF or office document depends on the tools available to the authoring
agent. Presmith itself does not parse or import those formats.

### Data and metrics

Confirm units, periods, denominators and missing values before calculating changes
or choosing charts. Retain source references and make calculations traceable.
Missing values are not zero. Label illustrative scenarios and distinguish
observations from forecasts.

> Use $presmith to build a five-slide monthly review from this CSV. Compare
> actuals with targets, show units and explain missing values without guessing.

Local DOM/SVG charts work well with the browser runtime. Check narrative numbers
against the data and rendered labels. PPTX export does not turn those visuals
into native Office charts with editable data tables.

### An existing deck, a translation or an audience-specific version

For a Presmith project, read `deck.json`, notes and sources before changing them.
Preserve stable IDs and unrelated content. For a separate variant, copy the source
project to a separate directory and initialize its renderer with `setup` as needed;
`init` does not clone existing decks. Recheck translated text for expansion.

> Use $presmith to make a separate six-slide customer version of this technical
> deck. Keep its visual style and supported metrics. Explain what you cut.

For PPTX, PDF or screenshots, inspect the material using suitable external tools
and reconstruct the requested HTML presentation. There is no `presmith import`
command and no round-trip synchronization with PowerPoint or Google Slides.
A screenshot cannot reveal speaker notes or the original editable objects.

### A walkthrough, lesson or product demo

Map steps to what the audience sees, the interaction and the explanation in notes.
Use supplied screenshots for real product claims; label mockups or simulations.
Give every interactive slide a useful static state for PNG/PDF/PPTX.

> Use $presmith to turn these onboarding steps and screenshots into a six-slide
> walkthrough. Add one local interactive example and make the static exports clear.

Follow the [interactive demo recipe](#build-an-interactive-demo-with-useful-static-exports)
and test both the live interaction and export state.

## Revise one slide

From inside an existing deck, make the requested change to the slide source or
its scoped styles. Preserve its ID and the `data-element-id` values used to locate
content. In the starter deck, the middle slide is `workflow`:

```sh
presmith check --slide workflow --json
presmith render --slide workflow --out .decksmith/revision --json
```

Inspect `.decksmith/revision/slides/workflow.png` and its contact sheet. A separate
output folder retains the previous whole-deck render. Reusing `.decksmith/render`
for a filtered render would replace it with only the selected slide's artifacts.

After accepting the change, check the entire deck and refresh deliverables:

```sh
presmith check --json
presmith render
presmith export --format pdf
```

Shared CSS/scripts can affect other slides, so a targeted pass is not sufficient
for the final handoff. When working in Git, review the source diff to confirm that
unrelated slides and facts stayed intact.

Example agent request:

> On slide workflow, shorten the headline and enlarge the diagram labels. Preserve
> the evidence, slide IDs and all other slides. Check and visually inspect the
> changed slide, then check the whole deck.

## Edit visually

```sh
presmith edit my-talk --open
```

1. Select a slide and element; use **Parent** when a containing layout needs editing.
2. Edit supported text, typography, color, spacing, borders or image properties.
   Use supported resize/move handles where available.
3. Update speaker notes or reorder slides if needed.
4. Finish canvas text editing by clicking outside the text, then choose **Save**.
5. Use Undo/Redo to refine changes. Saving retains session history; save again to
   persist a later undo.
6. Run check/render and inspect the result. Save before using the editor's Export
   menu or invoking CLI export.

GUI style overrides are stored in `styles/editor.css`, scoped to slide and element
IDs. A property's reset button removes its override and reveals the authored CSS.
Opening the editor alone does not change the source. See [Visual editor](editor.md)
for supported geometry, image formats, rich text and controls disabled by source
constraints.

Adding/deleting slides, restructuring arbitrary HTML and implementing widgets
require source edits. If a coding agent changes files while the editor has pending
changes, follow the conflict workflow instead of expecting an automatic merge.

## Change the theme or slide structure

For the starter's light theme, replace `styles/theme.css` with `styles/paper.css`
in the `styles` array of `deck.json`. Keep `styles/custom.css` and any registered
`styles/editor.css` entry in their intended order. GUI overrides may need resetting
if they still impose colors or typography from the previous design.

To add a slide, create a fragment under `slides/` and add an entry with a unique
stable `id` and `source` to the manifest's `slides` array. To reorder, move manifest
entries or use the editor. To remove a slide, remove its manifest entry and review
its associated scripts/assets before deleting any shared files.

After changing global styles, dimensions or structure:

```sh
presmith check my-talk --json
presmith render my-talk
```

Review every slide, then export again. Filenames and hashes follow stable IDs;
numeric filename prefixes are not needed. See [Authoring](authoring.md) for the
manifest schema, valid IDs and slide count/dimension limits.

## Build an interactive demo with useful static exports

Register slide behavior in a script listed in `deck.json` using
`Decksmith.register(id, { init, export })`. The `init` hook sets up the interaction
and awaits required work. The `export` hook sets a repeatable, meaningful static
state and awaits drawing completion. See [Runtime](runtime.md) for the full API
and example hook syntax.

For a slider chart, choose a documented default scenario for export, update the
chart and labels to that scenario, and stop any timers. Async work needed by the
capture belongs in returned promises. CSS motion suppression does not stop
arbitrary JavaScript timers or fetch data for you. Keep required resources local;
browser diagnostics and exports block external requests.

Assuming the interactive slide's ID is `scenario`:

```sh
presmith dev my-talk --open
# In another terminal, after trying the live controls:
presmith check my-talk --slide scenario --json
presmith render my-talk --slide scenario --out my-talk/.decksmith/demo-review
presmith export my-talk --format html
presmith export my-talk --format pdf
```

Open the PNG to verify the static state. Serve the HTML export and retest live
interaction separately. PDF/PPTX cannot retain the controls' behavior; complex
interactive elements may become pictures in PPTX.

## Choose a delivery format

| Audience need | Choose | Review before sharing |
|---|---|---|
| Present in a browser with authored interaction | HTML | Static-server navigation, local assets and live controls |
| Email a fixed document or print slides | PDF | One page per slide, fonts, static states and legibility |
| Let colleagues edit slides in PowerPoint/Google Slides | PPTX | Font substitution, line boxes, rasterization and notes |
| Get design feedback or embed slide images | PNG via `render` | Individual full-size slides plus contact sheet |

From inside the deck:

```sh
presmith check --json
presmith render
# After inspecting the images, choose the required deliverables:
presmith export --format html
presmith export --format pdf
presmith export --format pptx --json
```

To preview a web handoff:

```sh
python3 -m http.server 8080 --directory dist/html
```

Open `http://localhost:8080`. Share/host the complete directory, not just
`index.html`, so referenced styles, scripts and assets remain available. Presmith
has no hosting or deployment command.

For a PowerPoint handoff, open `dist/deck.pptx` in the target application, or upload
it to Google Drive and open with Google Slides. Check the imported slides there;
browser rendering is not an Office import test. Text and simple geometry remain
editable, while complex effects can be rasterized. Edits do not flow back to HTML.

Review notes before sharing HTML or PPTX: both include them. PDF and PNG omit
speaker notes. Rebuild exports after any source revision; old deliverables do not
update themselves.

## Upgrade an existing deck

Updating the CLI provides a newer embedded renderer for future projects, but does
not modify the tooling already copied into an existing deck. After updating the
CLI, run:

```sh
presmith setup my-talk --upgrade-renderer
presmith doctor my-talk --json
presmith check my-talk --json
presmith render my-talk
```

The upgrade keeps backups of replaced renderer files under
`my-talk/.decksmith/renderer-backups/`. Review new findings and rendered images,
then regenerate the desired exports. Confirm `pptx_export` is true if you need
PowerPoint output. This flag updates renderer tooling, not authored slides,
runtime library files or existing copies of documentation.

## Automate checks and exports

Use this shell script from inside a deck. Provision Node.js/npm first and run
`presmith setup` explicitly when the job needs its local dependencies. That setup
step needs network access; subsequent browser checks use local resources.

```sh
#!/bin/sh
set -eu

# Keep reports outside directories that render/export will replace.
mkdir -p .decksmith/reports
presmith doctor --json > .decksmith/reports/doctor.json

if presmith check --json > .decksmith/reports/check.json; then
  presmith render --json > .decksmith/reports/render.json
  presmith export --format html --json > .decksmith/reports/html.json
  presmith export --format pdf --json > .decksmith/reports/pdf.json
else
  check_status=$?
  cat .decksmith/reports/check.json
  exit "$check_status"
fi
```

The script preserves a failing check's status and stops subsequent operations
when a command fails. It stores stdout JSON while leaving stderr visible. Avoid
piping directly to a JSON formatter unless the shell also preserves the CLI's exit
status; otherwise the formatter's success may mask a failed command.

Warnings alone pass with exit 0. If your workflow treats selected warnings as
failures, inspect `findings` and impose that policy in the caller. Doctor may
succeed without PPTX support, so inspect `capabilities.pptx_export` before relying
on it. Rendering and exporting do not replace the full `check` step or human
visual review.

Archive the current run's reports, PNGs and successful exports as review evidence.
When reusing a workspace, do not label existing outputs from a failed run as fresh;
failed checks can leave the previous artifacts in place. There is no separate
`presmith build`, `presmith test` or `presmith ci` command.

## Recover from editing or rendering problems

**Missing browser dependencies:** run `presmith doctor my-talk --json`, inspect the
reported error, and run `presmith setup my-talk` when installation is needed. A
working preview does not imply that rendering dependencies are installed.

**Older PPTX renderer:** run `presmith setup my-talk --upgrade-renderer`. Updating
only the binary leaves existing renderer files unchanged.

**Source changed during visual editing:** download pending edits from the conflict
banner, explicitly reload/discard the stale session, then reconcile the downloaded
record with the changed source. It is a record for manual reconciliation, not an
automatic merge/import file.

**Interrupted editor save:** close other editors, then run:

```sh
presmith edit my-talk --recover --open
```

This rolls back a journaled save when safe. It cannot restore unsaved changes lost
with a browser session. Conflicting external edits stop recovery for manual review.

**Blocked output path:** choose a new/empty directory or an existing managed output.
Within the deck, use `dist/` or `.decksmith/`. Keep reports outside the render/HTML
output directory so successful reruns do not erase them.

For other failures, see the [troubleshooting table](cli.md#troubleshooting) and
[diagnostic repair guidance](diagnostics.md).

## Review before handoff

Confirm that the intended content, facts, citations, slide count/order and notes
survived authoring. Run the whole-deck check, inspect the contact sheet and full-size
PNGs, and inspect each requested final format. Test live demos in the browser and
static states in exports. Report any unresolved findings or format limitations
alongside the output paths. A clean automated check is evidence about the rules it
covers, not a judgment of narrative or visual quality.
