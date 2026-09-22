//! Offline installation of the same skill that `init` writes into new decks.
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

const PREFIX: &str = ".agents/skills/presmith/";

pub fn install_global(force: bool) -> Result<Value> {
    let home_var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let home = PathBuf::from(env::var_os(home_var).with_context(|| {
        format!("{home_var} is not set; cannot locate the global Codex skills directory")
    })?);
    if !home.is_absolute() {
        bail!("{home_var} must be a nonempty absolute home directory");
    }
    let home = home
        .canonicalize()
        .context("Cannot access the home directory")?;
    if !home.is_dir() {
        bail!("Home path is not a directory: {}", home.display());
    }
    let agents = home.join(".agents");
    ensure_directory(&agents)?;
    let skills = agents.join("skills");
    ensure_directory(&skills)?;

    // An OS lock serializes Presmith installers and releases automatically on exit.
    let lock_path = skills.join(".presmith-skill-install.lock");
    if lock_path.is_symlink() {
        bail!(
            "Refusing a symlink installation lock: {}",
            lock_path.display()
        );
    }
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)?;
    match lock.try_lock() {
        Ok(()) => {}
        Err(fs::TryLockError::WouldBlock) => {
            bail!("Another Presmith skill installation is running; retry after it finishes")
        }
        Err(fs::TryLockError::Error(e)) => return Err(e.into()),
    }

    let files: BTreeMap<PathBuf, &[u8]> = crate::ASSETS
        .iter()
        .filter_map(|(name, data)| {
            name.strip_prefix(PREFIX)
                .map(|name| (PathBuf::from(name), *data))
        })
        .collect();
    if !files.contains_key(Path::new("SKILL.md")) {
        bail!("This executable does not contain the Presmith skill; rebuild or reinstall it");
    }
    let target = skills.join("presmith");
    if target.is_symlink() {
        bail!(
            "Refusing to replace a symlink skill: {}. Manage its target explicitly",
            target.display()
        );
    }
    let exists = target.try_exists()?;
    if exists {
        if !target.is_dir() {
            bail!("Skill destination is not a directory: {}", target.display());
        }
        if matches_bundle(&target, &files)? {
            return Ok(result(&target, "unchanged", files.len(), None));
        }
        if !force {
            bail!(
                "An existing skill differs at {}. Use presmith skill install --global --force to back it up and replace it",
                target.display()
            );
        }
    }

    // Prepare everything before touching the previous installation.
    let stage = tempfile::Builder::new()
        .prefix(".presmith-skill-")
        .tempdir_in(&skills)?;
    for (name, data) in &files {
        let path = stage.path().join(name);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, data)?;
    }
    let backup = if exists {
        // Outside skills/: backups must not be discovered as duplicate skills.
        let backups = agents.join(".presmith-skill-backups");
        ensure_directory(&backups)?;
        let saved = tempfile::Builder::new()
            .prefix("install-")
            .tempdir_in(&backups)?;
        fs::rename(&target, saved.path().join("presmith"))
            .context("Could not back up the existing Presmith skill")?;
        Some(saved.keep().join("presmith"))
    } else {
        None
    };
    if let Err(error) = fs::rename(stage.path(), &target) {
        if let Some(saved) = &backup {
            if let Err(rollback) = fs::rename(saved, &target) {
                bail!(
                    "Skill installation failed: {error}. Restore the previous skill from {} (automatic restore failed: {rollback})",
                    saved.display()
                );
            }
            let _ = fs::remove_dir(saved.parent().unwrap());
        }
        return Err(error).context(
            "Could not install the staged Presmith skill; any previous skill was restored",
        );
    }
    Ok(result(
        &target,
        if exists { "updated" } else { "installed" },
        files.len(),
        backup.as_deref(),
    ))
}

fn ensure_directory(path: &Path) -> Result<()> {
    if path.is_symlink() {
        bail!(
            "Refusing to install through a symlink directory: {}",
            path.display()
        );
    }
    fs::create_dir_all(path).with_context(|| format!("Cannot create directory {}", path.display()))
}

fn matches_bundle(root: &Path, files: &BTreeMap<PathBuf, &[u8]>) -> Result<bool> {
    let mut stack = vec![root.to_path_buf()];
    let mut count = 0;
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                stack.push(entry.path());
            } else if kind.is_file() {
                let path = entry.path();
                let Some(expected) = files.get(path.strip_prefix(root)?) else {
                    return Ok(false);
                };
                if fs::read(&path)? != *expected {
                    return Ok(false);
                }
                count += 1;
            } else {
                // Do not follow custom symlinks or special files.
                return Ok(false);
            }
        }
    }
    Ok(count == files.len())
}

fn result(target: &Path, status: &str, files: usize, backup: Option<&Path>) -> Value {
    let mut value = crate::envelope("skill install");
    value["scope"] = json!("global");
    value["status"] = json!(status);
    value["cli_version"] = json!(env!("CARGO_PKG_VERSION"));
    value["artifacts"] = json!([{"kind":"skill", "path":target, "files":files}]);
    if let Some(backup) = backup {
        value["artifacts"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"skill_backup", "path":backup}));
    }
    value
}
