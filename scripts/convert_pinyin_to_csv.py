# scripts/convert_pinyin_to_csv.py

import sys
from pathlib import Path

from pypinyin.contrib.tone_convert import to_tone3

PROJECT_ROOT = Path(__file__).resolve().parent.parent

SRC_PINYIN_DATA = PROJECT_ROOT / "vendor" / "pinyin-data" / "pinyin.txt"
DST_PINYIN_CSV = PROJECT_ROOT / "data" / "pinyin.csv"


def normalize_pinyin(pinyin: str) -> str:
	return to_tone3(pinyin.strip())


def convert_pinyin_to_csv(src: Path, dst: Path) -> None:
	"""
	Convert pinyin data from a tab-separated format to a CSV format.
	The input file should have lines in the format: "character\tpinyin".
	The output file will have lines in the format: "character,pinyin".
	"""
	if not src.exists():
		print(f"Source file {src} does not exist.")
		sys.exit(1)

	if not dst.parent.exists():
		dst.parent.mkdir(parents=True, exist_ok=True)

	with open(src, "r", encoding="utf-8") as fin, open(dst, "w", encoding="utf-8") as fout:
		fout.write("codepoint,pinyin,char\n")
		for line in fin.readlines():
			if line and not line.startswith("#"):
				try:
					codepoint, rest = map(str.strip, line.strip().split(":"))
					pinyins, char = map(str.strip, rest.strip().split("#"))
					pinyins = "|".join(map(normalize_pinyin, pinyins.split(",")))
					fout.write(f"{codepoint},{pinyins},{char}\n")
				except:
					print(f"Error parsing line: {line}")
					raise


if __name__ == "__main__":
	convert_pinyin_to_csv(SRC_PINYIN_DATA, DST_PINYIN_CSV)
	print(f"Converted pinyin data from {SRC_PINYIN_DATA} to {DST_PINYIN_CSV}.")
