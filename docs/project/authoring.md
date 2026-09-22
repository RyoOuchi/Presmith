# Authoring format v1

`deck.json` contains `schema_version: 1`, a nonempty `title`, integer `width` and
`height` (default 1280 × 720), ordered `slides`, and explicit `styles` / `scripts`
arrays. Each slide has `id`, `source`, and optional `title` and `notes`. Additional metadata fields are preserved by assembly and the visual editor. Limits: 1–200 slides, width 320–4096, height
240–4096. IDs contain 1–80 ASCII letters, digits, `_` or `-` and are unique ignoring ASCII case, so filenames remain portable.

```json
{"id":"solution","source":"slides/solution.html","title":"The solution","notes":"Explain the tradeoff."}
```

Reorder this list, not filenames. Each fragment is inserted into
`<section class="deck-slide" data-slide-id="solution" data-source="slides/solution.html">`.
Do not include html/head/body tags or another slide wrapper. Use
`data-element-id="headline"` on important elements; values must be unique within
that slide. HTML `id` attributes should be globally unique because the browser
uses them globally (the checker checks duplicates within each slide).

All HTML asset URLs resolve from the **project root** in the assembled page:
`<img src="assets/diagram.svg">`, even in slides nested in subdirectories.
CSS `url(...)` paths resolve from the stylesheet: `url('../assets/photo.png')`.
Use relative URLs without a leading slash. Module imports resolve from their
script. Runtime requests should use project-relative paths, with `fetch` included
in a registered initialization promise when the result is needed for rendering.
Never depend on external fonts, CDNs or APIs for a portable deck.

Only index.html plus nonhidden files in `assets/`, `styles/`, `scripts/`, and
`lib/` are served/exported. Place public resources there. No directory listings,
source fragments, manifest, AGENTS, tooling, dotfiles or repository files are
served. `node_modules` is excluded everywhere. Symlinks in public directories
are rejected; manifest paths cannot leave the canonical project root. Every file
in these four public directories is public, even if unused: do not store secrets there.
Custom code is trusted project code, not sandboxed code; inspect unfamiliar decks.

Scope local CSS: `[data-slide-id="solution"] [data-element-id="headline"] { … }`.
Set shared tokens in styles/theme.css and put local rules in styles/custom.css.
Library tokens cover colors, typography, slide padding and spacing. Default text
is 26px, headings 58/88px, captions 18px; the 16px slide folio is a secondary label.
The Paper theme is styles/paper.css; replace styles/theme.css in the manifest.

Authored files: deck.json, slides/, styles/, scripts/, assets/. Bundled editable
runtime: lib/. Renderer source and pinned lockfile: tooling/renderer/. Generated
files: .decksmith/, dist/, node_modules/, .browsers/, .npm-cache/. Never edit
rendered HTML instead of its source. Commit authored source and both manifests;
ignore installed dependencies and generated outputs.

Patterns: `.title-layout`, `.split`, `.stat` + `.stat-label`, `.comparison`,
`.process` + `.step`, `.diagram`, `figure` + `figcaption`, `pre > code`, `.stack`,
`.eyebrow`, `.lead`, `.caption`, `.rule`. Section dividers can combine
`.section-number` and a large heading. A closing slide uses `.title-layout` with
one action. Image pattern: `<figure><img src="assets/image.svg" alt="Meaningful
summary"><figcaption>Source or explanation</figcaption></figure>`. Use licensed
local assets and semantic HTML. Avoid filling every layout with decorative cards.
