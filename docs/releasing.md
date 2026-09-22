# Releasing Presmith

Pushing a version tag such as `v0.2.1` runs the
[Tagged release workflow](https://github.com/RyoOuchi/Presmith/actions/workflows/release.yml).
It builds the native Apple Silicon CLI, verifies it, publishes GitHub release
assets, and updates [the Homebrew tap](https://github.com/RyoOuchi/homebrew-tap).
Pushing to `main` alone does not publish a release.

## Prepare a version

1. Update the package version in `Cargo.toml`, then run `cargo check` to refresh
   `Cargo.lock`. The CLI and optional plugin have independent versions; update
   both plugin manifests only when the plugin changes.
2. Add `docs/releases/v<VERSION>.md` with release notes and update installation
   instructions or platform requirements if they changed.
3. If editor sources changed, rebuild and commit `editor/build/`:

   ```sh
   npm ci --ignore-scripts --prefix editor
   npm run build --prefix editor
   ```

4. Commit the release sources and push `main`. Then create and push the matching
   version tag. For example, after setting the Cargo version to `0.2.1`:

   ```sh
   git tag -a v0.2.1 -m "Presmith v0.2.1"
   git push origin v0.2.1
   ```

The workflow rejects a tag that differs from the Cargo version or lacks release
notes. Versions below 1.0 and versions with a suffix such as `-rc.1` are published
as GitHub prereleases. The Homebrew tap follows these verified releases as it does
for the initial preview. Cargo and npm registry publishing remain disabled.

## What runs automatically

The build job uses GitHub's standard `macos-15` Apple Silicon runner, Node.js 24,
and stable Rust. It checks committed editor assets, Rust formatting and Clippy,
Rust tests, and release automation tests. It packages the CLI and plugin with
licenses, build information, and SHA-256 checksums, then verifies the archives.
The extracted CLI initializes a fresh deck and installs its renderer and Chromium.
Both browser integration suites run against that binary and renderer.

After those checks pass, the publish job:

1. Uploads the three release assets to a draft, downloads them again, and verifies
   their checksums, embedded binary checksum, version, and source commit.
2. Publishes the verified release.
3. Updates `Formula/presmith.rb` from the published archive's URL and checksum,
   removing the old formula revision when the version changes.
4. Runs Homebrew's strict online audit, style check, installation, and formula
   test, plus renderer setup and HTML export through the installed command.
5. Pushes the tested formula to the tap's `main` branch.

The launcher selects Homebrew's `node@24` runtime. Test a fresh `presmith setup`
before changing that dependency: Node 26.8.2 stalled while extracting Chromium
when the tap was first validated.

Installed copies update when users run:

```sh
brew update
brew upgrade ryoouchi/tap/presmith
```

## Validate without publishing

Use **Run workflow** on the Tagged release Actions page, or:

```sh
gh workflow run release.yml --repo RyoOuchi/Presmith --ref main
```

A manual run performs the full build and browser verification and saves artifacts.
It skips release publication and tap writes. Inspect the run's downloadable
`presmith-release` and `browser-verification` artifacts for results.

## Credentials and recovery

The publish job uses GitHub's built-in `GITHUB_TOKEN` to create releases in this
repository. A dedicated write-enabled deploy key on `RyoOuchi/homebrew-tap` supplies
cross-repository Git access. Its private key is stored only as the Presmith Actions
secret `HOMEBREW_TAP_SSH_KEY`; the checkout action configures and removes it on the
runner. To rotate it, replace that secret and the matching tap deploy key.

Rerun failed jobs from the Actions page after resolving a transient failure.
An existing complete release is downloaded and verified against its original
source commit; its archives are reused without replacement. If a draft has
incomplete or conflicting assets, inspect it and finish or delete that draft
before rerunning. A published release's tag and assets must never be replaced.
The updater refuses to downgrade the tap or change an existing version's checksum.
If the tap advanced while a job was running, its normal Git push fails rather
than overwriting those changes; rerun the publish job to use the current tap.

The binary remains unsigned and unnotarized by Developer ID. A new version is
required for changes to the published binary or release assets.

## Manual packaging fallback

After committing and validating the release sources on an Apple Silicon Mac:

```sh
cargo fetch --locked
python3 scripts/package-release.py
```

Archives and `SHA256SUMS` appear under `target/release-artifacts/v<VERSION>/`.
The packager rejects dirty checkouts and includes dependency licenses from the
Cargo cache and `licenses/dependencies/`. Its `--allow-dirty` option is only for
local previews. The normal release path is the tagged workflow above.
