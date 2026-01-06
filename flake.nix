{
  description = "Developer shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit overlays system; };
        python = pkgs.python313;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            python
            rust-bin.stable.latest.minimal
            uv
          ];

          shellHook = ''
            if ! [ -d './.venv' ]
            then uv venv -p '${python}/bin/python'
            fi

            unset VIRTUAL_ENV
            . .venv/bin/activate
          '';
        };
      }
    );
}
