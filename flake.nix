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
        devShells = rec {
          default = v110;
          v95 = mkGurobiDevShell nixpkgsGurobi95 "95";
          v110 = mkGurobiDevShell nixpkgsGurobi110 "110";
          v120 = mkGurobiDevShell nixpkgsGurobi120 "120";
        };
      }
    );
}
