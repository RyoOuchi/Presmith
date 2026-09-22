#!/usr/bin/env python3
"""Build and package the native macOS Apple Silicon release without publishing."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def run(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def license_files(directory):
    prefixes = ("LICENSE", "LICENCE", "COPYING", "COPYRIGHT", "NOTICE", "UNLICENSE")
    return sorted(p for p in directory.rglob("*") if p.is_file()
                  and p.name.upper().startswith(prefixes))


def notices(metadata, target):
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    reached, pending = set(), [metadata["resolve"]["root"]]
    while pending:
        ident = pending.pop()
        if ident not in reached:
            reached.add(ident)
            pending.extend(nodes[ident]["dependencies"])
    parts = ["Presmith third-party notices\n\n"
             "Rust dependencies reachable for " + target + ". This inventory also\n"
             "includes build dependencies. Components retain their upstream licenses.\n"]
    for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
        if package["id"] not in reached or not package["source"]:
            continue
        directory = Path(package["manifest_path"]).parent
        files = license_files(directory)
        if not files:
            fallback = ROOT / "licenses/dependencies" / (
                package["name"] + "-" + package["version"] + ".txt")
            if not fallback.is_file():
                raise RuntimeError("Missing upstream license text: " + package["name"])
            files = [fallback]
        parts.append("\n" + "=" * 72 + "\n" + package["name"] + " " + package["version"]
                     + "\nDeclared license: " + str(package["license"])
                     + "\nUpstream: " + str(package["repository"] or package.get("homepage")) + "\n")
        for path in files:
            parts.append("\n--- " + path.name + " ---\n" + path.read_text())
    parts.append("\n" + "=" * 72 + "\nBundled editor JavaScript\n")
    parts.append((ROOT / "editor/build/licenses.txt").read_text())
    parts.append((ROOT / "editor/build/editor.js.LEGAL.txt").read_text())
    parts.append("\nRenderer npm packages and Chromium are downloaded separately by\n"
                 "presmith setup; their licenses are provided in those distributions.\n")
    return "\n".join(parts)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--allow-dirty", action="store_true", help="For local validation only")
    args = parser.parse_args()
    dirty = bool(run("git", "status", "--porcelain"))
    if dirty and not args.allow_dirty:
        parser.error("Commit the release sources first, or use --allow-dirty for a local preview")
    target = next(line.split(": ", 1)[1] for line in run("rustc", "-vV").splitlines()
                  if line.startswith("host: "))
    if target != "aarch64-apple-darwin":
        parser.error("This release package is verified only on native Apple Silicon macOS")
    metadata = json.loads(run("cargo", "metadata", "--locked", "--offline", "--format-version", "1",
                              "--filter-platform", target))
    package = next(p for p in metadata["packages"] if p["id"] == metadata["resolve"]["root"])
    version = package["version"]
    if package["name"] != "presmith" or package["license"] != "MIT":
        parser.error("Expected the MIT-licensed presmith package")
    subprocess.run(["cargo", "build", "--release", "--locked"], cwd=ROOT, check=True)
    binary = Path(metadata["target_directory"]) / "release/presmith"
    assert run(str(binary), "--version") == "presmith " + version
    plugin_manifest = json.loads((ROOT / "plugins/presmith/plugin.json").read_text())
    plugin_version = plugin_manifest["version"]
    output = Path(metadata["target_directory"]) / "release-artifacts" / ("v" + version)
    output.mkdir(parents=True, exist_ok=True)
    names = []
    with tempfile.TemporaryDirectory(prefix="presmith-package-") as temporary:
        stage = Path(temporary)
        cli = stage / ("presmith-v" + version + "-" + target)
        cli.mkdir()
        shutil.copy2(binary, cli / "presmith")
        shutil.copy2(ROOT / "LICENSE", cli / "LICENSE")
        shutil.copy2(ROOT / "docs/install.md", cli / "README.md")
        shutil.copy2(ROOT / "docs/releases" / ("v" + version + ".md"), cli / "RELEASE-NOTES.md")
        (cli / "THIRD_PARTY_NOTICES.txt").write_text(notices(metadata, target))
        rust_license = Path(run("rustc", "--print", "sysroot")) / "share/doc/rust/COPYRIGHT-library.html"
        shutil.copy2(rust_license, cli / "RUST_STANDARD_LIBRARY_LICENSES.html")
        (cli / "BUILD-INFO.json").write_text(json.dumps({
            "name": "presmith", "version": version, "target": target,
            "commit": run("git", "rev-parse", "HEAD"), "dirty": dirty,
            "rustc": run("rustc", "--version"),
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "developer_id_signed": False, "notarized": False,
        }, indent=2) + "\n")
        plugin = stage / "presmith"
        shutil.copytree(ROOT / "plugins/presmith", plugin)
        shutil.copy2(ROOT / "LICENSE", plugin / "LICENSE")
        shutil.copy2(ROOT / "docs/install.md", plugin / "INSTALL.md")
        for directory, name in [(cli, cli.name + ".tar.gz"),
                                (plugin, "presmith-plugin-v" + plugin_version + ".tar.gz")]:
            with tarfile.open(output / (name + ".tmp"), "w:gz") as archive:
                archive.add(directory, arcname=directory.name)
            (output / (name + ".tmp")).replace(output / name)
            names.append(name)
    (output / "SHA256SUMS").write_text("".join(
        hashlib.sha256((output / name).read_bytes()).hexdigest() + "  " + name + "\n"
        for name in names))
    print(output)


if __name__ == "__main__":
    main()
