# Presmith command reference

Presmith creates, previews, edits, checks and exports local HTML presentations.
This reference covers every CLI command, its options, requirements, outputs and
failure behavior. For complete workflows, see [Use cases and recipes](use-cases.md).

The executable is `presmith`, formerly `decksmith`. Existing projects still use
`deck.json`, `lib/decksmith.css`, `lib/decksmith.js`, `window.Decksmith` and
`.decksmith/`. Those compatibility names do not need to be renamed.

## Contents

- [Command overview](#command-overview)
- [Invocation and prerequisites](#invocation-and-prerequisites)
- [Help and version](#help-and-version)
- [init](#init)
- [skill install](#skill-install)
- [setup](#setup)
- [doctor](#doctor)
- [dev](#dev)
- [edit](#edit)
- [check](#check)
- [render](#render)
- [export](#export)
- [Paths and output replacement](#paths-and-output-replacement)
- [JSON result schema v1](#json-result-schema-v1)
- [Exit codes and shutdown](#exit-codes-and-shutdown)
- [Troubleshooting](#troubleshooting)

## Command overview

| Command | Use it to | Requires the installed deck renderer? | Main result |
|---|---|---|---|
| `init DIRECTORY` | Start a new presentation | No | Three-slide source project with the Codex skill |
| `skill install --global` | Install the embedded Codex skill for your user | No | `~/.agents/skills/presmith/` |
| `setup [DIRECTORY]` | Install or refresh rendering dependencies | Installs it; needs Node.js 22+, npm and network access | Shared packages and Chromium (or `--local`) |
| `doctor [DIRECTORY]` | Diagnose project and runtime readiness | Probes it; reports missing dependencies | Capabilities and runtime versions |
| `dev [DIRECTORY]` | Preview source changes live | No | Loopback preview server |
| `edit [DIRECTORY]` | Edit supported content and appearance visually | No for editing; yes for its exports | Editor with explicit source saves |
| `check [DIRECTORY]` | Find source, asset, runtime and layout problems | Yes | Findings, optionally JSON |
| `render [DIRECTORY]` | Inspect the deck as images | Yes | Slide PNGs and a contact sheet |
| `export [DIRECTORY] --format FORMAT` | Deliver HTML, PDF or PowerPoint | Yes, including HTML verification | Directory or document |
| `help [COMMAND]` | Read CLI usage | No | Help text |

## Invocation and prerequisites

```sh
presmith COMMAND [DIRECTORY] [OPTIONS]
```

- `init` requires a directory. Every other project command defaults to the current
  directory (`.`). The directory must contain `deck.json`; commands do not search
  parent directories for a project.
- Quote paths with spaces: `presmith check "my talk" --json`.
- Options belong to the selected command. `--json` is supported only by `doctor`,
  `check`, `render`, `export` and `skill install`; it is not a global option.
- `--slide ID` is supported only by `check` and `render`. The value is a stable
  slide ID from `deck.json`, such as `intro`, not a slide number or filename.
- `--out PATH` is supported only by `render` and `export`. Relative paths resolve
  from the shell's current working directory. Default paths resolve from the deck.
- `skill install --global` does not need a deck or `deck.json`.
- Init, skill installation, preview and visual editing need the compiled CLI; preview/edit also need
  a browser to view their UI. Browser diagnostics and artifact generation need
  Node.js 22+, npm for setup, and the deck's pinned Playwright/Chromium installation.
- `setup` performs explicit downloads. Other commands do not install dependencies.
  Browser commands need permission to launch local processes and bind loopback.

When working from a source checkout, build with `cargo build --locked` and replace
`presmith` in the examples with the absolute path to `target/debug/presmith`.
A compiled binary contains the templates, documentation, complete Codex skill and
production editor assets, so it can initialize decks outside the checkout.

## Help and version

```sh
presmith --help
presmith -h
presmith help
presmith help export
presmith export --help
presmith --version
presmith -V
```

Every project subcommand accepts `-h` and `--help`. Version flags apply to the
root command. Help and version print ordinary text even when `--json` is present.
The `export --help` format list includes `html`, `pdf` and `pptx`.

## init

```sh
presmith init DIRECTORY
```

**Use when:** starting a deck from a brief, script, outline, document or other
source material. Init supplies the project structure; it does not generate a
presentation from that material itself.

`DIRECTORY` is required. Init creates a missing directory or fills an empty one.
It refuses an existing nonempty directory and never installs Node packages.
There are no command-specific flags.

The scaffold includes:

| Path | Purpose |
|---|---|
| `deck.json` | Title, dimensions, ordered slides, notes, styles and scripts |
| `slides/` | Three HTML fragments with IDs `intro`, `workflow` and `next` |
| `styles/` | Ink/Paper themes and authored custom styles |
| `scripts/` | Custom presentation behavior |
| `assets/` | Local images, fonts and other presentation resources |
| `lib/` | Presentation CSS/JS runtime |
| `tooling/renderer/` | Pinned package manifests and browser helpers |
| `AGENTS.md`, `README.md`, `docs/` | Local authoring instructions and reference |
| `.agents/skills/presmith/` | Complete Codex skill, references and picker metadata |

The bundled skill is copied from this executable without downloading anything.
Open the deck in Codex and invoke `$presmith`. Init never installs a global skill.
The skill directory is authored project material and is excluded from served and
exported presentation resources. Keep it in version control if sharing the deck.

The default logical canvas is 1280 × 720. Change content and slide order in source
or use the visual editor for its supported operations.

```sh
presmith init "my talk"
presmith dev "my talk" --open
# Install dependencies when ready to check, render or export:
presmith setup "my talk"
```

## skill install

```sh
presmith skill install --global [--force] [--json]
```

**Use when:** making the embedded Presmith skill available across this user's
projects, including older decks. The current directory does not need a manifest.
This command requires Presmith v0.2.1 or newer.

| Option | Default | Meaning |
|---|---|---|
| `--global` | Required | Install to the user's `.agents/skills/presmith/` |
| `--force` | Off | Back up and replace a skill that differs from the embedded copy |
| `--json` | Off | Emit installation status and absolute artifact/backup paths |

The home directory comes from `HOME` on Unix or `USERPROFILE` on Windows and must
be an existing absolute directory. Installation is offline and requires neither
Codex CLI nor Node/npm/Chromium. All embedded files, including references and
`agents/openai.yaml`, are copied. An identical copy is left untouched. Different
or additional files cause an error unless `--force` is supplied.

With `--force`, the entire previous skill is moved to
`~/.agents/.presmith-skill-backups/install-*/presmith/` before the prepared new copy
is installed. Backups are outside skill discovery. A replacement failure attempts
to restore the previous copy; if that fails, the error reports the backup location.
Symlink destinations/parents and non-directory destinations are refused. A running
installer holds a filesystem lock; a competing installer reports an error to retry.

JSON uses command `skill install`, scope `global`, status `installed`, `updated`
or `unchanged`, and `cli_version`. Artifacts include kind `skill` with `path` and
`files`, plus `skill_backup` with `path` when replacing an existing copy.

Codex discovers the user skill folder; restart it if the picker is stale. It may
list both global and project-local copies. This command does not register a plugin,
change Codex settings, or update existing deck copies. After upgrading Presmith,
rerun it to install the embedded skill from that binary. To uninstall this copy,
remove `~/.agents/skills/presmith/` after preserving any custom edits.

## setup

```sh
presmith setup [DIRECTORY] [--upgrade-renderer] [--local]
```

**Use when:** preparing a new deck for browser checks/exports, repairing missing
local dependencies, or upgrading an older renderer.

| Option | Default | Meaning |
|---|---|---|
| `--upgrade-renderer` | Off | Back up and replace bundled renderer files before installing dependencies |
| `--local` | Off | Install packages and Chromium inside this project instead of sharing the user cache |

Setup validates the manifest, checks for Node.js 22+, then reuses a complete shared
installation when the package manifests, platform, Node major version and ABI match.
A cache miss runs `npm ci --no-audit --no-fund --ignore-scripts` and installs pinned
Chromium. Setup always probes an actual browser launch. Downloads occur only during
explicit setup; a warm shared installation needs no npm invocation or downloads.
Linux may also require Playwright system libraries; setup does not install OS
packages or invoke `sudo`.

Without `--upgrade-renderer`, setup retains the deck's renderer source and pinned
package manifests. With the flag, it first backs up replaced files under
`.decksmith/renderer-backups/upgrade-*/`, then refreshes the bundled renderer from
this CLI and installs its dependencies. Authored slides, styles, scripts and
extra custom renderer files remain intact. Updating the CLI alone does not update
an existing deck's renderer, runtime library or copied documentation.

```sh
presmith setup my-talk
# After updating the CLI, refresh an older deck's renderer:
presmith setup my-talk --upgrade-renderer
presmith doctor my-talk --json
```

An upgrade refreshes renderer files before installing dependencies. If a later
installation step fails, retain the backup, fix the reported problem and rerun
setup; the upgrade is not an automatic rollback of the whole installation.

### Renderer cache

Default cache locations:

| Platform | Cache root |
|---|---|
| macOS | `~/Library/Caches/presmith` |
| Linux | `$XDG_CACHE_HOME/presmith`, or `~/.cache/presmith` |
| Windows | `%LOCALAPPDATA%/presmith/Cache` |

Set `PRESMITH_CACHE_DIR` to an absolute path to override the root. It must be outside
the deck. Entries live under `renderer-v1/<hash>/`; the hash includes both complete
package manifests, platform, Node architecture, major version and ABI. Identical
manifests share an entry. Different pins remain isolated. npm's download cache lives
under `npm/`. Renderer JavaScript and package manifests stay authored in each deck;
only `tooling/renderer/node_modules` and `.browsers` become directory symlinks.
Treat the linked installed files as generated dependencies; editing them affects
other decks that share the entry. Edit renderer sources and dependency manifests
in the deck, then rerun setup instead.

**Migrate an existing deck:** run `presmith setup PATH` with a CLI containing this
feature. Setup reuses its downloaded browsers to seed a cold cache. After successful
installation and a launch probe, it replaces the old local dependency directories
with shared links and removes the redundant copies. Authored sources and custom
renderer scripts stay intact. The old small project `.npm-cache` is retained; it
can be removed separately when no npm process is using it. Unconverted decks keep
working with their local installations. Use the updated CLI for future setup
commands: older setup implementations treat installed files as project-local.
This is not a global plugin or npm install.

`presmith setup PATH --local` installs real directories in that deck, detaching any
shared links without deleting their shared targets. It always runs npm installation.
Windows shared setup requires permission to create symlinks (Developer Mode); use
`--local` if unavailable. Moving a deck to another machine requires setup there;
shared links are not portable. Rerun setup after changing Node versions.

Concurrent setup runs serialize cache publication and project dependency swaps.
Cache entries publish only after installation completes. A failed install leaves
existing project dependencies intact; a failed launch probe restores the previous
links/directories. Run setup while the deck is idle. A forced kill or power loss
can leave `.presmith-setup-backup-*` under the deck's renderer; setup reports that
path and requires restoring its saved directories before continuing. Temporary
`.install-*` cache directories may also remain after a forced kill; remove those
only when no setup is running. This is not a filesystem-wide atomic transaction.

**Cleanup:** entries are not automatically evicted, since other decks may still
reference them. With render/export/setup processes stopped, removing a cache entry
reclaims space but breaks links from every deck using it until setup recreates the
entry. Removing the cache root also removes the npm download cache. A missing entry
is recreated by setup; an incomplete/corrupt existing entry produces an actionable
error asking you to remove that entry first. No source files are stored in the cache.

## doctor

```sh
presmith doctor [DIRECTORY] [--json]
```

**Use when:** confirming environment readiness or investigating a failed browser
command. Doctor validates the manifest, assembles the deck and attempts a real
Chromium launch. It does not install anything or perform per-slide layout checks.

| Option | Default | Meaning |
|---|---|---|
| `--json` | Off | Emit one structured result on stdout |

The result adds these boolean `capabilities`:

| Capability | What a true value indicates |
|---|---|
| `preview` | The project can be assembled for preview |
| `check`, `render`, `html_export`, `pdf_export` | Assembly and the browser probe succeeded |
| `pptx_export` | The browser probe also loaded the PPTX helper |

A successful probe includes `runtime.node` and `runtime.chromium`. These capability
checks do not prove that every slide will pass diagnostics or export successfully.
Preview may be available even when missing Chromium makes doctor exit with code 2.
An older renderer can pass doctor with `pptx_export: false`; check that field when
PowerPoint support is required.

```sh
presmith doctor my-talk
presmith doctor my-talk --json
```

## dev

```sh
presmith dev [DIRECTORY] [--port PORT] [--open]
```

**Use when:** authoring HTML/CSS/JS, rehearsing a presentation or testing live
interaction while source changes are made by you or a coding agent.

| Option | Default | Meaning |
|---|---|---|
| `--port PORT` | `4173` | Loopback port; `0` chooses a free port |
| `--open` | Off | Open the preview URL in the default browser |

The server binds to `127.0.0.1`, prints its URL, and stays running until stopped
with Ctrl-C. It requires no renderer installation. Run other CLI commands in a
second terminal while preview is active.

Preview rebuilds relevant manifest, fragment and public-resource changes about
every 500 ms. Tooling, `node_modules` and generated outputs are excluded. A valid
rebuild reloads while preserving the current `#slide-id`. A build error keeps the
last working deck visible with an error overlay; fixing the source restores it.

```sh
presmith dev my-talk --open
# Or let the OS select an available port:
presmith dev my-talk --port 0 --open
```

The server is local preview, not a deployment command. To share a portable deck,
use [HTML export](#html-export). See [Runtime](runtime.md) for presentation keyboard
shortcuts and interaction hooks.

## edit

```sh
presmith edit [DIRECTORY] [--port PORT] [--open] [--recover]
```

**Use when:** adjusting text, typography, colors, spacing, supported geometry,
images, speaker notes or slide order in a graphical editor.

| Option | Default | Meaning |
|---|---|---|
| `--port PORT` | `4174` | Loopback port; `0` chooses a free port |
| `--open` | Off | Open the editor in the default browser |
| `--recover` | Off | Roll back an interrupted save before opening the editor |

The printed URL includes a session credential; open that complete URL and keep it
private. Opening a deck makes no source changes. There is no autosave: Save writes
supported HTML edits, manifest changes and scoped overrides in `styles/editor.css`.
Image uploads are stored under `assets/`. Undo/redo history survives Save within
the session but is not persisted across reopening.

No Node installation or separate frontend server is needed for editing. The
editor's export operations still need the deck renderer. Save before
exporting so the artifacts contain the latest changes.

```sh
presmith edit my-talk --open
# After an interrupted save, close other editors first:
presmith edit my-talk --recover --open
```

Recovery concerns the durable save journal, not unsaved browser-session edits.
External changes cause a stale Save to be rejected. Download pending edits before
reloading/discarding them, then reconcile manually. Recovery also stops if it
would overwrite conflicting external changes.

Structural slide creation/deletion, arbitrary HTML changes and complex widgets
remain source-editing tasks. See [Visual editor](editor.md) for exact supported
operations, text markup, image limits, style rules and conflict handling.

## check

```sh
presmith check [DIRECTORY] [--slide ID] [--json]
```

**Use when:** finding defects before visual review or export, or validating a
specific slide after a small revision.

| Option | Default | Meaning |
|---|---|---|
| `--slide ID` | All slides | Inspect the selected stable slide ID |
| `--json` | Off | Emit findings in the versioned result format |

Check validates project sources, then activates each selected slide at logical
size in Chromium. It measures the same export state used for captures. Checks
include duplicate IDs, failed assets, broken images, JavaScript/readiness errors,
out-of-bounds elements, simple text overflow/clipping and small-text warnings.
External requests are blocked; use local presentation resources.

Filtering does not bypass whole-project manifest validation or shared styles and
scripts. A shared failure may still appear during a one-slide check. Unknown
slide IDs produce a `manifest.unknown_slide` finding.

```sh
presmith check my-talk --json
presmith check my-talk --slide workflow --json
```

Check writes no artifact files. Warning-only results exit 0; detected deck errors
exit 1. It does not evaluate factual accuracy, narrative, contrast, overlap or
complete accessibility. Read [Diagnostics](diagnostics.md), then inspect actual
rendered images.

## render

```sh
presmith render [DIRECTORY] [--slide ID] [--out PATH] [--json]
```

**Use when:** reviewing layout, comparing revisions or producing slide images.

| Option | Default | Meaning |
|---|---|---|
| `--slide ID` | All slides | Capture only the selected stable slide ID |
| `--out PATH` | `<deck>/.decksmith/render` | Destination directory |
| `--json` | Off | Emit absolute artifact paths and metadata |

Render produces one PNG per selected slide and a labeled contact sheet. Slide
images use one CSS pixel per logical pixel, so the default canvas produces
1280 × 720 PNGs. Filenames follow slide IDs rather than slide order:

```text
.decksmith/render/
  .decksmith-output
  contact-sheet.png
  slides/
    intro.png
    workflow.png
    next.png
```

A filtered render produces only the selected PNG and its contact sheet. Reusing
the default output directory replaces a previous full render, so choose a separate
output directory if you want to retain the whole-deck images.

```sh
presmith render my-talk --json
presmith render my-talk --slide workflow --out my-talk/.decksmith/revision --json
```

Rendering rejects asset, JavaScript and readiness errors. It does not enforce the
full layout checks performed by `check`. Run both and open the images: successful
capture alone is not visual approval.

## export

```sh
presmith export [DIRECTORY] --format FORMAT [--out PATH] [--json]
```

**Use when:** preparing a portable web deck, a static document or an editable
PowerPoint handoff.

| Option | Default | Meaning |
|---|---|---|
| `--format FORMAT` | Required | One of `html`, `pdf`, `pptx` |
| `--out PATH` | Depends on format | HTML directory or PDF/PPTX file |
| `--json` | Off | Emit findings and artifact metadata |

Export processes the entire deck in manifest order. It has no `--slide` option.
All formats use browser readiness/export hooks, block external requests and reject
asset/runtime failures. Run `check` separately to catch layout errors.

| Format | Default output | Interactivity/editability | Speaker notes |
|---|---|---|---|
| `html` | `<deck>/dist/html/` | HTML/CSS/JS presentation with live navigation and authored behavior | Embedded in runtime manifest |
| `pdf` | `<deck>/dist/deck.pdf` | Static pages; browser text stays selectable where possible | Not included |
| `pptx` | `<deck>/dist/deck.pptx` | Editable text, basic shapes and pictures; complex visuals may rasterize | Included |

### HTML export

```sh
presmith export my-talk --format html --json
python3 -m http.server 8080 --directory my-talk/dist/html
```

Open `http://localhost:8080` in a browser. Verification exercises keyboard/hash
navigation with external network requests blocked. Output includes presentation
resources without development tooling. View it through an ordinary static server;
`file://` is unsupported. Serving the result requires neither Rust nor Node.
Presmith produces the directory; uploading or hosting it is a separate step.

### PDF export

```sh
presmith export my-talk --format pdf --json
presmith export my-talk --format pdf --out my-talk/dist/handout.pdf
```

PDF uses the logical slide dimensions, print backgrounds, zero margins and no
browser headers/footers. It verifies exactly one page per slide. A page-count
mismatch is an operational error, commonly caused by custom print CSS. Interactive
content needs a meaningful static state defined by its export hook.

### PPTX export

```sh
presmith export my-talk --format pptx --json
```

Open the file in PowerPoint, or upload it to Google Drive and open it with Google
Slides. Text becomes editable line-sized boxes; supported fills, borders and simple
SVG geometry remain shapes. Images remain replaceable pictures. Dimensions,
slide order and speaker notes are preserved. ZIP integrity and slide count are
verified, and the artifact contains per-slide `editability` counts.

Complex SVG, canvas, controls, transforms and unsupported CSS effects use bitmap
fallbacks with `pptx.rasterized` warnings. `data-pptx="raster"` requests this
explicitly. A `pptx.portability` warning reminds you to inspect the imported result.
Tables are individual text/border objects; charts do not become Office data-backed
charts. Font substitution and text-metric changes can affect layout.
PowerPoint/Google Slides changes do not synchronize back to the HTML project.

If PPTX is unavailable on an older deck, run `setup --upgrade-renderer` and check
`doctor --json` for `pptx_export: true`.

## Paths and output replacement

For a command launched from the directory containing `my-talk`:

| Invocation | Output location |
|---|---|
| `presmith render my-talk` | `my-talk/.decksmith/render/` |
| `presmith render my-talk --out review-images` | `review-images/` beside `my-talk` |
| `presmith export my-talk --format pdf` | `my-talk/dist/deck.pdf` |
| `presmith export my-talk --format pdf --out my-talk/dist/review.pdf` | `my-talk/dist/review.pdf` |

`--out` may also be absolute. Paths containing `..` are rejected. Inside a deck,
outputs must be under `dist/` or `.decksmith/`; source directories, the project root
and its ancestors are protected. Existing ancestors/symlinks are resolved before
containment checks. Suitable external output locations are allowed.

Render and HTML outputs are managed directories, identified by
`.decksmith-output` containing `decksmith-output-v1`. A successful rerun replaces
all contents of that directory, including files added manually. Nonempty unmanaged
directories are refused; new or empty directories are accepted. PDF/PPTX outputs
are single files and may replace existing files at an allowed destination without
a directory ownership marker. Keep authored material out of output locations.

Artifacts are generated in temporary storage before installation. Failed browser
checks do not install partial artifacts or replace the previous output. Copy
failures during installation can still leave partial outputs; publishing across
filesystems is not atomic. An existing output after a failed run may be stale.

## JSON result schema v1

For `doctor`, `check`, `render`, `export` and `skill install`, use `--json` to obtain exactly one
JSON object on stdout. Progress and human diagnostics go to stderr. Operational
and CLI argument errors also have structured results when `--json` is requested;
help/version remain ordinary text.

A successful check has this envelope:

```json
{
  "schema_version": 1,
  "command": "check",
  "success": true,
  "findings": [],
  "artifacts": [],
  "error": null
}
```

| Field | Meaning |
|---|---|
| `schema_version` | Result schema version, currently `1` |
| `command` | Command name; argument-parsing failures use `arguments` |
| `success` | Whether this operation succeeded; warnings can coexist with `true` |
| `findings` | Diagnostic objects; an empty array means no reported findings |
| `artifacts` | Produced files/directories with absolute paths |
| `error` | `null` on success, otherwise an object with `kind` and `message` |
| `capabilities` | Doctor's capability booleans |
| `runtime` | Node/Chromium versions when doctor's probe succeeds |

Findings contain `severity`, `rule_id`, `slide_id`, `element_id`, `source`,
`message` and `measurements`. Attribution can be `null`; a global script error
may have no slide ID. See [Diagnostics](diagnostics.md) for rule IDs and repairs.

Artifact fields depend on their `kind`:

| Kind | Additional fields |
|---|---|
| `slide_png` | `slide_id`, logical `width` and `height` |
| `contact_sheet` | Path to the overview PNG |
| `html` | `slides`, `external_network: "blocked"`, `navigation_verified: true` |
| `pdf` | `pages` |
| `pptx` | `slides`, `editable: true`, per-slide `editability` counts |

PPTX counts include `text_boxes`, `shapes`, `images` and `rasterized_elements`
(a subset of images). Error kinds are `deck`, `operational` and `usage`.
Consumers should ignore unknown fields added within schema version 1 and inspect
both the exit code and `success`. The [automation recipe](use-cases.md#automate-checks-and-exports)
shows how to save JSON without losing the CLI exit status.

## Exit codes and shutdown

| Code | Meaning | Next step |
|---|---|---|
| `0` | Success, including warning-only results | Inspect warnings and artifacts |
| `1` | Detected deck problems | Repair the reported sources and rerun |
| `2` | Usage or operational failure | Check arguments, dependencies, permissions, ports and paths |
| `130` | Interrupted | Rerun when ready; use editor recovery if a save was interrupted |

Ctrl-C stops long-running preview/editor servers. On SIGINT/SIGTERM, Rust closes
its in-process server and terminates/waits for its child group; the Node helper
closes Chromium. Parent-pipe closure also triggers helper cleanup. Forced OS
termination or machine failure cannot guarantee temporary-file cleanup. Process
cleanup is tested on macOS; Windows behavior is unverified.

## Troubleshooting

| Symptom | Likely cause and action |
|---|---|
| Init refuses a directory | Use a new or empty directory; init has no overwrite flag |
| `deck.json` is missing | Run inside the deck root or pass its directory explicitly |
| Doctor cannot find/launch Chromium | Run `setup`; inspect Node/npm, download errors and required OS libraries |
| Preview works but render fails | Preview needs only the CLI; install and diagnose the deck renderer |
| `pptx_export` is false | Update the CLI if needed, then run `setup --upgrade-renderer` |
| `manifest.unknown_slide` | Use a slide's `id` from `deck.json`, not its filename or position |
| A layout problem survives export | Run `check`; render/export do not enforce all layout diagnostics |
| An external asset fails | Store and reference the resource locally; browser checks block external requests |
| Output goes to an unexpected folder | Explicit relative `--out` paths use the shell's working directory |
| An output directory is refused | Choose a new/empty destination or a previous Presmith-managed output |
| A partial render removed other PNGs | Use a distinct `--out` directory for filtered renders |
| Preview/editor port is occupied | Use another port or `--port 0` |
| Editor reports an interrupted save | Close other editors, then run `edit --recover` |
| Editor Save conflicts with source changes | Download pending edits, reload/discard explicitly and reconcile manually |
| PDF has an unexpected page count | Review custom print CSS and slide dimensions |
| PPTX looks different after import | Review fonts and rasterization warnings in the destination application |
