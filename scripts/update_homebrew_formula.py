#!/usr/bin/env python3

from __future__ import annotations

import argparse
from pathlib import Path


FORMULA_TEMPLATE = """class HanziSort < Formula
  desc "Sort Chinese text by pinyin or stroke count"
  homepage "https://github.com/Acture/hanzi-sort"
  url "https://github.com/Acture/hanzi-sort/archive/refs/tags/v{version}.tar.gz"
  sha256 "{sha256}"
  license "AGPL-3.0-only"

  livecheck do
    url :stable
    regex(/^v?(\\d+(?:\\.\\d+)+)$/i)
  end

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    output = shell_output("#{{bin}}/hanzi-sort -t 张三 李四 王五")

    assert_match "李四", output
    assert_match "王五", output
    assert_match "张三", output
    assert_operator output.index("李四"), :<, output.index("王五")
    assert_operator output.index("王五"), :<, output.index("张三")
  end
end
"""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Write the Homebrew formula for hanzi-sort."
    )
    parser.add_argument("--version", required=True, help="Release version without leading v.")
    parser.add_argument("--sha256", required=True, help="SHA-256 of the release source tarball.")
    parser.add_argument(
        "--output",
        required=True,
        type=Path,
        help="Target Formula/hanzi-sort.rb path.",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    args.output.write_text(
        FORMULA_TEMPLATE.format(version=args.version, sha256=args.sha256),
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
