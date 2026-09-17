# Interpreting diagnostics

`decksmith check [directory] [--slide ID] [--json]` validates the manifest first,
then activates each requested slide at logical size in Chromium. Shared JavaScript
and styles always load. External browser requests are blocked. Geometry is measured
in the deterministic export state so check/render/export agree.

Each finding includes `severity`, `rule_id`, `slide_id`, `element_id`, `source`,
`message`, and a `measurements` object. Unknown attribution uses null; initial global
script errors belong to scripts/ and can have no slide ID. Some request failures
include the active slide as context rather than a guaranteed initiator.

| Rule | Severity | Usual repair |
|---|---|---|
| `manifest.*` | error | Fix JSON/schema, duplicate/invalid ID, path or missing file |
| `element.duplicate_id` | error | Keep one stable data-element-id / HTML id per element |
| `asset.request_failed` | error | Fix URL, include local asset; remove external dependency |
| `asset.broken_image` | error | Fix image source/encoding |
| `runtime.javascript` | error | Fix uncaught script exception |
| `runtime.not_ready` | error | Resolve initialization or export promise within 10s |
| `layout.out_of_bounds` | error | Move/resize element into the logical slide (2px tolerance) |
| `text.overflow` | error | Shorten text or enlarge its container |
| `text.clipped` | error | Fix a clipping ancestor |
| `text.too_small` | warning | Prefer at least 18 logical pixels |

Visible-overflow text containers allow up to 0.3em of vertical font-metric overshoot;
clipped containers retain the strict 2px tolerance.

The library folio is intentionally exempt from the small-text warning. Warnings
alone return exit 0. A structural/geometry error returns exit 1; missing Chromium
or another operational problem returns exit 2. render/export reject failed assets,
JS errors and readiness errors but do not require all layout warnings/errors to be
resolved; run check separately. No overlap rule is used.

For **nonessential artwork only**, use both attributes on a decorative container:

```html
<div data-decksmith-overflow="decorative" aria-hidden="true">…</div>
```

This exempts that subtree from bounds and text layout checks, not broken assets or
JavaScript errors. Do not use it on meaningful text or interactive controls.
The starter title slide demonstrates a deliberately cropped circle.

Limitations: the checker is a conservative geometry heuristic, not a visual design
review. SVG internal geometry, canvas pixels, pseudo-elements, rotated shapes,
line-clamp, masks, complex inline layout, intentional clipping and occlusion are
incompletely understood. It may miss clipping or report a false positive. It does
not certify accessibility, contrast, semantics, narrative quality or factual truth.
Inspect actual PNGs and the final PDF. A clean check does not guarantee good design.
