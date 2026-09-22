"""Release safety checks for version tags, archives, reruns, and formula updates."""
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("release", Path(__file__).resolve().parents[1] / "scripts/release.py")
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.meta = {
            "version": "0.2.1", "tag": "v0.2.1", "target": release.TARGET,
            "commit": "a" * 40, "plugin_version": "0.2.1", "prerelease": True,
            "cli": "presmith-v0.2.1-aarch64-apple-darwin.tar.gz",
            "plugin": "presmith-plugin-v0.2.1.tar.gz",
        }
        binary = b"a compiled CLI fixture"
        info = {**self.meta, "dirty": False, "binary_sha256": hashlib.sha256(binary).hexdigest()}
        top = "presmith-v0.2.1-aarch64-apple-darwin"
        self.archive(self.meta["cli"], {top + "/BUILD-INFO.json": json.dumps(info).encode(), top + "/presmith": binary})
        self.archive(self.meta["plugin"], {"presmith/plugin.json": b'{"version":"0.2.1"}'})
        self.hashes = {self.meta[key]: release.digest(self.root / self.meta[key]) for key in ("cli", "plugin")}
        (self.root / "SHA256SUMS").write_text("".join(f"{sha}  {name}\n" for name, sha in self.hashes.items()))
        self.formula = self.root / "presmith.rb"
        self.formula.write_text('class Presmith < Formula\n'
                               '  url "https://github.com/RyoOuchi/Presmith/releases/download/v0.2.0/presmith-v0.2.0-aarch64-apple-darwin.tar.gz"\n'
                               f'  sha256 "{"b" * 64}"\n  license "MIT"\n  revision 1\n'
                               '  depends_on "node@24"\nend\n')

    def archive(self, name, files):
        with tarfile.open(self.root / name, "w:gz") as archive:
            for filename, content in files.items():
                member = tarfile.TarInfo(filename)
                member.size = len(content)
                archive.addfile(member, io.BytesIO(content))

    def test_matching_archives_and_binary_extraction(self):
        binary = self.root / "extracted/presmith"
        self.assertEqual(release.verify(self.root, self.meta, binary), self.hashes)
        self.assertTrue(binary.stat().st_mode & 0o111)
        self.assertEqual(binary.read_bytes(), b"a compiled CLI fixture")

    def test_corrupt_archive_rejected(self):
        (self.root / self.meta["cli"]).write_bytes(b"corrupt")
        with self.assertRaisesRegex(ValueError, "Checksum mismatch"):
            release.verify(self.root, self.meta)

    def test_wrong_commit_rejected(self):
        with self.assertRaisesRegex(ValueError, "commit"):
            release.verify(self.root, {**self.meta, "commit": "b" * 40})

    def test_duplicate_checksum_rejected(self):
        path = self.root / "SHA256SUMS"
        path.write_text(path.read_text() + path.read_text().splitlines()[0] + "\n")
        with self.assertRaisesRegex(ValueError, "duplicate"):
            release.verify(self.root, self.meta)

    def test_plugin_version_mismatch_rejected(self):
        with self.assertRaisesRegex(ValueError, "Plugin version"):
            release.verify(self.root, {**self.meta, "plugin_version": "0.9.0"})

    def test_version_update_resets_revision_and_preserves_runtime(self):
        self.assertTrue(release.update_formula(self.formula, self.meta, self.hashes))
        text = self.formula.read_text()
        self.assertIn('/v0.2.1/presmith-v0.2.1-', text)
        self.assertNotIn('revision ', text)
        self.assertIn('depends_on "node@24"', text)
        self.assertIn(self.hashes[self.meta["cli"]], text)

    def test_rerun_is_noop(self):
        release.update_formula(self.formula, self.meta, self.hashes)
        self.formula.write_text(self.formula.read_text().replace('  license "MIT"', '  revision 2\n  license "MIT"'))
        before = self.formula.read_text()
        self.assertFalse(release.update_formula(self.formula, self.meta, self.hashes))
        self.assertEqual(self.formula.read_text(), before)

    def test_same_version_cannot_replace_archive(self):
        release.update_formula(self.formula, self.meta, self.hashes)
        with self.assertRaisesRegex(ValueError, "published version"):
            release.update_formula(self.formula, self.meta, {self.meta["cli"]: "c" * 64})

    def test_older_tag_cannot_downgrade_tap(self):
        release.update_formula(self.formula, self.meta, self.hashes)
        with self.assertRaisesRegex(ValueError, "downgrade"):
            release.update_formula(self.formula, {**self.meta, "version": "0.1.9"}, self.hashes)

    def test_semver_order_and_invalid_tags(self):
        versions = ["0.2.0-alpha.2", "0.2.0-alpha.10", "0.2.0-beta", "0.2.0", "0.10.0", "1.0.0"]
        self.assertEqual(sorted(reversed(versions), key=release.version_key), versions)
        for version in ("main", "01.2.0", "1.0.0-alpha.01", "1.0.0/evil"):
            with self.assertRaises(ValueError):
                release.version_key(version)

    @patch.object(release, "command")
    def test_tag_must_match_cargo_and_notes_must_exist(self, command):
        command.side_effect = [json.dumps({"packages": [{"name": "presmith", "version": "0.2.1"}]})]
        with self.assertRaisesRegex(ValueError, "does not match"):
            release.metadata(self.root, "v0.2.2")
        command.side_effect = [json.dumps({"packages": [{"name": "presmith", "version": "0.2.1"}]})]
        with self.assertRaisesRegex(ValueError, "Missing release notes"):
            release.metadata(self.root, "v0.2.1")

    @patch.object(release, "command")
    @patch.object(release.subprocess, "run")
    def test_published_release_is_verified_without_replacement(self, run, command):
        run.return_value.returncode = 0
        run.return_value.stdout = '{"isDraft":false,"tagName":"v0.2.1"}'
        destination = self.root / "downloaded"
        def download(*args, **kwargs):
            self.assertEqual(args[:3], ("gh", "release", "download"))
            for name in (*self.hashes, "SHA256SUMS"):
                (destination / name).write_bytes((self.root / name).read_bytes())
        command.side_effect = download
        release.publish(self.root, self.meta, destination, self.root)
        self.assertEqual(command.call_count, 1)


if __name__ == "__main__":
    unittest.main()
