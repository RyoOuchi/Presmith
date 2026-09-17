# Source conventions

Read the deck's docs/authoring.md and docs/runtime.md if present. Manifest v1 has
title, width/height (1280×720 default), ordered slides {id,source,title?,notes?},
styles[], scripts[]. A source is an HTML fragment, automatically wrapped in
section.deck-slide with data-slide-id and data-source. Reordering manifest entries
does not require renaming files. IDs use ASCII letters/digits/hyphens/underscores.

Add unique data-element-id values within each slide to important elements, e.g.
headline, key-stat, diagram, caption. HTML DOM ids should be globally unique.
Use assets/image.svg in fragments, ../assets/image.svg from a stylesheet. Assets
resolve from the assembled project root; avoid leading slashes and remote URLs.
Only assets/, lib/, styles/, scripts/ are public. No secrets there. Store notes in
the manifest; exported HTML includes them in the embedded runtime manifest.

Register custom script hooks synchronously after Decksmith loads:
`Decksmith.register('scenario', {async init(slide) {}, async export(slide) {}})`.
Initialization runs once per active slide. Await necessary async work. Export hooks
must reset interactive state deterministically and be repeatable. Stop timers.

Authored: deck.json, slides/, styles/, scripts/, assets/. Bundled lib/ is shared.
Generated: .decksmith/ and dist/. Never edit generated index.html to fix a source bug.
