use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    files
}
fn source_skill() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("plugins/presmith/skills/presmith")
}
fn command(binary: &Path, cwd: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(binary)
        .current_dir(cwd)
        .args(args)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("PATH", "")
        .output()
        .unwrap()
}
fn install(home: &Path, force: bool) -> (Output, Value) {
    let mut args = vec!["skill", "install", "--global", "--json"];
    if force {
        args.push("--force");
    }
    let out = command(Path::new(env!("CARGO_BIN_EXE_presmith")), home, home, &args);
    let value = serde_json::from_slice(&out.stdout).expect("JSON result");
    (out, value)
}
fn target(home: &Path) -> PathBuf {
    home.join(".agents/skills/presmith")
}

#[test]
fn relocated_binary_bundles_complete_skill_in_new_deck_and_global_install() {
    let tmp = tempfile::tempdir().unwrap();
    let binary = tmp.path().join(format!(
        "standalone-presmith{}",
        std::env::consts::EXE_SUFFIX
    ));
    fs::copy(env!("CARGO_BIN_EXE_presmith"), &binary).unwrap();
    let home = tmp.path().join("home with spaces");
    fs::create_dir(&home).unwrap();
    // The copied executable has no helpers on PATH and runs outside the repository.
    let out = command(&binary, &home, &home, &["init", "talk with spaces"]);
    assert!(out.status.success(), "{out:?}");
    let deck = home.join("talk with spaces");
    assert_eq!(
        tree(&deck.join(".agents/skills/presmith")),
        tree(&source_skill())
    );
    assert!(!target(&home).exists(), "init must not install globally");
    let out = command(
        &binary,
        &home,
        &home,
        &["skill", "install", "--global", "--json"],
    );
    assert!(out.status.success(), "{out:?}");
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["status"], "installed");
    assert_eq!(tree(&target(&home)), tree(&source_skill()));
    assert_eq!(v["artifacts"][0]["files"], tree(&source_skill()).len());
    assert!(!home.join(".codex").exists());
    assert!(!deck.join("tooling/renderer/node_modules").exists());
}

#[test]
fn identical_global_install_is_a_noop_even_with_force() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(install(tmp.path(), false).0.status.success());
    let skill = target(tmp.path()).join("SKILL.md");
    let modified = fs::metadata(&skill).unwrap().modified().unwrap();
    for force in [false, true] {
        let (out, v) = install(tmp.path(), force);
        assert!(out.status.success(), "{out:?}");
        assert_eq!(v["status"], "unchanged");
        assert_eq!(v["artifacts"].as_array().unwrap().len(), 1);
        assert_eq!(fs::metadata(&skill).unwrap().modified().unwrap(), modified);
    }
    assert!(!tmp.path().join(".agents/.presmith-skill-backups").exists());
}

#[test]
fn replacement_requires_force_and_preserves_every_previous_file_in_backup() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(install(tmp.path(), false).0.status.success());
    let dest = target(tmp.path());
    fs::write(dest.join("SKILL.md"), "custom instructions").unwrap();
    fs::write(dest.join("references/personal.md"), "custom reference").unwrap();
    let other = tmp.path().join(".agents/skills/another-skill");
    fs::create_dir(&other).unwrap();
    fs::write(other.join("SKILL.md"), "untouched").unwrap();
    let before = tree(&dest);
    let (out, v) = install(tmp.path(), false);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(v["success"], false);
    assert!(v["error"]["message"].as_str().unwrap().contains("--force"));
    assert_eq!(tree(&dest), before);
    let (out, v) = install(tmp.path(), true);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(v["status"], "updated");
    let backup = PathBuf::from(v["artifacts"][1]["path"].as_str().unwrap());
    assert!(!backup.starts_with(tmp.path().join(".agents/skills")));
    assert_eq!(tree(&backup), before);
    assert_eq!(tree(&dest), tree(&source_skill()));
    assert_eq!(
        fs::read_to_string(other.join("SKILL.md")).unwrap(),
        "untouched"
    );
}

