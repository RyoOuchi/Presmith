//! Explicit setup only: shared immutable dependency installations, linked into decks.
use crate::process::{self, INTERRUPTED};
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

const DEPENDENCIES: [&str; 2] = ["node_modules", ".browsers"];

fn cache_root() -> Result<PathBuf> {
    let root = if let Some(path) = env::var_os("PRESMITH_CACHE_DIR") {
        PathBuf::from(path)
    } else if cfg!(target_os = "macos") {
        PathBuf::from(
            env::var_os("HOME").context("Set PRESMITH_CACHE_DIR to an absolute directory")?,
        )
        .join("Library/Caches/presmith")
    } else if cfg!(windows) {
        PathBuf::from(
            env::var_os("LOCALAPPDATA")
                .context("Set PRESMITH_CACHE_DIR to an absolute directory")?,
        )
        .join("presmith/Cache")
    } else if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(path).join("presmith")
    } else {
        PathBuf::from(
            env::var_os("HOME").context("Set PRESMITH_CACHE_DIR to an absolute directory")?,
        )
        .join(".cache/presmith")
    };
    if !root.is_absolute() {
        bail!("PRESMITH_CACHE_DIR (or the OS cache directory) must be an absolute path");
    }
    Ok(root)
}

fn key(dir: &Path, node: &str) -> Result<String> {
    let mut hash = Sha256::new();
    // Version the storage protocol, platform and Node ABI as well as dependency pins.
    for data in [
        b"presmith-renderer-cache-v1".as_slice(), env::consts::OS.as_bytes(),
        env::consts::ARCH.as_bytes(), node.as_bytes(),
        &fs::read(dir.join("package.json")).context("Missing tooling/renderer/package.json")?,
        &fs::read(dir.join("package-lock.json")).context("Missing tooling/renderer/package-lock.json; restore renderer files from a fresh presmith init project")?,
    ] {
        hash.update((data.len() as u64).to_le_bytes());
        hash.update(data);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn lock(path: &Path) -> Result<fs::File> {
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    let started = Instant::now();
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(fs::TryLockError::WouldBlock) => {
                if INTERRUPTED.load(Ordering::Relaxed)
                    || started.elapsed() > Duration::from_secs(600)
                {
                    bail!(
                        "Setup interrupted or timed out waiting for another setup to finish: {}",
                        path.display()
                    );
                }
                if started.elapsed() < Duration::from_millis(100) {
                    eprintln!("Waiting for another setup to finish…");
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(fs::TryLockError::Error(e)) => return Err(e.into()),
        }
    }
}

fn link_dir(source: &Path, target: &Path) -> Result<()> {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target)
        .context("Shared setup needs permission to create directory symlinks. Enable Windows Developer Mode or use presmith setup --local")?;
    Ok(())
}

// Preserve macOS framework symlinks without following them recursively.
fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let dest = target.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            let link = fs::read_link(entry.path())?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(link, dest)?;
            #[cfg(windows)]
            if entry.path().is_dir() {
                std::os::windows::fs::symlink_dir(link, dest)?;
            } else {
                std::os::windows::fs::symlink_file(link, dest)?;
            }
        } else if kind.is_dir() {
            copy_tree(&entry.path(), &dest)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), dest)?;
        } else {
            bail!(
                "Unsupported file in renderer installation: {}",
                entry.path().display()
            );
        }
    }
    Ok(())
}

fn remove_entry(path: &Path) -> Result<()> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    if meta.is_symlink() {
        #[cfg(unix)]
        fs::remove_file(path)?;
        #[cfg(windows)]
        fs::remove_dir(path)?;
    } else if meta.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        bail!(
            "Refusing to remove a non-directory renderer dependency: {}",
            path.display()
        );
    }
    Ok(())
}

