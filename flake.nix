{
  description = "Pinyin Sort CLI Tool with Sparse Checkout";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = inputs@{ flake-parts, nixpkgs, flake-utils, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ ];

      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem = { pkgs, system, ... }: {
        devShells.default = pkgs.mkShell {
          name = "pinyin-sort-dev";
          buildInputs = [
            pkgs.rustup    # or pkgs.rustc + pkgs.cargo if you prefer static
            pkgs.cargo
            pkgs.git
            pkgs.just
            pkgs.bash
            pkgs.curl
            pkgs.python314
            pkgs.python314Packages.pypinyin
          ];

          shellHook = ''
            echo "[*] Welcome to the pinyin-sort dev shell."
            export RUST_BACKTRACE=1
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage  {
            pname = "pinyin-sort";
            version = "0.1.0";

            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            # 若有多个二进制或复杂构建需求，也可配置 override
            # postInstall = "mv $out/bin/xxx $out/bin/pinyin-sort";
            };
      };

      flake = { };
    };
}