#!/usr/bin/env python3
"""Genera latest.json a partir de una release y sus checksums descargados.

Uso: latest_json.py <directorio-con-release.json-y-*.sha256> <salida>
"""

from __future__ import annotations

import json
import pathlib
import sys

SUFFIXES = (".zip", ".tar.gz", ".tar.xz", ".tar.zst", ".msi")


def build(workdir: pathlib.Path, output: pathlib.Path) -> dict[str, object]:
    release = json.loads((workdir / "release.json").read_text(encoding="utf-8"))

    hashes: dict[str, str] = {}
    for checksum in workdir.glob("*.sha256"):
        parts = checksum.read_text(encoding="utf-8").split()
        if parts:
            hashes[checksum.name.removesuffix(".sha256")] = parts[0]

    assets = []
    for asset in release["assets"]:
        name = asset["name"]
        if name.startswith("tchat-cli-installer"):
            continue
        if not name.endswith(SUFFIXES):
            continue
        assets.append(
            {
                "name": name,
                "url": asset.get("downloadUrl") or asset.get("url"),
                "size": asset["size"],
                "sha256": hashes.get(name),
            }
        )

    tag = release["tagName"]
    payload = {
        "version": tag.removeprefix("tchat-cli-v").removeprefix("v"),
        "tag": tag,
        "released_at": release["publishedAt"],
        "assets": assets,
    }

    output.write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    return payload


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("uso: latest_json.py <workdir> <salida>")

    payload = build(pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]))
    assets = payload["assets"]
    print(f"latest.json: {len(assets)} artefactos, versión {payload['version']}")


if __name__ == "__main__":
    main()
