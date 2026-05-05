#!/usr/bin/env python3
"""Derive Mandarin Zhuyin (Bopomofo) CSV data from bundled pinyin data."""

import argparse
import csv
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
SRC_PINYIN_CSV = PROJECT_ROOT / "data" / "pinyin.csv"
DST_ZHUYIN_CSV = PROJECT_ROOT / "data" / "zhuyin.csv"

TONE_MARKS = {
	"1": "",
	"2": "ˊ",
	"3": "ˇ",
	"4": "ˋ",
	"5": "˙",
}

INITIALS = {
	"b": "ㄅ",
	"p": "ㄆ",
	"m": "ㄇ",
	"f": "ㄈ",
	"d": "ㄉ",
	"t": "ㄊ",
	"n": "ㄋ",
	"l": "ㄌ",
	"g": "ㄍ",
	"k": "ㄎ",
	"h": "ㄏ",
	"j": "ㄐ",
	"q": "ㄑ",
	"x": "ㄒ",
	"zh": "ㄓ",
	"ch": "ㄔ",
	"sh": "ㄕ",
	"r": "ㄖ",
	"z": "ㄗ",
	"c": "ㄘ",
	"s": "ㄙ",
}
INITIAL_ORDER = sorted(INITIALS, key=len, reverse=True)
APICAL_INITIALS = {"zh", "ch", "sh", "r", "z", "c", "s"}
UMLAUT_INITIALS = {"j", "q", "x"}

FINALS = {
	"": "",
	"a": "ㄚ",
	"o": "ㄛ",
	"e": "ㄜ",
	"eh": "ㄝ",
	"ai": "ㄞ",
	"ei": "ㄟ",
	"ao": "ㄠ",
	"ou": "ㄡ",
	"an": "ㄢ",
	"en": "ㄣ",
	"ang": "ㄤ",
	"eng": "ㄥ",
	"er": "ㄦ",
	"i": "ㄧ",
	"ia": "ㄧㄚ",
	"io": "ㄧㄛ",
	"ie": "ㄧㄝ",
	"iao": "ㄧㄠ",
	"iu": "ㄧㄡ",
	"iou": "ㄧㄡ",
	"ian": "ㄧㄢ",
	"in": "ㄧㄣ",
	"iang": "ㄧㄤ",
	"ing": "ㄧㄥ",
	"u": "ㄨ",
	"ua": "ㄨㄚ",
	"uo": "ㄨㄛ",
	"uai": "ㄨㄞ",
	"ui": "ㄨㄟ",
	"uei": "ㄨㄟ",
	"uan": "ㄨㄢ",
	"un": "ㄨㄣ",
	"uen": "ㄨㄣ",
	"uang": "ㄨㄤ",
	"ong": "ㄨㄥ",
	"ueng": "ㄨㄥ",
	"v": "ㄩ",
	"ve": "ㄩㄝ",
	"van": "ㄩㄢ",
	"vn": "ㄩㄣ",
	"iong": "ㄩㄥ",
}

SYLLABIC_CONSONANTS = {
	"m": "ㄇ",
	"n": "ㄋ",
	"hm": "ㄏㄇ",
}

CHECK_CASES = {
	"中": ("zhong1", "ㄓㄨㄥ"),
	"汉": ("han4", "ㄏㄢˋ"),
	"一": ("yi1", "ㄧ"),
	"五": ("wu3", "ㄨˇ"),
	"鱼": ("yu2", "ㄩˊ"),
	"儿": ("er2", "ㄦˊ"),
}


def zero_initial_final(base: str) -> str:
	if base == "yong":
		return "iong"
	if base.startswith("yu"):
		rest = base[2:]
		if rest == "":
			return "v"
		if rest == "e":
			return "ve"
		if rest == "an":
			return "van"
		if rest == "n":
			return "vn"
	if base.startswith("y"):
		rest = base[1:]
		if rest in {"i", "in", "ing"}:
			return rest
		if rest == "ou":
			return "iu"
		if rest == "ong":
			return "iong"
		return "i" + rest
	if base.startswith("w"):
		rest = base[1:]
		if rest == "u":
			return "u"
		if rest == "o":
			return "uo"
		if rest == "ei":
			return "ui"
		if rest == "en":
			return "un"
		if rest in {"eng", "ong"}:
			return "ong"
		return "u" + rest
	return base


