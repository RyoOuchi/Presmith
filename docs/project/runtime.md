# Runtime contract v1

The same assembled index.html and styles power preview, checks, PNGs, HTML and
PDF. No bundler or model API exists. Scripts listed in the manifest run in order,
after lib/decksmith.js, before DOMContentLoaded. Register initialization at script
top level; do not wait for a remote import to register it.

```js
Decksmith.register('scenario', {
  async init(slide) { /* install event listeners; await required data/assets */ },
  async export(slide) { /* set a defined static value; await drawing */ }
});
```

Registration is optional and once per slide. `init` runs once, on first activation.
`export` can run repeatedly: make it idempotent. All asynchronous work needed for
capture belongs in the returned promises. Input handlers should leave keyboard
shortcuts alone. Prefer DOM/SVG for static charts. If using canvas, draw at logical
size and wait for completion inside the hooks. Stop timers in the export hook;
CSS animation suppression cannot stop arbitrary timers, clocks, random numbers,
video streams or network-driven content.

Public browser API `window.Decksmith`:

| Member | Contract |
|---|---|
| `version` | `1` |
| `listSlides()` | Ordered `{id,title,source,index}` objects |
| `select(id)` | Activate exactly one slide; update `#id`; unknown ID throws |
| `current()` | Active slide ID |
| `setExportMode(true)` | Hide controls, disable CSS animation/transitions, scale 1 |
| `ready(id?)` | Select; await init, static export hook, fonts, required images and two animation frames |
| `register(id, {init, export})` | Register author hooks |

Readiness phases time out after 10 seconds. Broken image decoding is reported by
the checker. The helper limits page navigation to 20 seconds and browser actions
to 15 seconds; Rust bounds each helper invocation to 10 minutes. There is no fixed
sleep used as a substitute for readiness, and no network-idle wait.

All slide images are required. Fonts wait on `document.fonts.ready`. CSS backgrounds
are checked for failed requests; their visual decoding is less deeply inspected.
If you add an async chart or background, wait for its readiness in `init`/`export`.
SVG animation, third-party widgets, animations created after export mode and
unregistered asynchronous work may require explicit author export hooks.

Navigation: arrows, PageUp/PageDown, Space, Home/End; F or the Fullscreen button.
Shortcuts ignore inputs, buttons, links, selects and contenteditable controls.
Inactive slides are hidden and inert. Hashes use stable IDs. Preview scales the
logical canvas proportionally and reserves room for controls. Fullscreen depends
on browser support and a user gesture. Reduced motion is honored.

PNG output uses one CSS pixel per logical pixel. PDF uses explicit CSS page size,
print backgrounds, zero margins/headers/footers, selectable browser text, and an
independently parsed page-count check. All slides are prepared individually before
printing in manifest order. Authors can still break printing with custom print CSS;
a page-count mismatch is an operational failure, never reported as success.

HTML export is assembled, verified over HTTP with external requests blocked and
keyboard/hash navigation exercised. Serve its directory through any ordinary
static server, including under a URL subdirectory. Rust/Node are not required by
the exported presentation. file:// is not supported. Dependencies and notes source
files are not exported separately; notes remain embedded in the runtime manifest,
so exported HTML includes speaker notes. Do not put confidential notes in a shared deck.

Pinning Playwright/Chromium improves repeatability on one platform. System-font
metrics, font versions, OS rasterization and browser/platform differences prevent
a promise of universal pixel identity. PDF cannot preserve interactive controls.

PPTX export measures each active slide after the same readiness/export hooks.
Text becomes editable line segments using measured positions, font sizes, colors
and emphasis. Solid fills, borders, rectangles, ellipses and simple SVG shapes/text
remain native objects. Images remain pictures. Complex SVG, canvas, controls,
generated content, clipping, transforms, gradients and other unsupported effects
use bitmap fallbacks with `pptx.rasterized` warnings. `data-pptx="raster"` explicitly
requests a picture. This conversion covers common source layouts, not every CSS
painting rule. Fonts and text metrics can change in Office/Google Slides.
System sans-serif and common macOS Helvetica/SF faces map to Arial; macOS UI
monospace faces map to Courier New. Other installed font names are retained.

PPTX preserves slide dimensions, order and speaker notes, validates ZIP integrity
and slide count, and reports per-slide editability counts. Notes are shared with the
file. Tables remain individual text/border objects, and charts do not acquire Office
data tables. Exported objects do not retain HTML behavior, links or CSS rules.
PPTX/Google Slides edits do not synchronize back to the original source.