#[test]
fn missing_or_extra_skill_files_are_not_silently_overwritten() {
    for missing in [true, false] {
        let tmp = tempfile::tempdir().unwrap();
        assert!(install(tmp.path(), false).0.status.success());
        let dest = target(tmp.path());
        if missing {
            fs::remove_file(dest.join("references/authoring.md")).unwrap();
        } else {
            fs::write(dest.join("custom.md"), "keep me").unwrap();
        }
        let before = tree(&dest);
        assert_eq!(install(tmp.path(), false).0.status.code(), Some(2));
        assert_eq!(tree(&dest), before);
    }
}

#[test]
fn invalid_home_or_missing_scope_fails_without_writing_to_cwd() {
    let tmp = tempfile::tempdir().unwrap();
    let exe = Path::new(env!("CARGO_BIN_EXE_presmith"));
    for home in ["", "relative-home"] {
        let out = command(
            exe,
            tmp.path(),
            Path::new(home),
            &["skill", "install", "--global", "--json"],
        );
        assert_eq!(out.status.code(), Some(2));
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert!(v["error"]["message"].as_str().unwrap().contains("absolute"));
    }
    let out = Command::new(exe)
        .current_dir(tmp.path())
        .env_remove("HOME")
        .env_remove("USERPROFILE")
        .args(["skill", "install", "--global", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["error"]["message"].as_str().unwrap().contains("not set"));
    let out = command(exe, tmp.path(), tmp.path(), &["skill", "install", "--json"]);
    assert_eq!(out.status.code(), Some(2));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["error"]["kind"], "usage");
    assert!(!tmp.path().join(".agents").exists());
}

#[test]
fn existing_file_destination_is_preserved_even_with_force() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join(".agents/skills")).unwrap();
    fs::write(target(tmp.path()), "keep me").unwrap();
    assert_eq!(install(tmp.path(), true).0.status.code(), Some(2));
    assert_eq!(fs::read_to_string(target(tmp.path())).unwrap(), "keep me");
}

#[test]
fn competing_installer_lock_preserves_existing_skill() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(install(tmp.path(), false).0.status.success());
    fs::write(target(tmp.path()).join("SKILL.md"), "custom").unwrap();
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(
            tmp.path()
                .join(".agents/skills/.presmith-skill-install.lock"),
        )
        .unwrap();
    lock.lock().unwrap();
    let (out, v) = install(tmp.path(), true);
    assert_eq!(out.status.code(), Some(2));
    assert!(v["error"]["message"].as_str().unwrap().contains("Another"));
    assert_eq!(
        fs::read_to_string(target(tmp.path()).join("SKILL.md")).unwrap(),
        "custom"
    );
}

#[cfg(unix)]
#[test]
fn installer_rejects_symlink_destinations_and_parents() {
    for relative in [
        ".agents",
        ".agents/skills",
        ".agents/skills/presmith",
        ".agents/skills/.presmith-skill-install.lock",
    ] {
        let tmp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let link = tmp.path().join(relative);
        fs::create_dir_all(link.parent().unwrap()).unwrap();
        // Dangling links must be protected too.
        std::os::unix::fs::symlink(outside.path().join("missing"), &link).unwrap();
        let (out, v) = install(tmp.path(), true);
        assert_eq!(out.status.code(), Some(2));
        assert!(v["error"]["message"].as_str().unwrap().contains("symlink"));
        assert!(link.is_symlink());
        assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
    }
}

#[cfg(unix)]
#[test]
fn backup_failure_leaves_existing_skill_intact() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(install(tmp.path(), false).0.status.success());
    fs::write(target(tmp.path()).join("SKILL.md"), "custom").unwrap();
    let before = tree(&target(tmp.path()));
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(
        outside.path(),
        tmp.path().join(".agents/.presmith-skill-backups"),
    )
    .unwrap();
    assert_eq!(install(tmp.path(), true).0.status.code(), Some(2));
    assert_eq!(tree(&target(tmp.path())), before);
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
}
