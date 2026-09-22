# Creation workflows

Use the section matching the supplied input, then return to the shared authoring,
check, render, inspection and export steps in SKILL.md. These workflows do not add
CLI commands. Accept natural-language requests and existing file formats; do not
require the user to convert their input into a special schema.

For mixed inputs, assign each source a role: structure, narration, evidence or
visual style. A supplied outline can control order while a script supplies notes.
If a required source cannot be read, identify the missing input and continue only
work that does not depend on it. Do not invent its contents.

## Brief

Use for a topic, goal or loose collection of ideas.

- Infer audience, purpose, duration and tone. Ask only when missing context changes
  the result materially. Use a reasonable slide count unless the user specifies one.
- Develop a narrative suited to the task: explanation, proposal, teaching, update,
  pitch or decision. Do not impose a pitch structure on every presentation.
- Separate user-provided facts from assumptions and proposed claims. Use accessible
  sources when research is part of the task; otherwise expose evidence gaps instead
  of manufacturing numbers, customer quotes or citations.
- Draft the slide plan and continue into authoring unless the user asked to review
  the outline first. A plan-only request does not authorize creating a full deck.

Example: “Use $presmith to create a ten-minute introduction to our local developer
tool for backend engineers. Use this README as the factual source.”

## Script

Use for a speech, narration script, transcript or timestamped talk. Here, “script”
means spoken content. If the input is executable code, inspect what it contains and
treat it as demo or source material rather than assuming it is narration.

- Read the entire script, including speaker labels, timestamps and stage directions.
  Mark semantic beats and explicit slide cues. A paragraph is not automatically a
  slide; combine or split at changes in the idea, evidence or visual.
- Map each beat to a stable slide ID. Preserve the intended sequence, key claims,
  transitions, quotations and any requested slide boundaries. Make gaps or repeated
  coverage visible in the working plan before creating slide fragments.
- Put concise headlines, evidence and visuals on screen. Keep the corresponding
  spoken wording in `deck.json` notes unless the user requests a rewrite or summary.
  Keep timestamps, speaker labels and stage cues as notes rather than slide copy.
  If verbatim narration is requested, preserve its wording, including meaningful
  repetitions. Do not silently shorten the speech to fit the slides.
- Use supplied timing when available. Otherwise label duration as an estimate and
  state the assumed speaking rate if calculating it from word count. Account for
  pauses and demo time; do not promise exact timing from word count alone.
- Check that every required passage has a slide/notes destination and that slide
  transitions still make sense when reading the narration aloud. Notes travel in
  HTML and PPTX exports, so keep the shared export guidance in mind.

Example: “Use $presmith to turn this keynote script into 12 slides. Keep my spoken
wording in speaker notes, follow the existing sequence, and use minimal text on screen.”

## Structure

Use for an agenda, numbered outline, slide-by-slide spec, Markdown table, or
structured JSON/YAML content. Read it as a user brief, not as `deck.json` syntax.

- Determine whether entries are slide slots or broad sections. Explicit slide
  numbers and counts are binding; broad sections may span slides when no count is
  fixed. If the distinction materially changes the result and is unclear, ask.
- Preserve requested order, titles marked exact, required points, evidence, notes
  and layout constraints. Treat headings as editable only when the request allows
  it. Do not replace a supplied narrative with a generic template.
- An exact six-slide structure means six slides total. Do not add a cover, agenda,
  divider or closing slide outside those slots. With a flexible outline, propose
  useful splits or combinations in the plan and state the assumption.
- Map source entries to stable IDs. Fill gaps only to the extent requested; label
  assumptions or placeholders and ask for facts needed to complete a required claim.
- Before handoff, compare the manifest and slide content against every required
  slot. Check count, order and coverage separately from visual layout.

A user may provide a lightweight spec like this; no fixed column names are required:

| Slide | Exact title | Required content | Visual direction |
|---|---|---|---|
| 1 | The current workflow | Supplied process steps | Process diagram |
| 2 | Where time goes | Timing data with units | Bar chart |
| 3 | The proposed change | Two supplied options and tradeoffs | Comparison |

