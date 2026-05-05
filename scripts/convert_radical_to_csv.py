#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
import io
import re
import sys
import urllib.request
import zipfile
from pathlib import Path
from typing import Iterator, TextIO

PROJECT_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_UNIHAN_URL = "https://www.unicode.org/Public/UCD/latest/ucd/Unihan.zip"
DEFAULT_OUTPUT = PROJECT_ROOT / "data" / "radical.csv"
RS_UNICODE_RE = re.compile(r"^(\d+)'*\.(-?\d+)$")


def parse_args() -> argparse.Namespace:
	parser = argparse.ArgumentParser(
		description="Convert Unicode Unihan kRSUnicode data into a radical CSV file."
	)
	parser.add_argument(
		"input",
		nargs="?",
		type=Path,
		help=(
			"Path to Unihan_IRGSources.txt or a Unihan.zip archive. "
			"When omitted, the latest Unihan.zip is fetched from unicode.org."
		),
	)
	parser.add_argument(
		"output",
		nargs="?",
		type=Path,
		default=DEFAULT_OUTPUT,
		help=f"Path to the output CSV file (default: {DEFAULT_OUTPUT}).",
	)
	return parser.parse_args()


def open_unihan_source(path: Path | None) -> TextIO:
	if path is None:
		with urllib.request.urlopen(DEFAULT_UNIHAN_URL, timeout=120) as response:
			payload = response.read()
		archive = zipfile.ZipFile(io.BytesIO(payload))
		return io.TextIOWrapper(archive.open("Unihan_IRGSources.txt"), encoding="utf-8")

	if path.suffix.lower() == ".zip":
		archive = zipfile.ZipFile(path)
		return io.TextIOWrapper(archive.open("Unihan_IRGSources.txt"), encoding="utf-8")

	return path.open("r", encoding="utf-8")


def codepoint_to_char(codepoint: str) -> str:
	return chr(int(codepoint.removeprefix("U+"), 16))


def parse_rs_unicode(value: str, codepoint: str) -> tuple[int, int]:
	first = value.split()[0]
	match = RS_UNICODE_RE.fullmatch(first)
	if match is None:
		raise ValueError(f"unsupported kRSUnicode value for {codepoint}: {value!r}")

	radical = int(match.group(1))
	residual = int(match.group(2))
	if not 1 <= radical <= 214:
		raise ValueError(f"radical out of range for {codepoint}: {radical}")
	return radical, residual


def iter_radical_rows(handle: TextIO) -> Iterator[tuple[str, int, int, str]]:
	for line_number, raw_line in enumerate(handle, start=1):
		line = raw_line.strip()
		if not line or line.startswith("#"):
			continue

		try:
			codepoint, field, value = line.split("\t", 2)
		except ValueError as exc:
			raise ValueError(f"line {line_number}: expected 3 tab-separated fields") from exc

		if field != "kRSUnicode":
			continue

		radical, residual = parse_rs_unicode(value, codepoint)
		yield codepoint, radical, residual, codepoint_to_char(codepoint)


def main() -> None:
	args = parse_args()
	rows: list[tuple[str, int, int, str]] = []
	try:
		with open_unihan_source(args.input) as handle:
			rows.extend(iter_radical_rows(handle))
	except Exception as exc:
		print(f"failed to read Unihan radical data: {exc}", file=sys.stderr)
		raise SystemExit(1) from exc

	rows.sort(key=lambda row: int(row[0].removeprefix("U+"), 16))
	args.output.parent.mkdir(parents=True, exist_ok=True)
	with args.output.open("w", encoding="utf-8", newline="") as handle:
		writer = csv.writer(handle, lineterminator="\n")
		writer.writerow(["codepoint", "radical", "residual", "char"])
		writer.writerows(rows)


if __name__ == "__main__":
	main()
