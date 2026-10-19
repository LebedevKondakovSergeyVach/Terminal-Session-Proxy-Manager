"""Exercise release artifact validation and the generated formula's selection."""

import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest

from homebrew_formula import render_formula


class HomebrewFormulaTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.expected = {}
        for os_name in ["macos", "linux"]:
            for arch in ["arm64", "x86_64"]:
                name = f"tspm-{os_name}-{arch}.tar.gz"
                directory = self.root / name
                directory.mkdir()
                payload = f"binary for {os_name}/{arch}".encode()
                archive = directory / name
                with tarfile.open(archive, "w:gz") as tar:
                    member = tarfile.TarInfo("tspm")
                    member.size = len(payload)
                    member.mode = 0o755
                    tar.addfile(member, io.BytesIO(payload))
                digest = hashlib.sha256(archive.read_bytes()).hexdigest()
                (directory / f"{name}.sha256").write_text(f"{digest}  {name}\n")
                self.expected[os_name, arch] = (name, digest)

    def test_the_formula_selects_the_matching_archive_and_checksum(self):
        formula = self.root / "tspm.rb"
        formula.write_text(render_formula("3.0.0", self.root))
        # Execute the emitted Ruby with a minimal DSL receiver. This checks the
        # generated conditions and values, rather than their source formatting.
        receiver = r'''
require "json"
class Formula
  def self.desc(_); end
  def self.homepage(_); end
  def self.version(value); @version = value; end
  def self.url(value); @url = value; end
  def self.sha256(value); @sha256 = value; end
  def self.test; end
  def self.on_macos; yield if ARGV[1] == "macos"; end
  def self.on_linux; yield if ARGV[1] == "linux"; end
  def self.on_arm; yield if ARGV[2] == "arm64"; end
  def self.on_intel; yield if ARGV[2] == "x86_64"; end
end
load ARGV[0]
puts JSON.generate([Tspm.instance_variable_get(:@version),
                    Tspm.instance_variable_get(:@url),
                    Tspm.instance_variable_get(:@sha256)])
'''
        for (os_name, arch), (name, digest) in self.expected.items():
            with self.subTest(os=os_name, arch=arch):
                output = subprocess.check_output(
                    ["ruby", "-e", receiver, str(formula), os_name, arch], text=True
                )
                self.assertEqual(json.loads(output), [
                    "3.0.0",
                    "https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager"
                    f"/releases/download/v3.0.0/{name}",
                    digest,
                ])

    def test_a_missing_platform_prevents_formula_publication(self):
        (self.root / "tspm-linux-arm64.tar.gz/tspm-linux-arm64.tar.gz.sha256").unlink()
        with self.assertRaises(FileNotFoundError):
            render_formula("3.0.0", self.root)

    def test_a_checksum_for_different_bytes_is_rejected(self):
        archive = self.root / "tspm-macos-arm64.tar.gz/tspm-macos-arm64.tar.gz"
        archive.write_bytes(archive.read_bytes() + b"changed")
        with self.assertRaisesRegex(ValueError, "Checksum mismatch"):
            render_formula("3.0.0", self.root)

    def test_an_archive_without_the_executable_is_rejected(self):
        name = "tspm-macos-arm64.tar.gz"
        archive = self.root / name / name
        with tarfile.open(archive, "w:gz") as tar:
            tar.addfile(tarfile.TarInfo("wrong-name"))
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        (archive.parent / f"{name}.sha256").write_text(f"{digest}  {name}\n")
        with self.assertRaisesRegex(ValueError, "executable tspm"):
            render_formula("3.0.0", self.root)

    def test_a_version_cannot_inject_ruby_into_the_formula(self):
        with self.assertRaisesRegex(ValueError, "version"):
            render_formula('3.0.0"; system("false")', self.root)


if __name__ == "__main__":
    unittest.main()
