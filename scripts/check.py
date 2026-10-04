"""Validate Rust and run isolated, offline CLI regressions without device access."""

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
VERSION = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
BINARY = None


def run(command, **kwargs):
    return subprocess.run(command, check=True, cwd=ROOT, **kwargs)


def build():
    env = dict(os.environ)
    env["FEROS_BUILD_COMMIT"] = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    env["FEROS_BUILD_TIMESTAMP"] = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    for args in (["fmt", "--all", "--check"],
                 ["check", "--all-targets", "--locked"],
                 ["test", "--locked"],
                 ["clippy", "--all-targets", "--locked", "--", "-D", "warnings"],
                 ["build", "--release", "--locked"]):
        run(["cargo", *args], env=env)
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"],
        cwd=ROOT, text=True))
    return Path(metadata["target_directory"]) / "release" / (
        "flasher.exe" if os.name == "nt" else "flasher")


class CliChecks(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="flasher-check-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.binary = self.root / BINARY.name
        shutil.copy2(BINARY, self.binary)
        self.cwd = self.root / "unrelated-working-directory"
        self.cwd.mkdir()
        self.config = self.root / "config" / "repositories.toml"

    def invoke(self, *args, success=True, input=""):
        result = subprocess.run([str(self.binary), *args], input=input,
                                cwd=self.cwd, capture_output=True, text=True,
                                encoding="utf-8", timeout=20)
        output = result.stdout + result.stderr
        if success:
            self.assertEqual(result.returncode, 0, output)
        else:
            self.assertNotEqual(result.returncode, 0, output)
        self.assertNotIn("panicked at", output)
        return output

    def seed(self):
        self.config.parent.mkdir()
        self.config.write_text('''[[repositories]]
name = "Official fixture"
url = "https://example.org/official"
official = true
is_default = true
published_date = "2026-10-04T00:00:00Z"

[[repositories]]
name = "Community fixture"
url = "https://example.org/community"
official = false
is_default = false
published_date = "2026-10-04T00:00:00Z"
''', encoding="utf-8")

    def entries(self):
        return tomllib.loads(self.config.read_text())["repositories"]

    def test_help_and_version(self):
        self.assertIn("FeROS Flasher", self.invoke("help"))
        self.assertIn("Version: " + VERSION, self.invoke("version"))
        self.assertIn("--target", self.invoke("help", "--flash"))
        help_text = self.invoke("help", "--channel")
        for option in ("--add", "--list", "--select", "--delete"):
            self.assertIn(option, help_text)
        self.assertNotIn("--update", help_text)

    def test_fresh_configuration(self):
        self.assertIn("none selected", self.invoke("channel"))
        self.assertIn("No repositories configured", self.invoke("channel", "--list"))
        self.assertFalse(self.config.exists())

    def test_default_selection_and_deletion(self):
        self.seed()
        self.assertIn("Official fixture", self.invoke("channel"))
        self.invoke("channel", "--select", "2")
        self.assertEqual([r["is_default"] for r in self.entries()], [False, True])
        self.assertIn("Community fixture", self.invoke("channel"))
        self.invoke("channel", "--delete", "1")
        self.assertEqual([r["name"] for r in self.entries()], ["Community fixture"])
        self.assertFalse((self.cwd / "config").exists())

    def test_invalid_indices_preserve_configuration(self):
        self.seed()
        before = self.config.read_bytes()
        for operation in ("--select", "--delete"):
            for number in ("0", "3", "999"):
                self.assertIn("out of range", self.invoke(
                    "channel", operation, number, success=False))
                self.assertEqual(self.config.read_bytes(), before)

    def test_cannot_delete_default(self):
        self.seed()
        before = self.config.read_bytes()
        self.assertIn("cannot delete the default", self.invoke(
            "channel", "--delete", "1", success=False))
        self.assertEqual(before, self.config.read_bytes())

    def test_conflicting_operations(self):
        self.seed()
        before = self.config.read_bytes()
        for args in (("--add", "--list"), ("--select", "1", "--delete", "2"),
                     ("--select", "1", "--list")):
            self.invoke("channel", *args, success=False)
        self.assertEqual(before, self.config.read_bytes())

    def test_invalid_add_never_reaches_network(self):
        for address in ("http://example.org", "https://user:secret@example.org"):
            self.invoke("channel", "--add", input=address + "\nFixture\nn\n", success=False)
        self.assertFalse(self.config.exists())

    def test_malformed_configuration_is_not_overwritten(self):
        self.config.parent.mkdir()
        self.config.write_text("not valid toml [", encoding="utf-8")
        before = self.config.read_bytes()
        self.assertIn("invalid local repository store", self.invoke(
            "channel", "--select", "1", success=False))
        self.assertEqual(before, self.config.read_bytes())


def check_binary(binary):
    global BINARY
    BINARY = Path(binary).resolve(strict=True)
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(CliChecks)
    if not suite.countTestCases():
        raise RuntimeError("No CLI tests discovered")
    if not unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful():
        raise RuntimeError("CLI regression checks failed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path,
                        help="Only test a copied executable; skip Cargo validation/build")
    args = parser.parse_args()
    try:
        check_binary(args.binary if args.binary else build())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        parser.exit(1, f"Validation failed: {error}\n")