fn install(dir: &Path, npm_cache: &Path, seed: &Path) -> Result<()> {
    // Reuse an existing browser download when converting a local installation.
    if seed.join(".browsers").is_dir() {
        eprintln!("Reusing the deck's downloaded browser files…");
        copy_tree(&seed.join(".browsers"), &dir.join(".browsers"))?;
    }
    eprintln!("Installing pinned renderer dependencies…");
    let output = process::run(
        Command::new(if cfg!(windows) { "npm.cmd" } else { "npm" })
            .args(["ci", "--no-audit", "--no-fund", "--ignore-scripts"])
            .env("npm_config_cache", npm_cache)
            .current_dir(dir),
        None,
        Duration::from_secs(600),
    )
    .context("npm ci failed. Check npm and network access, then rerun presmith setup")?;
    eprintln!("{output}Installing pinned Chromium (existing downloads are reused)…");
    let output = process::run(
        Command::new("node").args(["node_modules/playwright/cli.js", "install", "chromium"])
            .env("PLAYWRIGHT_BROWSERS_PATH", dir.join(".browsers")).current_dir(dir),
        None, Duration::from_secs(600),
    ).context("Chromium installation failed. Check network/disk space. On Linux install Playwright system libraries; see README")?;
    eprintln!("{output}");
    Ok(())
}

fn manifests(source: &Path, target: &Path) -> Result<()> {
    for name in ["package.json", "package-lock.json"] {
        fs::copy(source.join(name), target.join(name))?;
    }
    Ok(())
}

fn healthy(dir: &Path) -> bool {
    if !dir.join(".complete").is_file() {
        return false;
    }
    // No downloads. Catch deleted packages/browser executables before reusing a cache.
    process::run(
        Command::new("node")
            .args([
                "-e",
                r#"
        const fs = require('node:fs');
        const pkg = require('./package.json');
        for (const name of Object.keys(pkg.dependencies || {})) require.resolve(name);
        const {chromium} = require('playwright');
        if (!fs.existsSync(chromium.executablePath())) process.exit(1);
    "#,
            ])
            .env("PLAYWRIGHT_BROWSERS_PATH", dir.join(".browsers"))
            .current_dir(dir),
        None,
        Duration::from_secs(15),
    )
    .is_ok()
}

/// Swap only generated dependency directories. Restore both if validation fails.
/// Backups survive an uncatchable process/OS termination for manual recovery.
fn attach(dir: &Path, prepared: &Path, probe: impl FnOnce() -> Result<()>) -> Result<()> {
    for name in DEPENDENCIES {
        if let Ok(meta) = fs::symlink_metadata(dir.join(name))
            && !meta.is_dir()
            && !meta.is_symlink()
        {
            bail!("Expected a directory at {}", dir.join(name).display());
        }
    }
    let backup = tempfile::Builder::new()
        .prefix(".presmith-setup-backup-")
        .tempdir_in(dir)?
        .keep();
    let mut installed = Vec::new();
    let result = (|| {
        for name in DEPENDENCIES {
            let target = dir.join(name);
            if fs::symlink_metadata(&target).is_ok() {
                fs::rename(&target, backup.join(name))?;
            }
            fs::rename(prepared.join(name), &target)?;
            installed.push(name);
        }
        probe()
    })();
    if let Err(error) = result {
        let rollback = (|| -> Result<()> {
            for name in DEPENDENCIES.into_iter().rev() {
                if installed.contains(&name) {
                    remove_entry(&dir.join(name))?;
                }
                if fs::symlink_metadata(backup.join(name)).is_ok() {
                    fs::rename(backup.join(name), dir.join(name))?;
                }
            }
            fs::remove_dir(&backup)?;
            Ok(())
        })();
        if let Err(rollback) = rollback {
            bail!(
                "{error:#}. Restore renderer dependencies from {}: {rollback:#}",
                backup.display()
            );
        }
        return Err(error);
    }
    // Old shared links are unlinked; their shared targets are never deleted.
    for name in DEPENDENCIES {
        remove_entry(&backup.join(name))?;
    }
    fs::remove_dir(backup)?;
    Ok(())
}

