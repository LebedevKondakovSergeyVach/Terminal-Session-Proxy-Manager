"""Render the native Homebrew formula after validating all release archives."""

import argparse
import hashlib
from pathlib import Path
import re
import tarfile


REPOSITORY = "https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager"


def render_formula(version, artifacts):
    """Return Ruby with one verified download for each supported OS and CPU."""
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        raise ValueError("Expected a numeric X.Y.Z release version")

    lines = [
        "class Tspm < Formula",
        '  desc "Per-session SOCKS/HTTP proxy profiles, checks, and shell exports"',
        f'  homepage "{REPOSITORY}"',
        f'  version "{version}"',
    ]
    for os_name in ["macos", "linux"]:
        lines.extend(["", f"  on_{os_name} do"])
        for arch, condition in [("arm64", "arm"), ("x86_64", "intel")]:
            name = f"tspm-{os_name}-{arch}.tar.gz"
            archive = artifacts / name / name
            fields = archive.with_name(name + ".sha256").read_text().split()
            if (len(fields) != 2 or fields[1] != name
                    or not re.fullmatch(r"[0-9a-f]{64}", fields[0])):
                raise ValueError(f"Invalid checksum file for {name}")
            digest = hashlib.sha256(archive.read_bytes()).hexdigest()
            if digest != fields[0]:
                raise ValueError(f"Checksum mismatch for {name}")
            with tarfile.open(archive, "r:gz") as tar:
                members = tar.getmembers()
                if (len(members) != 1 or members[0].name != "tspm"
                        or not members[0].isfile() or not members[0].mode & 0o111):
                    raise ValueError(f"Expected one executable tspm in {name}")
            lines.extend([
                f"    on_{condition} do",
                f'      url "{REPOSITORY}/releases/download/v{version}/{name}"',
                f'      sha256 "{digest}"',
                "    end",
            ])
            if condition == "arm":
                lines.append("")
        lines.append("  end")
    lines.extend([
        "",
        "  def install",
        '    bin.install "tspm"',
        "  end",
        "",
        "  test do",
        '    system "#{bin}/tspm", "--version"',
        "  end",
        "end",
        "",
    ])
    return "\n".join(lines)


def main():
    """Write a complete formula only after all four artifacts validate."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--artifacts", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    args.output.write_text(render_formula(args.version, args.artifacts))


if __name__ == "__main__":
    main()
