# Releasing Presmith

The first release is a GitHub prerelease for Apple Silicon macOS. Cargo and npm
registry publishing remain disabled. The CLI and optional plugin have independent
versions, currently 0.2.0 and 0.2.1.

## Validate and package

From a native Apple Silicon Mac with Rust, Node.js 22+, npm, Python 3, and GitHub CLI:

```sh
npm ci --ignore-scripts --prefix editor
npm run build --prefix editor
npm run format:check --prefix editor
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 scripts/prepare-example.py
./target/debug/presmith setup examples/product
node tests/browser.mjs
node tests/editor-browser.mjs
git diff --check
```

Run the browser suites sequentially. Update the installation guide and versioned
release notes, and commit all release sources, including the production editor
bundle. Then create the archives:

```sh
python3 scripts/package-release.py
```

The script builds the CLI, verifies its version, collects dependency license
texts, packages both artifacts, and writes `SHA256SUMS` under
`target/release-artifacts/v0.2.0/`. It rejects a dirty checkout by default;
`--allow-dirty` is for local preparation only. Upstream license files missing from
published crates are preserved under `licenses/dependencies/` with exact source URLs.

Extract the CLI archive outside the checkout and run both browser suites with
`PRESMITH_BIN` set to the extracted executable's absolute path. Verify the checksums
and `BUILD-INFO.json` commit. A full fresh `init → setup → doctor → check → render →
export` smoke test confirms the renderer installs from its bundled lockfile.

## Publish

After all checks pass and the release commit is pushed:

```sh
git tag -a v0.2.0 -m "Presmith v0.2.0"
git push origin v0.2.0
gh release create v0.2.0 \
  target/release-artifacts/v0.2.0/presmith-v0.2.0-aarch64-apple-darwin.tar.gz \
  target/release-artifacts/v0.2.0/presmith-plugin-v0.2.1.tar.gz \
  target/release-artifacts/v0.2.0/SHA256SUMS \
  --repo RyoOuchi/Presmith --verify-tag --prerelease \
  --title "Presmith v0.2.0" --notes-file docs/releases/v0.2.0.md
```

Check the published release and download its assets to verify their hashes. Do not
replace an already published version's tag or artifacts; prepare a new version.
The current package is not Developer ID signed or notarized, and its installation
guide and release notes disclose that limitation.

## Update Homebrew

The formula lives in [RyoOuchi/homebrew-tap](https://github.com/RyoOuchi/homebrew-tap),
at `Formula/presmith.rb`. After verifying the published release assets, update its
URL to the new version and copy the CLI archive's SHA-256 from `SHA256SUMS`.
Keep the architecture and minimum macOS requirements aligned with the tested release.
The launcher selects `node@24` for renderer compatibility. Test a fresh
`presmith setup` before changing the Node dependency; Node 26.8.2 stalled while
extracting Chromium during the initial Homebrew validation.

Edit the formula in Homebrew's tap checkout (`brew --repository ryoouchi/tap`).
Validate it on a supported Apple Silicon Mac before committing and pushing the tap:

```sh
brew audit --strict --online ryoouchi/tap/presmith
brew style ryoouchi/tap/presmith
brew reinstall ryoouchi/tap/presmith
brew test ryoouchi/tap/presmith
```

The formula test creates a deck and verifies its manifest and bundled runtime files
without browser downloads. The release checks above cover browser rendering and exports. Users install with
`brew install ryoouchi/tap/presmith` and update with `brew update` followed by
`brew upgrade ryoouchi/tap/presmith`.
