# Editable PPTX export (v0.2)

Run `presmith export PATH --format pptx --json`. The default file is
`PATH/dist/deck.pptx`. Open it in PowerPoint or upload it to Google Drive and open
with Google Slides. This is a one-way conversion; edits in those applications do
not update the HTML source.

## Conversion

The existing temporary HTTP server and Chromium renderer prepare each slide in
manifest order. The exporter uses the same readiness and static export hooks as
PNG/PDF. Browser Range geometry supplies text positions and wrapping. Each visible
line segment becomes a native text box, with font size, color, emphasis and letter
spacing. This preserves positioning while letting users edit the text. Paragraphs
may span several boxes, so edits do not reflow automatically between lines.

Solid backgrounds, borders, rectangles and ellipses become native shapes. Simple
SVG rect/circle/ellipse/line/text elements also remain editable. Pictures capture
the rendered image appearance and can be moved, resized or replaced. Complex SVG,
canvas, inputs, generated content and unsupported CSS effects become pictures with
`pptx.rasterized` warnings. Use `data-pptx="raster"` to explicitly flatten an element.

Each artifact reports `editability` by slide: text boxes, shapes, images and the
subset of images used for unsupported elements. Slide dimensions, order and
speaker notes are preserved. Notes are shared as part of the PPTX.

This does not implement every CSS painting rule. Fonts are not embedded. Generic
system sans faces and common macOS Helvetica/SF faces map to Arial, and macOS UI
monospace faces map to Courier New. Other installed font families are retained.
Check fonts, text spacing and overlapping content in the target application.
HTML tables become separate text/border objects, and HTML charts do not become
Office charts with editable data. Hyperlinks and JavaScript behavior do not carry
over.

## Existing decks

After updating the CLI, run `presmith setup PATH --upgrade-renderer`. It backs up
every replaced renderer file under `.decksmith/renderer-backups/`, refreshes the
embedded renderer sources and lockfile, then runs the normal local dependency
installation. Authored slides, styles and scripts, plus extra custom renderer
files, remain intact. Symlink renderer paths are rejected. Setup without the flag
keeps the existing tooling. Doctor reports `pptx_export` independently for old decks.

## Verification

Verified on macOS on 2026-09-21 with Decksmith 0.2.0 (now Presmith), pinned PptxGenJS 4.0.1,
JSZip 3.10.1 and Playwright 1.58.2:

- Rust tests cover asset embedding, upgrade backups, authored-source preservation,
  symlink rejection, old-renderer errors and output path protection.
- Browser integration opens the generated ZIP and inspects OOXML for native text,
  shapes and pictures, notes, slide dimensions/order and hidden-content exclusion.
  It checks export hooks, missing-font fallback, explicit raster requests and that
  failed exports preserve previous output. Existing HTML/PDF and cleanup tests pass.
- The eight-slide product deck and ten-slide showcase were exported, converted to
  PDF by headless LibreOffice and visually inspected. The showcase has 202 text
  boxes, 59 shapes and one static slider picture. The product's complex SVG diagram
  and slider are pictures; its simple SVG chart remains editable geometry/text.

Live Google Slides import and PowerPoint GUI editing were not verified. LibreOffice
rendering and OOXML checks establish the tested boundary; target applications may
substitute fonts or interpret text metrics differently.
