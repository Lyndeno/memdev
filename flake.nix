{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    crane.url = "github:ipetkov/crane";

    pre-commit-hooks-nix = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    crane,
    pre-commit-hooks-nix,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages."${system}";
      craneLib = crane.mkLib pkgs;
      lib = pkgs.lib;

      jsonFilter = path: _type: builtins.match ".*json$" path != null;
      jsonOrCargo = path: type:
        (jsonFilter path type) || (craneLib.filterCargoSources path type);

      common-args = {
        src = lib.cleanSourceWith {
          src = ./.;
          filter = jsonOrCargo;
          name = "source";
        };
        strictDeps = true;

        buildInputs = [pkgs.udev];
        nativeBuildInputs = [pkgs.installShellFiles pkgs.pkg-config];

        postInstall = ''
          installShellCompletion --cmd memdev \
            --bash ./target/release/build/memdev-*/out/memdev.bash \
            --fish ./target/release/build/memdev-*/out/memdev.fish \
            --zsh ./target/release/build/memdev-*/out/_memdev
          installManPage ./target/release/build/memdev-*/out/memdev.1
        '';
      };

      memdev = craneLib.buildPackage (common-args
        // {
          cargoArtifacts = craneLib.buildDepsOnly common-args;
        });
    in rec {
      checks = {
        inherit memdev;
      };
      packages.memdev = memdev;
      packages.default = packages.memdev;

      apps.memdev = utils.lib.mkApp {
        drv = packages.memdev;
      };
      apps.default = apps.memdev;

      formatter = pkgs.alejandra;

      devShells.default = let
        pre-commit-format = pre-commit-hooks-nix.lib.${system}.run {
          src = ./.;

          hooks = {
            alejandra.enable = true;
            rustfmt.enable = true;
            clippy.enable = true;
          };
        };
      in
        craneLib.devShell {
          packages = with pkgs; [
            rustfmt
            clippy
            cargo-deny
            cargo-about
            termshot
            pkg-config
            udev
          ];
          shellHook = ''
            ${pre-commit-format.shellHook}
          '';
        };
    });
}
