#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
import io
import re
import zipfile
from pathlib import Path

TONE_RE = re.compile(r"^[a-z]+[1-6]$")


# Unicode Unihan kCantonese data is distributed under the Unicode License; see
# https://www.unicode.org/license.txt. Jyutping has six tone numbers; the
# Linguistic Society of Hong Kong Jyutping scheme describes tone 6 as low-level,
# so rare toneless kCantonese syllables are normalized by appending "6" to keep
# every sort key in the same lowercase-letters-plus-tone-digit shape.


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Convert Unicode Unihan kCantonese data into Jyutping CSV."
    )
    parser.add_argument(
        "input",
        type=Path,
        help="Path to Unihan_Readings.txt or a Unihan.zip archive.",
    )
    parser.add_argument("output", type=Path, help="Path to the output CSV file.")
    return parser.parse_args()


def iter_lines(path: Path) -> io.TextIOBase:
    if path.suffix.lower() == ".zip":
        archive = zipfile.ZipFile(path)
        return io.TextIOWrapper(archive.open("Unihan_Readings.txt"), encoding="utf-8")
    return path.open("r", encoding="utf-8")


def codepoint_to_char(codepoint: str) -> str:
    return chr(int(codepoint.removeprefix("U+"), 16))


def normalize_jyutping(reading: str, codepoint: str) -> str:
    normalized = reading.strip().lower()
    if not normalized:
        raise ValueError(f"empty kCantonese reading for {codepoint}")
    if not normalized[-1].isdigit():
        normalized = f"{normalized}6"
    if not normalized.isascii():
        raise ValueError(f"non-ASCII kCantonese reading {reading!r} for {codepoint}")
    if not TONE_RE.fullmatch(normalized):
        raise ValueError(
            f"kCantonese reading {reading!r} for {codepoint} did not normalize to "
            "lowercase letters plus tone 1-6"
        )
    return normalized


def main() -> None:
    args = parse_args()

    rows: list[tuple[str, str, str]] = []
    with iter_lines(args.input) as handle:
        for raw_line in handle:
            line = raw_line.strip()
            if not line or line.startswith("#"):
                continue

            codepoint, field, value = line.split("\t", 2)
            if field != "kCantonese":
                continue

            readings = [normalize_jyutping(item, codepoint) for item in value.split()]
            rows.append((codepoint, "|".join(readings), codepoint_to_char(codepoint)))

    rows.sort(key=lambda row: int(row[0].removeprefix("U+"), 16))

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(["codepoint", "jyutping", "char"])
        writer.writerows(rows)


if __name__ == "__main__":
    main()