Example: “Use $presmith to create exactly these three slides in this order. Keep
the titles as written and use the attached notes to fill each required section.”

## Documents

Use for reports, articles, meeting notes, specifications or a research packet.

- Extract the questions, claims and evidence relevant to the audience. Keep source
  anchors such as filename and section, page, or URL so facts remain traceable.
- Build a presentation narrative rather than one slide per source page. Preserve
  qualifications, conflicting results and limitations that affect the conclusion.
- Separate quotations, paraphrases, interpretation and recommendations. Keep source
  references in notes and visible citations where the claim needs attribution.
- If a user gives multiple sources, reconcile them explicitly; do not quietly use
  the most convenient number. An inaccessible source remains a stated gap.
- Check that major conclusions have support and that condensation has not removed
  a caveat or reversed the meaning of the original material.

Example: “Use $presmith to turn these three incident reports into a seven-slide
leadership briefing. Explain the common causes and end with the decisions needed.”

## Data

Use for tables, CSVs, spreadsheets, experiment results or supplied metrics.

- Inspect column meanings, units, periods, population/denominators and missing
  values before choosing claims or chart types. Missing is not zero. Clarify an
  ambiguity when it changes the chart or conclusion.
- Select comparisons that answer the presentation's question. Preserve labels,
  baselines, uncertainty and scale. Distinguish observations from projections or
  illustrative scenarios, and avoid causal claims unsupported by the data.
- Keep derived values traceable to the source and calculation. Calculate changes,
  totals and percentages rather than inventing or eyeballing them.
- Prefer local DOM/SVG charts when suitable. Retain the underlying supplied data
  and source references as appropriate for the deliverable. Do not promise native
  editable Office charts merely because the HTML chart is editable.
- Cross-check the narrative numbers against the source and rendered chart labels.
  Review units, axes, legends and any export state for interactive charts.

Example: “Use $presmith to build a five-slide monthly review from this CSV. Compare
actuals with targets, show units, and explain missing values without guessing.”

## Existing deck

Use when adapting an existing presentation, following a visual reference, or
creating a version for a new audience, duration or language.

- Establish the role of the reference: content, design, or both. Follow explicit
  constraints. A document supplied for facts is not automatically a design template.
- For a Presmith project, read the manifest, notes and sources and use the revision
  workflow. Preserve IDs and unrelated content. Create a separate project when the
  user asks for a variant or copy, keeping the original intact.
- For another format, inspect what available tools can actually read. Reconstruct
  an HTML deck when requested; do not claim a Presmith import command or automatic
  conversion. Images of slides do not expose hidden notes or editable source.
- Match relevant typography, colors, layouts and asset proportions when asked to
  follow a reference. If branding assets are missing, identify the gap rather than
  fabricating an exact logo or claiming a faithful match.
- For shorter or audience-specific variants, preserve evidence and citations while
  changing emphasis. Record material cuts in the handoff. For translation, retain
  numbers, units and meaning and recheck layout for text expansion.

Example: “Use $presmith to make a separate six-slide customer version of this
technical deck. Keep its visual style and supported metrics. Explain what you cut.”

## Demo

Use for a product walkthrough, lesson, sequence of screenshots or storyboard.

- Map each step to what the audience should see, the action or transition, and the
  explanation in speaker notes. Include setup and outcome only when relevant to
  the requested scope and slide count.
- Use supplied screenshots, code or local assets for real product claims. Label
  mockups or simulated states. Do not present invented UI as a captured product.
- Choose live interaction only where it helps the demonstration. Define an initial
  state and a meaningful static state for each interactive slide. Register required
  initialization and repeatable export hooks using the authoring/runtime contract.
- Keep static exports understandable without clicking, hovering or waiting. Give
  animations, videos or live-dependent steps a suitable still state or explanation.
- Test the intended control sequence and inspect its static export state. A
  screenshot of the initial state does not verify the live demonstration.

Example: “Use $presmith to turn these onboarding steps and screenshots into a
six-slide walkthrough. Add one local interactive example and make PDF states clear.”
