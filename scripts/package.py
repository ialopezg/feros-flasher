"""Package a clean, tested native Flasher with build identity and checksums."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib

from check import check_binary

ROOT = Path(__file__).resolve().parents[1]
DOCUMENTS = ("README.md", "LICENSE", "CHANGELOG.md", "RELEASE.md", "THIRD_PARTY.md", "Cargo.lock")


def git(*args):
    return subprocess.check_output(["git", "-C", str(ROOT), *args], text=True).strip()


def sha256(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def package(binary, tag):
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    version = manifest["package"]["version"]
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise ValueError("Expected numeric MAJOR.MINOR.PATCH")
    commit = git("rev-parse", "HEAD")
    if tag:
        if tag != f"v{version}" or git("rev-parse", tag + "^{commit}") != commit:
            raise ValueError("Tag must match Cargo.toml and the checked-out commit")
    if git("status", "--porcelain"):
        raise ValueError("Commit the candidate before packaging; working tree is not clean")
    locked = tomllib.loads((ROOT / "Cargo.lock").read_text())
    project = [p for p in locked["package"] if p["name"] == "feros-flasher"]
    if len(project) != 1 or project[0]["version"] != version:
        raise ValueError("Cargo.lock project version does not match Cargo.toml")
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"],
        cwd=ROOT, text=True))
    system = platform.system().lower()
    arch = {"arm64": "aarch64", "amd64": "x86_64"}.get(
        platform.machine().lower(), platform.machine().lower())
    if (system, arch) not in {("darwin", "aarch64"), ("linux", "x86_64"), ("windows", "x86_64")}:
        raise ValueError("Unsupported native release package host")
    exe = "flasher.exe" if system == "windows" else "flasher"
    binary = (binary or Path(metadata["target_directory"]) / "release" / exe).resolve(strict=True)
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise ValueError("A native executable is required")
    result = subprocess.check_output([str(binary), "version"], text=True, encoding="utf-8")
    if f"Version: {version}\n" not in result or f"Commit: {commit}\n" not in result:
        raise ValueError("Binary identity does not match source; run python scripts/check.py")
    if "Built: unknown" in result:
        raise ValueError("Binary build timestamp is missing")
    name = f"feros-flasher-v{version}-{system}-{arch}"
    dist = ROOT / "dist"
    dist.mkdir(exist_ok=True)
    archive = dist / (name + ".tar.gz")
    checksum = dist / (archive.name + ".sha256")
    if archive.exists() or checksum.exists():
        raise ValueError("Package already exists; use a fresh output directory")
    with tempfile.TemporaryDirectory(prefix="flasher-package-") as temp:
        directory = Path(temp)
        stage = directory / name
        stage.mkdir()
        shutil.copy2(binary, stage / exe)
        for document in DOCUMENTS:
            shutil.copy2(ROOT / document, stage / document)
        shutil.copytree(ROOT / "docs", stage / "docs")
        if (ROOT / "licenses").is_dir():
            shutil.copytree(ROOT / "licenses", stage / "licenses")
        info = {
            "product": "feros-flasher", "version": version, "commit": commit, "tag": tag,
            "os": system, "architecture": arch, "host": platform.platform(),
            "rustc": subprocess.check_output(["rustc", "-Vv"], text=True),
            "cargo": subprocess.check_output(["cargo", "-V"], text=True).strip(),
            "cli_version_output": result,
            "executable_sha256": sha256(stage / exe),
            "cargo_lock_sha256": sha256(ROOT / "Cargo.lock"),
            "dependencies": [p for p in locked["package"] if p["name"] != "feros-flasher"],
            "inventory_scope": "Cargo.lock dependency inventory, not a complete binary SBOM",
            "physical_media_backend": "implemented" if system == "darwin" else "not implemented",
        }
        (stage / "BUILD-INFO.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
        check_binary(stage / exe)
        packed = directory / archive.name
        with tarfile.open(packed, "w:gz") as output:
            output.add(stage, arcname=name)
        extracted = directory / "extracted"
        with tarfile.open(packed) as source:
            for member in source.getmembers():
                path = Path(member.name)
                if path.is_absolute() or ".." in path.parts or not (member.isdir() or member.isfile()):
                    raise ValueError("Unexpected archive entry")
            source.extractall(extracted, filter="data")
        payload = extracted / name / exe
        if sha256(payload) != info["executable_sha256"]:
            raise ValueError("Archive executable checksum differs from staging")
        check_binary(payload)
        shutil.copy2(packed, archive)
    checksum.write_text(f"{sha256(archive)}  {archive.name}\n", encoding="utf-8", newline="\n")
    print(archive)
    print(checksum)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--tag", help="Existing vMAJOR.MINOR.PATCH tag to verify")
    args = parser.parse_args()
    try:
        package(args.binary, args.tag)
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        parser.exit(1, f"Packaging failed: {error}\n")
