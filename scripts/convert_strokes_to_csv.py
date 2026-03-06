#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
import io
import zipfile
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Convert Unicode Unihan kTotalStrokes data into a CSV file."
    )
    parser.add_argument(
        "input",
        type=Path,
        help="Path to Unihan_IRGSources.txt or a Unihan.zip archive.",
    )
    parser.add_argument("output", type=Path, help="Path to the output CSV file.")
    parser.add_argument(
        "--filter-csv",
        type=Path,
        help="Optional CSV with a codepoint column used to filter the output set.",
    )
    return parser.parse_args()


def load_allowed_codepoints(path: Path | None) -> set[str] | None:
    if path is None:
        return None

    allowed: set[str] = set()
    with path.open("r", encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            allowed.add(row["codepoint"])
    return allowed


def iter_lines(path: Path) -> io.TextIOBase:
    if path.suffix.lower() == ".zip":
        archive = zipfile.ZipFile(path)
        member = "Unihan_IRGSources.txt"
        return io.TextIOWrapper(archive.open(member), encoding="utf-8")
    return path.open("r", encoding="utf-8")


def codepoint_to_char(codepoint: str) -> str:
    return chr(int(codepoint.removeprefix("U+"), 16))


def main() -> None:
    args = parse_args()
    allowed = load_allowed_codepoints(args.filter_csv)

    rows: list[tuple[str, str, str]] = []
    with iter_lines(args.input) as handle:
        for raw_line in handle:
            line = raw_line.strip()
            if not line or line.startswith("#"):
                continue

            codepoint, field, value = line.split("\t", 2)
            if field != "kTotalStrokes":
                continue
            if allowed is not None and codepoint not in allowed:
                continue
            rows.append((codepoint, value, codepoint_to_char(codepoint)))

    rows.sort(key=lambda row: int(row[0].removeprefix("U+"), 16))

    with args.output.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerow(["codepoint", "strokes", "char"])
        writer.writerows(rows)


if __name__ == "__main__":
    main()
