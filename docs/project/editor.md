# Visual editor

Run `presmith edit --open` inside a deck, or `presmith edit path/to/deck --open`.
The editor binds to `127.0.0.1:4174`; `--port 0` chooses a free port. The printed URL
contains a per-session credential in its fragment. Keep that URL private. Stop the
server with Ctrl-C. No frontend development server or Node installation is needed
for editing; existing browser checks/exports still need `presmith setup`.

The editor renders the assembled presentation in a sandboxed iframe. Click a
source element, or select it from **Source element** for keyboard access. Use
**Parent** to select a containing layout. Source scripts can run inside the frame,
but cannot access the editor's credential or invoke its save/export API. Generated
script nodes and unsupported source nodes are read-only. Slides with scripts inside
fragments or inline event handlers have read-only elements, because those scripts
can run before reliable source mapping. Move their initialization to a script
registered in deck.json with `Decksmith.register` to enable editing. Preview mode hides the
selection and enables presentation interaction. Export never includes the editor
bridge, overlays, session credential or controls.

## Supported changes

- Select slides using thumbnails. Drag to reorder, or use each slide's Up/Down
  buttons. The order is saved to `deck.json`, and reaches all exports.
- Double-click supported text on the canvas. Finish by clicking outside it. A
  focused text block is one undo transaction; composition input is allowed to
  finish before committing. Paste inserts plain text. The properties text field
  accepts text plus attribute-free `strong`, `em`, `b`, `i`, `u`, `s`, `code`, `br`.
  Existing supported emphasis and line breaks are preserved. Do not use this
  field for arbitrary HTML; layout containers, comments and other nested markup
  are intentionally read-only for text changes. Multiline canvas input uses `br`;
  in the formatting field, insert `<br>` for a visible line break.
- Typography: family, size, weight, italic, underline/strike, color and alignment.
  Colors use hex notation or `transparent`. Font availability follows the browser
  and deck's authored font files; the editor does not download fonts.
- Background, width/height, padding, border color/width/style/radius. Numeric
  geometry/spacing uses **px**. Flex/grid containers expose gaps, alignment and
  justification; flex direction and item order/alignment preserve the layout model.
- Resize block elements and images with the right, bottom or corner handle. Move
  absolutely positioned elements using the move handle when they have left/top
  anchors and no right/bottom constraints. Coordinates are relative to the
  existing containing block; translate transforms are preserved. One completed
  gesture is one undo step at any canvas zoom.
- Upload PNG, JPEG, WebP or GIF images, up to 10 MB, with bounded decoded dimensions
  (8192 per axis, 128 MB allocation limit). Files are validated by Rust and stored
  as `assets/editor-<SHA256>.<extension>`. Identical images reuse the same file;
  same-named different uploads cannot overwrite each other. Set alternative text
  and object-fit. SVG uploads and responsive `srcset`/`sizes` replacements are
  not supported; edit those assets/references in source.
- Speaker notes are saved in `deck.json` and included in HTML and PPTX as before.

Unsupported controls are disabled with an explanation on hover/focus. Geometry
for rotated, scaled or skewed elements/ancestors, inline text dimensions, relative
position dragging, free positioning of flex/grid children, rich widgets,
animations, and structural slide creation/deletion are outside this release.
Position handles require CSS Typed OM support (current Chromium); numerical
appearance/text controls work without a separate canvas library.

## Source and styles

Opening/closing does not modify any project file. HTML changes use tokenizer byte
ranges; surrounding tags, comments and unrelated attributes retain their source
bytes. A previously unidentified edited element receives a stable `data-element-id`
on Save. All targets combine slide ID and element ID. Duplicate IDs, ambiguous
shared slide sources, document wrappers, malformed or implicitly closed HTML, and
reserved editor attributes produce actionable errors. Editable fragments must
live under `slides/` and use explicitly balanced tags. IDs use letters, numbers,
hyphens and underscores.

GUI styles live in `styles/editor.css`, registered **after** existing styles in
`deck.json`. Each rule is scoped to both IDs; repeated edits update declarations
instead of accumulating duplicate rules. The purple dot indicates an active GUI
override. Click **↺** beside a property to remove its override and reveal authored
CSS. Other authored styles/scripts remain unchanged. Inline declarations that own
a property disable that control. If a higher-specificity or important authored
rule wins, the editor reports the ineffective override and blocks Save until it
is reset or the source is corrected. It never persists blanket `!important`.
`styles/editor.css` is a reserved, canonical override stylesheet: keep its header,
scoped selectors and supported declarations if editing it manually. Conflicting
non-editor content at this path must be renamed before using GUI styles.

Manifest updates preserve additional metadata, slide titles, script registrations,
and other unrelated fields, although JSON indentation/key order may normalize.
Session selection, zoom and history are not stored in the deck.

## Save, undo and conflicts

There is **no autosave**. Save or ⌘/Ctrl-S with editor controls focused writes the
complete pending command set. Canvas text must first lose focus. Undo/Redo buttons
and ⌘/Ctrl-Z / Shift-⌘/Ctrl-Z operate across text, properties, images, geometry,
notes and slide order. While a text field is focused its native text undo applies.
Saving **does not clear session history**: undoing a saved change makes the deck
unsaved again, and the next Save persists that reversal. Reopening starts a new
history. Close warns about pending changes; a browser crash cannot recover an
unsaved session.

The server hashes the manifest, fragments and all served resources. If source
changes externally, a stale Save is rejected before overwriting any files. The
canvas does not auto-reload during typing or while changes are pending. The
conflict banner lets you **Download pending edits** as a JSON record, then
**Reload & discard pending edits** after explicit confirmation. Automatic merge or
replay onto changed source is not supported; use the downloaded record to reconcile
changes manually. Network/save/export errors leave the current edit history intact.

All commands validate before writing. Each changed file is atomically replaced;
a durable `.decksmith-editor-transaction/` journal allows rollback across files.
A recoverable write error rolls the whole batch back. After a process interruption,
close other editors and run `presmith edit --recover --open` to roll back that
transaction. If a file has also been changed externally since the interrupted
save, recovery stops and retains the journal for manual reconciliation rather than
overwriting it. This is a local explicit-save workflow, not a multi-writer database;
avoid concurrent source changes during the brief commit window.

## Exports

Save first, then choose HTML, PNG, PDF or PPTX in **Export**. Existing CLI validation,
renderer hooks, output ownership, PDF page-count and PPTX structural checks run.
The UI reports output paths and errors. PNG uses the existing `render` workflow.
Files go to the existing `.decksmith/render/` and `dist/` defaults. Speaker notes
are present in HTML/PPTX, not PNG/PDF. Import into PowerPoint, Keynote or Google
Slides must be verified separately; browser rendering is not an Office import test.
