#!/usr/bin/env python3
"""Validate, publish, and distribute immutable Presmith releases."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tarfile

REPOSITORY = "RyoOuchi/Presmith"
TARGET = "aarch64-apple-darwin"
VERSION = r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?"


def version_key(version):
    match = re.fullmatch(VERSION, version)
    if not match:
        raise ValueError(f"Unsupported release version: {version}")
    major, minor, patch, preview = match.groups()
    identifiers = []
    for part in preview.split(".") if preview else []:
        if part.isdigit() and len(part) > 1 and part.startswith("0"):
            raise ValueError("Numeric prerelease identifiers cannot have leading zeros")
        identifiers.append((0, int(part)) if part.isdigit() else (1, part))
    return (int(major), int(minor), int(patch), not preview, tuple(identifiers))


def command(*args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True).strip()


def metadata(root, tag=""):
    cargo = json.loads(command("cargo", "metadata", "--locked", "--no-deps",
                               "--format-version", "1", cwd=root))
    package = next(p for p in cargo["packages"] if p["name"] == "presmith")
    version = package["version"]
    version_key(version)
    expected_tag = "v" + version
    if tag and tag != expected_tag:
        raise ValueError(f"Tag {tag} does not match Cargo version {expected_tag}")
    notes = root / "docs/releases" / (expected_tag + ".md")
    if not notes.is_file():
        raise ValueError(f"Missing release notes: {notes}")
    plugin = json.loads((root / "plugins/presmith/plugin.json").read_text())["version"]
    version_key(plugin)
    return {
        "version": version, "tag": expected_tag, "target": TARGET,
        "commit": command("git", "rev-parse", "HEAD", cwd=root),
        "prerelease": version.startswith("0.") or "-" in version,
        "plugin_version": plugin,
        "cli": f"presmith-{expected_tag}-{TARGET}.tar.gz",
        "plugin": f"presmith-plugin-v{plugin}.tar.gz",
    }


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(directory, meta, binary_out=None):
    expected = {meta["cli"], meta["plugin"]}
    hashes = {}
    for line in (directory / "SHA256SUMS").read_text().splitlines():
        match = re.fullmatch(r"([a-f0-9]{64})  ([A-Za-z0-9._-]+)", line)
        if not match or match[2] in hashes:
            raise ValueError("Malformed or duplicate checksum entry")
        hashes[match[2]] = match[1]
    if set(hashes) != expected:
        raise ValueError("Checksums must cover exactly the CLI and plugin archives")
    for name, checksum in hashes.items():
        if digest(directory / name) != checksum:
            raise ValueError(f"Checksum mismatch: {name}")
    top = f"presmith-{meta['tag']}-{meta['target']}"
    with tarfile.open(directory / meta["cli"], "r:gz") as archive:
        info = json.load(archive.extractfile(top + "/BUILD-INFO.json"))
        for key in ("version", "target", "commit"):
            if info[key] != meta[key]:
                raise ValueError(f"Release build information mismatch: {key}")
        if info.get("dirty") is not False:
            raise ValueError("Release must be built from a clean checkout")
        binary = archive.extractfile(top + "/presmith").read()
        if hashlib.sha256(binary).hexdigest() != info["binary_sha256"]:
            raise ValueError("Binary checksum mismatch")
    with tarfile.open(directory / meta["plugin"], "r:gz") as archive:
        plugin = json.load(archive.extractfile("presmith/plugin.json"))
        if plugin["version"] != meta["plugin_version"]:
            raise ValueError("Plugin version mismatch")
    if binary_out:
        binary_out.parent.mkdir(parents=True, exist_ok=True)
        binary_out.write_bytes(binary)
        binary_out.chmod(0o755)
    return hashes


def publish(directory, meta, verified, root):
    verify(directory, meta)
    tag = meta["tag"]
    existing = subprocess.run(
        ["gh", "release", "view", tag, "--repo", REPOSITORY,
         "--json", "isDraft,tagName"], capture_output=True, text=True)
    if existing.returncode:
        if "release not found" not in existing.stderr.lower():
            raise RuntimeError(existing.stderr.strip())
        assets = [str(directory / name) for name in (meta["cli"], meta["plugin"], "SHA256SUMS")]
        command("gh", "release", "create", tag, *assets, "--repo", REPOSITORY,
                "--verify-tag", "--draft", "--title", "Presmith " + tag,
                "--notes-file", str(root / "docs/releases" / (tag + ".md")))
        draft = True
    else:
        draft = json.loads(existing.stdout)["isDraft"]
    verified.mkdir(parents=True, exist_ok=False)
    command("gh", "release", "download", tag, "--repo", REPOSITORY,
            "--dir", str(verified), "--pattern", meta["cli"],
            "--pattern", meta["plugin"], "--pattern", "SHA256SUMS")
    # Reruns reuse verified remote bytes, even if a fresh build has different tar timestamps.
    # Incomplete/conflicting drafts fail here and are left intact for inspection.
    verify(verified, meta)
    if draft:
        command("gh", "release", "edit", tag, "--repo", REPOSITORY, "--draft=false",
                "--prerelease=" + str(meta["prerelease"]).lower())
    print(f"Verified published release: https://github.com/{REPOSITORY}/releases/tag/{tag}")


def update_formula(formula, meta, hashes):
    text = formula.read_text()
    pattern = (r'^  url "https://github.com/RyoOuchi/Presmith/releases/download/'
               r'v([^/]+)/presmith-v[^/]+-aarch64-apple-darwin\.tar\.gz"$')
    current = re.findall(pattern, text, re.MULTILINE)
    if len(current) != 1:
        raise ValueError("Expected exactly one Presmith release URL")
    before, after = version_key(current[0]), version_key(meta["version"])
    checksum = re.findall(r'^  sha256 "([a-f0-9]{64})"$', text, re.MULTILINE)
    if len(checksum) != 1:
        raise ValueError("Expected exactly one archive checksum")
    if after < before:
        raise ValueError("Refusing to downgrade the Homebrew formula")
    if after == before:
        if checksum[0] != hashes[meta["cli"]]:
            raise ValueError("Refusing to change a published version's archive checksum")
        return False
    url = f"https://github.com/{REPOSITORY}/releases/download/{meta['tag']}/{meta['cli']}"
    text = re.sub(pattern, f'  url "{url}"', text, flags=re.MULTILINE)
    text = re.sub(r'^  sha256 "[a-f0-9]{64}"$', f'  sha256 "{hashes[meta["cli"]]}"',
                  text, flags=re.MULTILINE)
    text = re.sub(r'^  revision [0-9]+\n', '', text, flags=re.MULTILINE)
    formula.write_text(text)
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    info = sub.add_parser("metadata")
    info.add_argument("--tag", default="")
    info.add_argument("--output", type=Path, required=True)
    info.add_argument("--github-output", type=Path)
    for name in ("verify", "publish", "update-formula"):
        action = sub.add_parser(name)
        action.add_argument("--metadata", type=Path, required=True)
        action.add_argument("--artifacts", type=Path, required=True)
        if name == "verify":
            action.add_argument("--binary-out", type=Path)
        elif name == "publish":
            action.add_argument("--verified-dir", type=Path, required=True)
        else:
            action.add_argument("--formula", type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    if args.command == "metadata":
        meta = metadata(root, args.tag)
        args.output.write_text(json.dumps(meta, indent=2) + "\n")
        if args.github_output:
            with args.github_output.open("a") as output:
                for key in ("tag", "version", "cli", "plugin"):
                    output.write(f"{key}={meta[key]}\n")
        print(meta["tag"])
        return
    meta = json.loads(args.metadata.read_text())
    if args.command == "verify":
        verify(args.artifacts, meta, args.binary_out)
    elif args.command == "publish":
        publish(args.artifacts, meta, args.verified_dir, root)
    else:
        changed = update_formula(args.formula, meta, verify(args.artifacts, meta))
        print("Updated Homebrew formula" if changed else "Homebrew formula already current")


if __name__ == "__main__":
    main()
