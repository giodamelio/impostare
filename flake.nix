{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, devenv, systems, ... } @ inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      packages = forEachSystem (system: {
        devenv-up = self.devShells.${system}.default.config.procfileScript;
      });

      devShells = forEachSystem
        (system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
            lib = pkgs.lib;
          in
          {
            default = devenv.lib.mkShell {
              inherit inputs pkgs;
              modules = [
                {
                  languages.nix.enable = true;
                  languages.rust.enable = true;

                  pre-commit.hooks.cargo-check.enable = true;
                  pre-commit.hooks.clippy.enable = true;
                  pre-commit.hooks.rustfmt.enable = true;

                  difftastic.enable = true;

                  enterTest = ''
                    cargo test
                  '';

                  services.postgres = {
                    enable = true;
                    package = pkgs.postgresql_15;
                    extensions = extensions: [
                      extensions.timescaledb
                    ];
                    initialScript = ''
                      CREATE USER postgres SUPERUSER;
                    '';
                    settings.shared_preload_libraries = "timescaledb";
                  };

                  packages = lib.optionals
                    pkgs.stdenv.isDarwin
                    (with pkgs.darwin.apple_sdk; [
                      frameworks.CoreFoundation
                      frameworks.Security
                      frameworks.SystemConfiguration
                    ]);
                }
              ];
            };
          });
    };
}
