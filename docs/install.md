# Install Presmith

The v0.2.0 prerelease provides an `aarch64-apple-darwin` binary for Apple Silicon
Macs. Intel Macs, Linux, and Windows do not have verified binaries in this release.
Use macOS 14 or newer for the full browser workflow, following
[Playwright's system requirements](https://playwright.dev/docs/intro#system-requirements).

The CLI includes its visual editor and project templates. Rust is required only
when building from source. Browser checks, PNG rendering, PDF, and PowerPoint
export require Node.js 22+ with npm. `presmith setup` downloads the pinned renderer
packages and Chromium into each deck's `tooling/renderer/` directory.

## Install with Homebrew

On an Apple Silicon Mac running macOS 14 or newer, with
[Homebrew](https://brew.sh/) installed:

```sh
brew install ryoouchi/tap/presmith
presmith --version
```

This adds the [Presmith tap](https://github.com/RyoOuchi/homebrew-tap), verifies the
release archive's SHA-256 checksum, and installs the CLI plus Node.js 24 with npm.
Presmith automatically uses that tested runtime; your shell's default Node version
does not need to change. Rust is not required. Continue with [Create your first deck](#create-your-first-deck)
to install the project-local renderer and Chromium.

To update after a new version is published:

```sh
brew update
brew upgrade ryoouchi/tap/presmith
```

To remove the CLI, run `brew uninstall presmith`. Your deck projects remain intact.

## Install the binary manually

Download the macOS Apple Silicon archive and `SHA256SUMS` from
[Presmith releases](https://github.com/RyoOuchi/Presmith/releases).

Run these commands in the directory containing the downloaded archive and
`SHA256SUMS`. The checksum file also lists the optional plugin archive; use the
matching line to verify just the CLI archive:

```sh
shasum -a 256 presmith-v0.2.0-aarch64-apple-darwin.tar.gz
# Compare the complete hash with the matching filename in SHA256SUMS.
tar -xzf presmith-v0.2.0-aarch64-apple-darwin.tar.gz
mkdir -p "$HOME/.local/bin"
install -m 755 presmith-v0.2.0-aarch64-apple-darwin/presmith "$HOME/.local/bin/presmith"
export PATH="$HOME/.local/bin:$PATH"
presmith --version
```

Add the `export PATH` line to your shell configuration if that directory is not
already on your PATH. The archive includes the project license, dependency
notices, Rust standard-library notices, and build information.

This preview binary is not Developer ID signed or notarized. macOS may block its
first launch. Review [Apple's instructions for opening downloaded software](https://support.apple.com/en-us/102445)
or use the source installation below.

## Build from source

With stable Rust/Cargo installed:

```sh
cargo install --git https://github.com/RyoOuchi/Presmith.git --tag v0.2.0 --locked
presmith --version
```

Production editor assets are included in the source, so this build does not need
frontend build tools. The executable is installed in Cargo's bin directory,
normally `$HOME/.cargo/bin`; ensure that directory is on PATH.

## Create your first deck

```sh
presmith init my-talk
cd my-talk
presmith setup
presmith doctor
presmith edit --open
```

For live preview use `presmith dev --open`. After saving your slides:

```sh
presmith check --json
presmith render
presmith export --format html
presmith export --format pdf
presmith export --format pptx
```

Rendered images appear in `.decksmith/render/`; exports appear in `dist/`.
Existing Decksmith projects retain their file format and browser runtime API.
For an older deck, `presmith setup --upgrade-renderer` refreshes the renderer and
backs up replaced renderer files without changing authored slides.

## Optional Codex skill

The release also includes `presmith-plugin-v0.2.1.tar.gz`. Extract it, then copy
the `presmith/skills/presmith` folder into your deck's `.agents/skills/` directory
to use `$presmith`. Preserve an existing skill folder when updating it.
The CLI is installed separately. See the repository's
[plugin instructions](https://github.com/RyoOuchi/Presmith/blob/v0.2.0/docs/plugin.md)
for the plugin and local-marketplace installation options.
