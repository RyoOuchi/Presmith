//! Same-filesystem atomic replacements plus a durable undo journal for multi-file recovery.
use super::*;
use std::io::Write;
const JOURNAL: &str = ".decksmith-editor-transaction";
pub fn safe_target(root: &Path, name: &str) -> Result<PathBuf> {
    let path = manifest::relative_path(name)?;
    let mut full = root.to_path_buf();
    for component in path.components() {
        full.push(component);
        match fs::symlink_metadata(&full) {
            Ok(m) => ensure!(
                !m.file_type().is_symlink(),
                "Symlink write/serve rejected: {name}"
            ),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(full)
}
fn replace(path: &Path, bytes: &[u8]) -> Result<()> {
    fs::create_dir_all(path.parent().unwrap())?;
    let mut temp = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
    if let Ok(metadata) = fs::metadata(path) {
        temp.as_file().set_permissions(metadata.permissions())?;
    }
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| e.error)?;
    #[cfg(unix)]
    fs::File::open(path.parent().unwrap())?.sync_all()?;
    Ok(())
}
#[derive(Serialize, Deserialize)]
struct Entry {
    path: String,
    before: Option<Vec<u8>>,
    after_hash: Option<String>,
}
pub fn commit(root: &Path, before: &Files, after: &Files, fail_after: Option<usize>) -> Result<()> {
    let keys: std::collections::BTreeSet<_> = before.keys().chain(after.keys()).collect();
    let mut entries = Vec::new();
    for name in keys {
        if before.get(name) == after.get(name) {
            continue;
        }
        ensure!(
            name == "deck.json"
                || name == style::PATH
                || name.starts_with("slides/") && name.ends_with(".html")
                || name.starts_with("assets/editor-"),
            "Unauthorized write: {name}"
        );
        let path = safe_target(root, name)?;
        let actual = match fs::read(&path) {
            Ok(b) => Some(b),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        ensure!(
            actual.as_ref() == before.get(name),
            "CONFLICT: {name} changed before writing"
        );
        entries.push(Entry {
            path: name.clone(),
            before: actual,
            after_hash: after.get(name).map(|b| hash(b)),
        });
    }
    if entries.is_empty() {
        return Ok(());
    }
    let journal = root.join(JOURNAL);
    fs::create_dir(&journal).context("Another save is active or a previous save needs recovery")?;
    if let Err(e) = replace(
        &journal.join("journal.json"),
        &serde_json::to_vec(&entries)?,
    ) {
        let _ = fs::remove_dir_all(&journal);
        return Err(e);
    }
    #[cfg(unix)]
    fs::File::open(root)?.sync_all()?;
    let result = (|| -> Result<()> {
        for (i, e) in entries.iter().enumerate() {
            if fail_after == Some(i) {
                bail!("Simulated write failure");
            }
            let path = safe_target(root, &e.path)?;
            // Do not replace a concurrently edited file even after journal creation.
            ensure!(
                fs::read(&path).ok() == e.before,
                "CONFLICT: {} changed during save",
                e.path
            );
            if let Some(bytes) = after.get(&e.path) {
                replace(&path, bytes)?;
            } else if path.exists() {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    })();
    if let Err(e) = result {
        recover(root).context("Save failed; automatic rollback also failed. Keep the recovery journal and run edit --recover")?;
        return Err(e);
    }
    fs::remove_dir_all(journal)?;
    #[cfg(unix)]
    fs::File::open(root)?.sync_all()?;
    Ok(())
}
pub fn recover(root: &Path) -> Result<()> {
    let journal = root.join(JOURNAL);
    ensure!(
        !fs::symlink_metadata(&journal)?.file_type().is_symlink(),
        "Recovery journal cannot be a symlink"
    );
    let journal_file = journal.join("journal.json");
    // A missing durable journal means no writes started, or completed writes have
    // reached journal cleanup. Neither state requires source rollback.
    if !journal_file.exists() && fs::symlink_metadata(&journal_file).is_err() {
        fs::remove_dir_all(journal)?;
        #[cfg(unix)]
        fs::File::open(root)?.sync_all()?;
        return Ok(());
    }
    ensure!(
        !fs::symlink_metadata(&journal_file)?
            .file_type()
            .is_symlink(),
        "Recovery data cannot be a symlink"
    );
    let entries: Vec<Entry> = serde_json::from_slice(&fs::read(journal_file)?)?;
    for e in &entries {
        ensure!(
            e.path == "deck.json"
                || e.path == style::PATH
                || e.path.starts_with("slides/") && e.path.ends_with(".html")
                || e.path.starts_with("assets/editor-"),
            "Invalid recovery target"
        );
        let path = safe_target(root, &e.path)?;
        let actual = fs::read(&path).ok();
        ensure!(
            actual == e.before || actual.as_ref().map(|b| hash(b)) == e.after_hash,
            "Recovery stopped: {} was edited since the interrupted save; keep journal and reconcile manually",
            e.path
        );
    }
    for e in entries.iter().rev() {
        let path = safe_target(root, &e.path)?;
        if let Some(bytes) = &e.before {
            replace(&path, bytes)?;
        } else if path.exists() {
            fs::remove_file(path)?;
        }
    }
    fs::remove_dir_all(journal)?;
    #[cfg(unix)]
    fs::File::open(root)?.sync_all()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rollback_multi_file_failure() {
        let root = tempfile::tempdir().unwrap();
        let before = Files::from([
            ("deck.json".into(), b"old".to_vec()),
            ("slides/a.html".into(), b"original".to_vec()),
        ]);
        for (n, b) in &before {
            replace(&root.path().join(n), b).unwrap();
        }
        let after = Files::from([
            ("deck.json".into(), b"new".to_vec()),
            ("slides/a.html".into(), b"changed".to_vec()),
        ]);
        assert!(commit(root.path(), &before, &after, Some(1)).is_err());
        for (n, b) in before {
            assert_eq!(fs::read(root.path().join(n)).unwrap(), b);
        }
        assert!(!root.path().join(JOURNAL).exists());
    }
}