fn shared_install(
    dir: &Path,
    cache: &Path,
    key: &str,
    healthy: impl Fn(&Path) -> bool,
    install: impl FnOnce(&Path) -> Result<()>,
    probe: impl FnOnce() -> Result<()>,
) -> Result<PathBuf> {
    fs::create_dir_all(cache.join("renderer-v1"))?;
    let cache = cache.canonicalize()?;
    let _cache_lock = lock(&cache.join(".setup.lock"))?;
    let target = cache.join("renderer-v1").join(key);
    if !healthy(&target) {
        if fs::symlink_metadata(&target).is_ok() {
            // Do not replace a published cache in place: readers may still be using it.
            bail!(
                "Renderer cache is incomplete: {}. Close render/export commands, remove this cache entry and rerun presmith setup; project sources are unaffected",
                target.display()
            );
        }
        let staging = tempfile::Builder::new()
            .prefix(".install-")
            .tempdir_in(cache.join("renderer-v1"))?;
        manifests(dir, staging.path())?;
        install(staging.path())?;
        fs::write(
            staging.path().join(".complete"),
            "presmith-renderer-cache-v1\n",
        )?;
        fs::rename(staging.path(), &target)?;
        eprintln!("Installed shared renderer: {}", target.display());
    } else {
        eprintln!(
            "Reusing shared renderer: {} (no downloads)",
            target.display()
        );
    }
    let prepared = tempfile::Builder::new()
        .prefix(".presmith-setup-links-")
        .tempdir_in(dir)?;
    for name in DEPENDENCIES {
        link_dir(&target.join(name), &prepared.path().join(name))?;
    }
    attach(dir, prepared.path(), probe)?;
    Ok(target)
}

