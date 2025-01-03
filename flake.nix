{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/release-24.11"; # to not have unstable's Gurobi 12
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs =
    { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true; # gurobi has unfree licence
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            gcc # for the linker
            gurobi
          ];
          GUROBI_HOME = pkgs.gurobi;
        };
      }
    );
}
