{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-parts,
    fenix,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux"];

      perSystem = {system, ...}: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [fenix.overlays.default];
        };
        lib = pkgs.lib;
        craneLib = (crane.mkLib pkgs).overrideToolchain (fenix.packages.${system}.fromToolchainFile {
          dir = ./.;
          sha256 = "sha256-VZZnlyP69+Y3crrLHQyJirqlHrTtGTsyiSnZB8jEvVo=";
        });

        bot-crate = craneLib.buildPackage {
          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type: (lib.strings.hasSuffix ".md" path) || (craneLib.filterCargoSources path type);
            name = "source";
          };

          buildInputs =
            [
              pkgs.openssl
              pkgs.pkg-config
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
        };
      in {
        formatter = pkgs.alejandra;
        packages.default = bot-crate;

        devShells.default = craneLib.devShell {
          inputsFrom = [bot-crate];

          packages = [
            pkgs.rust-analyzer-nightly
          ];
        };
      };
    };
}
