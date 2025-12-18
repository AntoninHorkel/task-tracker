{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils = {
      url = "github:numtide/flake-utils";
      # inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    # TODO: Specify systems.
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.buildNpmPackage {
          pname = "task-tracker-frontend";
          version = "0.1.0";
          src = ./.;
          # npmDepsHash = pkgs.lib.fakeHash;
          npmDepsHash = "sha256-Cmbwg8I6ykJGfLqJ+1WcPWbx9nrxZJ2ZOupJ06O/Qvg=";
          buildInputs = [ pkgs.python3 ];
          installPhase = ''
            mkdir -p $out/bin
            cp -r dist/* $out/bin
            echo -e "!#/bin/sh\npython3 -m http.server" > $out/bin/task-tracker-backend
            chmod +x $out/bin/task-tracker-backend
          '';
        };
        devShells.default = pkgs.mkShellNoCC {
          packages = with pkgs; [
            nodejs
            nodePackages.npm
          ];
        };
        formatter = pkgs.nixfmt-tree;
      }
    );
}