pub fn setup(root: &Path, local: bool) -> Result<serde_json::Value> {
    let node = process::run(Command::new("node").args(["-e", "if(Number(process.versions.node.split('.')[0])<22)process.exit(1);console.log(process.platform+'-'+process.arch+'-'+process.versions.node.split('.')[0]+'-'+process.versions.modules)" ]), None, Duration::from_secs(15))
        .context("Node.js 22+ is required. Install Node.js with npm, then rerun presmith setup")?;
    let dir = root.join("tooling/renderer");
    let _project_lock = lock(&dir.join(".presmith-setup.lock"))?;
    let cache_key = key(&dir, node.trim())?;
    // A hard termination leaves the original directories intact here. Never discard them silently.
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(".presmith-setup-backup-")
        {
            bail!(
                "An interrupted setup left backups at {}. Restore any node_modules and .browsers directories from that backup (replacing the current entries), remove the empty backup directory, then rerun setup",
                entry.path().display()
            );
        }
    }
    let mut probed = serde_json::Value::Null;
    let probe = || -> Result<()> {
        if key(&dir, node.trim())? != cache_key {
            bail!(
                "Renderer dependency manifests changed during setup. Rerun setup with the updated files"
            );
        }
        let result = process::helper(root, &serde_json::json!({"action":"probe"}))?;
        if result["success"] != true {
            bail!("Chromium could not launch: {}", result["error"]);
        }
        probed = result;
        Ok(())
    };
    if local {
        let staging = tempfile::Builder::new()
            .prefix(".presmith-setup-local-")
            .tempdir_in(&dir)?;
        manifests(&dir, staging.path())?;
        install(staging.path(), &dir.join(".npm-cache"), &dir)?;
        attach(&dir, staging.path(), probe)?;
        eprintln!(
            "Using project-local renderer dependencies: {}",
            dir.display()
        );
    } else {
        let cache = cache_root()?;
        fs::create_dir_all(&cache)?;
        if cache.canonicalize()?.starts_with(root) {
            bail!(
                "PRESMITH_CACHE_DIR must be outside the presentation project; use --local for a self-contained installation"
            );
        }
        shared_install(
            &dir,
            &cache,
            &cache_key,
            healthy,
            |staging| {
                install(staging, &cache.join("npm"), &dir)?;
                if key(staging, node.trim())? != cache_key {
                    bail!(
                        "Renderer dependency manifests changed during setup; no cache entry was published. Rerun setup"
                    );
                }
                Ok(())
            },
            probe,
        )?;
    }
    Ok(probed)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier, atomic::AtomicUsize};

    fn deck(parent: &Path, name: &str) -> PathBuf {
        let dir = parent.join(name);
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("package.json"), "{\"dependencies\":{}}").unwrap();
        fs::write(dir.join("package-lock.json"), "{}").unwrap();
        fs::write(dir.join("render.mjs"), "custom renderer stays here").unwrap();
        dir
    }
    fn fake_install(dir: &Path) -> Result<()> {
        for name in DEPENDENCIES {
            fs::create_dir(dir.join(name))?;
            fs::write(dir.join(name).join("installed"), "new")?;
        }
        Ok(())
    }
    fn complete(dir: &Path) -> bool {
        dir.join(".complete").is_file()
    }

    #[test]
    fn identity_uses_both_manifests_and_node_but_not_project_path_or_renderer_source() {
        let temp = tempfile::tempdir().unwrap();
        let a = deck(temp.path(), "a");
        let b = deck(temp.path(), "b");
        let first = key(&a, "22-127").unwrap();
        assert_eq!(first, key(&b, "22-127").unwrap());
        fs::write(a.join("render.mjs"), "custom updated").unwrap();
        assert_eq!(first, key(&a, "22-127").unwrap());
        assert_ne!(first, key(&a, "24-137").unwrap());
        fs::write(
            a.join("package.json"),
            "{\"dependencies\":{},\"version\":\"2\"}",
        )
        .unwrap();
        assert_ne!(first, key(&a, "22-127").unwrap());
        fs::write(b.join("package-lock.json"), "{\"lockfileVersion\":3}").unwrap();
        assert_ne!(first, key(&b, "22-127").unwrap());
    }

    #[test]
    fn two_decks_and_repeated_setup_share_one_install_and_preserve_sources() {
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        let a = deck(temp.path(), "a");
        let b = deck(temp.path(), "b with spaces");
        fake_install(&a).unwrap(); // Migrate a physical project-local installation.
        let target = shared_install(&a, &cache, "v1", complete, fake_install, || Ok(())).unwrap();
        for dir in [&a, &b] {
            shared_install(
                dir,
                &cache,
                "v1",
                complete,
                |_| panic!("warm cache must not install"),
                || Ok(()),
            )
            .unwrap();
            for name in DEPENDENCIES {
                assert!(dir.join(name).is_symlink());
                assert_eq!(dir.join(name).canonicalize().unwrap(), target.join(name));
            }
            assert_eq!(
                fs::read_to_string(dir.join("render.mjs")).unwrap(),
                "custom renderer stays here"
            );
            assert!(!fs::read_dir(dir).unwrap().any(|e| {
                e.unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".presmith-setup-backup-")
            }));
        }
        let v2 = shared_install(&b, &cache, "v2", complete, fake_install, || Ok(())).unwrap();
        assert_ne!(v2, target);
        assert_eq!(
            a.join("node_modules").canonicalize().unwrap(),
            target.join("node_modules")
        );
        assert!(target.join("node_modules/installed").is_file());
    }

    #[test]
    fn failed_install_is_not_published_and_does_not_touch_local_install() {
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        let dir = deck(temp.path(), "deck");
        fake_install(&dir).unwrap();
        let result = shared_install(
            &dir,
            &cache,
            "v1",
            complete,
            |p| {
                fs::create_dir(p.join("node_modules"))?;
                bail!("simulated npm failure");
            },
            || panic!("must not attach"),
        );
        assert!(result.is_err());
        assert!(!cache.join("renderer-v1/v1").exists());
        assert_eq!(fs::read_dir(cache.join("renderer-v1")).unwrap().count(), 0);
        assert!(!dir.join("node_modules").is_symlink());
        assert!(dir.join("node_modules/installed").is_file());
    }

    #[test]
    fn failed_probe_restores_both_local_directories_and_existing_shared_links() {
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        let dir = deck(temp.path(), "deck");
        fake_install(&dir).unwrap();
        assert!(
            shared_install(&dir, &cache, "v1", complete, fake_install, || {
                assert!(dir.join("node_modules").is_symlink());
                bail!("simulated browser failure");
            })
            .is_err()
        );
        for name in DEPENDENCIES {
            assert!(!dir.join(name).is_symlink());
            assert!(dir.join(name).join("installed").is_file());
        }
        let v1 =
            shared_install(&dir, &cache, "v1", complete, |_| unreachable!(), || Ok(())).unwrap();
        assert!(
            shared_install(&dir, &cache, "v2", complete, fake_install, || bail!(
                "probe failed"
            ))
            .is_err()
        );
        assert_eq!(
            dir.join("node_modules").canonicalize().unwrap(),
            v1.join("node_modules")
        );
        assert!(v1.join(".browsers/installed").is_file());
    }

    #[test]
    fn partial_attach_failure_rolls_back_and_local_mode_does_not_delete_shared_target() {
        let temp = tempfile::tempdir().unwrap();
        let dir = deck(temp.path(), "deck");
        fake_install(&dir).unwrap();
        let prepared = deck(temp.path(), "prepared");
        fs::create_dir(prepared.join("node_modules")).unwrap();
        assert!(attach(&dir, &prepared, || Ok(())).is_err()); // Missing second directory.
        assert!(dir.join("node_modules/installed").is_file());
        assert!(dir.join(".browsers/installed").is_file());
        let cache = shared_install(
            &dir,
            &temp.path().join("cache"),
            "v1",
            complete,
            fake_install,
            || Ok(()),
        )
        .unwrap();
        fs::create_dir(prepared.join("node_modules")).unwrap();
        fs::create_dir(prepared.join(".browsers")).unwrap();
        attach(&dir, &prepared, || Ok(())).unwrap();
        assert!(!dir.join("node_modules").is_symlink());
        assert!(cache.join("node_modules/installed").is_file());
    }

    #[test]
    fn concurrent_decks_publish_one_complete_cache() {
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        let barrier = Arc::new(Barrier::new(2));
        let installs = Arc::new(AtomicUsize::new(0));
        let mut threads = Vec::new();
        for name in ["a", "b"] {
            let dir = deck(temp.path(), name);
            let cache = cache.clone();
            let barrier = barrier.clone();
            let installs = installs.clone();
            threads.push(thread::spawn(move || {
                barrier.wait();
                shared_install(
                    &dir,
                    &cache,
                    "v1",
                    complete,
                    |p| {
                        installs.fetch_add(1, Ordering::SeqCst);
                        thread::sleep(Duration::from_millis(150));
                        fake_install(p)
                    },
                    || Ok(()),
                )
                .unwrap()
            }));
        }
        let a = threads.remove(0).join().unwrap();
        let b = threads.remove(0).join().unwrap();
        assert_eq!(a, b);
        assert_eq!(installs.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn corrupt_cache_is_reported_without_overwriting_it() {
        let temp = tempfile::tempdir().unwrap();
        let dir = deck(temp.path(), "deck");
        let cache = temp.path().join("cache");
        fs::create_dir_all(cache.join("renderer-v1/v1")).unwrap();
        let result = shared_install(
            &dir,
            &cache,
            "v1",
            complete,
            |_| panic!("must not overwrite"),
            || Ok(()),
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cache is incomplete")
        );
        assert!(!dir.join("node_modules").exists());
    }
}