def split_initial(base: str) -> tuple[str, str]:
	for initial in INITIAL_ORDER:
		if base.startswith(initial):
			return initial, base[len(initial) :]
	return "", zero_initial_final(base)


def pinyin_to_zhuyin(syllable: str) -> str | None:
	if not syllable or syllable[-1] not in TONE_MARKS:
		return None

	base = syllable[:-1].replace("u:", "v").replace("ü", "v")
	tone_mark = TONE_MARKS[syllable[-1]]

	if base in SYLLABIC_CONSONANTS:
		return SYLLABIC_CONSONANTS[base] + tone_mark

	initial, final = split_initial(base)
	initial_symbol = INITIALS.get(initial, "")

	if initial in APICAL_INITIALS and final == "i":
		final = ""
	elif initial in UMLAUT_INITIALS and final.startswith("u"):
		final = "v" + final[1:]

	final_symbol = FINALS.get(final)
	if final_symbol is None:
		return None
	return initial_symbol + final_symbol + tone_mark


def primary_pinyin(row: dict[str, str]) -> str:
	return row["pinyin"].split("|", 1)[0]


def convert_pinyin_csv(src: Path, dst: Path) -> int:
	failures = 0
	with src.open("r", encoding="utf-8", newline="") as fin, dst.open(
		"w", encoding="utf-8", newline=""
	) as fout:
		reader = csv.DictReader(fin)
		writer = csv.DictWriter(fout, fieldnames=["codepoint", "zhuyin", "char"], lineterminator="\n")
		writer.writeheader()
		for row in reader:
			pinyin = primary_pinyin(row)
			zhuyin = pinyin_to_zhuyin(pinyin)
			if zhuyin is None:
				failures += 1
				print(
					f"unmatched pinyin {pinyin!r} for {row['codepoint']} {row['char']}",
					file=sys.stderr,
				)
				continue
			writer.writerow({"codepoint": row["codepoint"], "zhuyin": zhuyin, "char": row["char"]})
	return failures


def run_checks(src: Path) -> int:
	failures = 0
	for char, (pinyin, expected) in CHECK_CASES.items():
		actual = pinyin_to_zhuyin(pinyin)
		if actual != expected:
			failures += 1
			print(
				f"check failed for {char} ({pinyin}): expected {expected}, got {actual}",
				file=sys.stderr,
			)

	with src.open("r", encoding="utf-8", newline="") as fin:
		for row in csv.DictReader(fin):
			pinyin = primary_pinyin(row)
			if pinyin_to_zhuyin(pinyin) is None:
				failures += 1
				print(
					f"unmatched pinyin {pinyin!r} for {row['codepoint']} {row['char']}",
					file=sys.stderr,
				)
	return failures


def parse_args() -> argparse.Namespace:
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--src", type=Path, default=SRC_PINYIN_CSV, help="input pinyin CSV")
	parser.add_argument("--dst", type=Path, default=DST_ZHUYIN_CSV, help="output zhuyin CSV")
	parser.add_argument("--check", action="store_true", help="run converter self-checks and exit")
	return parser.parse_args()


def main() -> int:
	args = parse_args()
	if args.check:
		failures = run_checks(args.src)
		if failures:
			print(f"{failures} zhuyin conversion check(s) failed", file=sys.stderr)
			return 1
		print("zhuyin conversion checks passed")
		return 0

	args.dst.parent.mkdir(parents=True, exist_ok=True)
	failures = convert_pinyin_csv(args.src, args.dst)
	if failures:
		print(f"skipped {failures} row(s) with unmatched pinyin", file=sys.stderr)
	print(f"Converted zhuyin data from {args.src} to {args.dst}.")
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
