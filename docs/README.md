# Presmith documentation

Start with the [command reference](project/cli.md) for every CLI command and
option, or [use cases and recipes](project/use-cases.md) for complete workflows.
The executable is `presmith`; existing `decksmith` project/runtime filenames
remain compatible.

## Find the right guide

| Guide | What it covers |
|---|---|
| [Install Presmith](install.md) | Homebrew, manual binaries, source installation and first setup |
| [Command reference](project/cli.md) | All commands, flags, defaults, examples, output ownership, JSON, exit codes and troubleshooting |
| [Use cases and recipes](project/use-cases.md) | Creating from different inputs, targeted revisions, visual editing, demos, delivery, upgrades and automation |
| [Authoring](project/authoring.md) | Manifest, fragments, paths, IDs, CSS scoping and layout patterns |
| [Visual editor](project/editor.md) | Supported operations, explicit saves, undo/redo, source conflicts and recovery |
| [Runtime](project/runtime.md) | Browser API, navigation, readiness, interaction and static export hooks |
| [Diagnostics](project/diagnostics.md) | Finding IDs, severity, repairs and checker limitations |
| [PPTX export](pptx-export.md) | Editable objects, raster fallbacks, compatibility and verification |
| [Codex plugin](plugin.md) | Optional skill/plugin installation and integration boundaries |
| [Creation workflows](../plugins/presmith/skills/presmith/references/creation-workflows.md) | Agent guidance for briefs, scripts, outlines, documents, data, existing decks and demos |
| [Verification](verification.md) | Recorded CLI/browser checks and known limits |
| [Editor verification](editor-verification.md) | Editor test evidence and manual inspection boundaries |
| [Evaluation](evaluation.md) | Protocol for comparative evaluation |
| [Releasing](releasing.md) | Versioning, packaging, release automation and Homebrew distribution |
| [v0.2.2 release notes](releases/v0.2.2.md) | Updated skill with visual direction and consistency requirements |
| [v0.2.1 release notes](releases/v0.2.1.md) | Bundled skills, shared renderer cache and upgrade instructions |
| [v0.2.0 release notes](releases/v0.2.0.md) | Initial prerelease features and platform limitations |

The files in `docs/project/` are embedded at compile time and copied to each new
deck's `docs/` by `presmith init`. Relative links among those files work both here
and in an initialized deck. Repository installation, plugin and maintainer docs
stay in this checkout. Updating the CLI does not refresh existing decks' copied
documentation.

## Common starting points

