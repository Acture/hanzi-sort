dev-shell:
	nix develop

build:
	echo "Building project..." && nix build

prep-data:
	echo "Preparing data..." && nix develop --command python3 scripts/convert_pinyin_to_csv.py
