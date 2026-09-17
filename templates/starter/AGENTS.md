# Working on this deck

Read deck.json and the relevant slide fragments before editing. Follow docs/authoring.md
and docs/runtime.md. Use the Decksmith skill if installed; the CLI never calls a model.
Draft a short narrative outline before creating slides. Preserve facts and label illustrative data.
Keep stable slide IDs and data-element-id attributes. For a local revision, modify only the
requested slide and selectors scoped to [data-slide-id="…"]. Do not change the global theme.
Run decksmith check --slide ID, then render and actually open the resulting PNG.
If shared CSS/JS changes, check and visually inspect the entire deck. Inspect the diff.
Try at most three repair passes, then report unresolved findings honestly.
Authored: deck.json, slides/, styles/, scripts/, assets/. Bundled runtime: lib/.
Generated: .decksmith/ and dist/; never fix the generated HTML instead of source.
