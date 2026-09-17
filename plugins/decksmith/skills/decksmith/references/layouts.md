# Layout vocabulary

Use shared tokens --bg, --fg, --muted, --accent, --line, --surface, --space,
--slide-padding. Ink is dark technical; Paper is warm editorial. Change themes
only when the request is about the deck-wide style. Default body 26px, h2 58px,
h1 88px, captions 18px. Prefer shorter text over shrinking type.

Title / closing:
```html
<div class="title-layout">
  <p class="eyebrow" data-element-id="eyebrow">Audience / context</p>
  <h1 data-element-id="headline">The central <em>idea.</em></h1>
  <p class="lead" data-element-id="subtitle">One supporting thought.</p>
</div>
```

Section divider: .section-number + h1. Two-column explanation:
```html
<h2 data-element-id="headline">The claim</h2>
<div class="split"><p>The explanation.</p><figure>Evidence and caption.</figure></div>
```

Large statistic: .stat + .stat-label, with a cited source or “Illustrative”.
Comparison: table.comparison with th/td. Process/timeline: ol.process with li,
span.step, h3 and p. Keep to three steps at the default dimensions.
Chart/diagram: SVG.diagram with viewBox, accessible title, legible labels and units.
Image: figure > img[src="assets/local-image.svg"][alt] + figcaption.
Code: pre > code with escaped HTML, short lines and a useful explanation.

Use .stack, .rule, .caption and .muted sparingly. Avoid repeated card grids.
A nonessential cropped shape may carry both data-decksmith-overflow="decorative"
and aria-hidden="true". Never mark meaningful content decorative to hide a finding.
