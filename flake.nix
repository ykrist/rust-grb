{
  inputs = {
    nixpkgsGurobi95.url = "github:NixOS/nixpkgs/817a592c923636e0f5c3549d393c603fec1c4db7";
    nixpkgsGurobi110.url = "github:NixOS/nixpkgs/release-24.11";
    nixpkgsGurobi120.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgsGurobi95,
      nixpkgsGurobi110,
      nixpkgsGurobi120,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs =
          nixpkgs:
          import nixpkgs {
            inherit system;
            config.allowUnfree = true; # gurobi has unfree license
          };

        # a helper function to create devshells with different versions of Gurobi installed
        mkGurobiDevShell =
          nixpkgs: version:
          let
            # create a `pkgs` from the given Nixpkgs revision
            pkgs' = pkgs nixpkgs;
          in
          pkgs'.mkShell {
            packages = with pkgs'; [
              gcc
              # add the corresponding Gurobi install
              gurobi
            ];
            # set to point to the corresponding Gurobi install's path
            GUROBI_HOME = pkgs'.gurobi;
            # .. and the corresponding license file
            GRB_LICENSE_FILE = "./lib/gurobi${version}.lic";
          };
      in
      {
        # This flake provides devshells for multiple versions of Gurobi.
        # The setup is pretty automated -- the only manual adjustment needed is to copy the license file to the right location.
        # E.g. the license for Gurobi 9.5 should go in `./lib/gurobi95.lic` (the file name is important!)

        devShells = rec {
          # this is the devshell loaded by default (also e.g. by nix-direnv)
          default = v110;

          # to switch to a particular version, use `nix devshell`. For example, to switch to Gurobi 9.5 devshell:
          # ```sh
          # nix develop .#v95
          # ```
          v95 = mkGurobiDevShell nixpkgsGurobi95 "95";
          v110 = mkGurobiDevShell nixpkgsGurobi110 "110";
          v120 = mkGurobiDevShell nixpkgsGurobi120 "120";
        };
      }
    );
}