| I want to… | Start here |
|---|---|
| See every supported command | [Command overview](project/cli.md#command-overview) |
| Create and deliver a presentation | [First-deck recipe](project/use-cases.md#create-and-deliver-your-first-deck) |
| Turn a script, outline, report or data into slides | [Source-material workflows](project/use-cases.md#create-from-different-source-material) |
| Fix one slide without replacing the full render | [Targeted revision](project/use-cases.md#revise-one-slide) |
| Use the graphical editor | [Visual editing recipe](project/use-cases.md#edit-visually) |
| Choose between HTML, PDF, PPTX and images | [Delivery formats](project/use-cases.md#choose-a-delivery-format) |
| Parse results or automate a quality gate | [Automation recipe](project/use-cases.md#automate-checks-and-exports) and [JSON schema](project/cli.md#json-result-schema-v1) |
| Diagnose a failed command | [Troubleshooting](project/cli.md#troubleshooting) |
| Upgrade an older deck | [Upgrade recipe](project/use-cases.md#upgrade-an-existing-deck) |

## Repository development commands

These commands run from the repository root. They are contributor tools, separate
from the deck CLI. The normal build embeds the already committed editor assets.

| Command | Use case and effect |
|---|---|
| `cargo build --locked` | Build `target/debug/presmith` from pinned Rust dependencies |
| `cargo build --release --locked` | Build the optimized binary under `target/release/` |
| `cargo install --path . --locked` | Install the CLI from this checkout into Cargo's bin directory |
| `cargo fmt --check` | Check Rust formatting without rewriting files |
| `cargo clippy --all-targets -- -D warnings` | Lint all Rust targets and fail on warnings |
| `cargo test --locked` | Run Rust tests; standalone Node browser suites are separate |
| `python3 scripts/prepare-example.py` | Refresh canonical runtime, renderer and docs copies in `examples/product`; preserve authored slides |
| `./target/debug/presmith setup examples/product` | Install the example's browser runtime for preview verification and integration suites |
| `node tests/browser.mjs [RUNTIME_PROJECT]` | Run CLI/browser integration checks using an installed project renderer; default is `examples/product` |
| `node tests/editor-browser.mjs [RUNTIME_PROJECT]` | Run visual editor integration checks and generate verification evidence; same default runtime project |
| `python3 scripts/verify-workflow.py [DIRECTORY]` | Exercise init/setup/doctor/preview/check/render/HTML/PDF and a targeted revision; explicitly downloads dependencies |
| `python3 tests/test_release.py` | Run release automation unit tests without publishing |

Browser suites use `target/debug/presmith` by default; `PRESMITH_BIN` selects another
binary. Build it first and prepare the runtime project with `setup`. Browser
verification needs local process/loopback permissions. The fresh-workflow script
uses `verification/workflow talk` by default; choose a new or empty directory for
another run. It records results in `verification/fresh-workflow.json`.

### Editor frontend

Run these after changing editor source. They are unnecessary for ordinary deck
authoring or building Rust against the committed bundle.

| Command | Use case and effect |
|---|---|
| `npm ci --ignore-scripts --prefix editor` | Install the pinned frontend development dependencies |
| `npm run typecheck --prefix editor` | Check TypeScript without emitting files |
| `npm run build --prefix editor` | Typecheck and rebuild production assets under `editor/build/` |
| `npm run dev --prefix editor` | Watch/rebuild both frontend entry points; no separate frontend web server |
| `npm run format:check --prefix editor` | Check frontend formatting |
| `npm run format --prefix editor` | Rewrite frontend source formatting with Prettier |

After changing the bundle, rebuild Rust and restart `presmith edit` because the
assets are embedded in the binary. See the repository
[frontend development instructions](../README.md#frontend-development-and-production-build).

### Packaging and release helpers

See [Releasing](releasing.md) for the complete sequence, prerequisites and recovery
behavior. The normal publishing path is its tagged release workflow. These helper
commands expose the underlying operations; publication changes remote state.

| Command | Use case and options |
|---|---|
| `cargo fetch --locked` | Populate the pinned Cargo dependency cache before packaging |
| `python3 scripts/package-release.py [--allow-dirty]` | Build and package release archives with licenses/checksums under `target/release-artifacts/v<VERSION>/`; dirty builds are local previews only |
| `python3 scripts/release.py metadata --output FILE [--tag TAG] [--github-output FILE]` | Validate version/release notes, write metadata, optionally verify a tag and append GitHub Actions outputs |
| `python3 scripts/release.py verify --metadata FILE --artifacts DIRECTORY [--binary-out FILE]` | Verify archive and embedded metadata/checksums; optionally extract the executable |
| `python3 scripts/release.py publish --metadata FILE --artifacts DIRECTORY --verified-dir DIRECTORY` | Create/reuse, download, verify and publish a GitHub release; needs authenticated `gh` and a new verification directory |
| `python3 scripts/release.py update-formula --metadata FILE --artifacts DIRECTORY --formula FILE` | Verify artifacts and update the local Homebrew formula; does not push it |
| `gh workflow run release.yml --repo RyoOuchi/Presmith --ref main` | Dispatch build/browser verification without publishing release assets or updating the tap |

The packaging script and release helper subcommands accept `--help` for their
argument syntax. A tagged workflow publishes only after its checks; a manual
workflow run performs validation without publication. Push/tag instructions and
credential setup are documented in the release guide.
