{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };
  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [];
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { config, self', inputs', pkgs, system, ... }: {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "impostare";
          version = "0.1.0";

          src = ./.;
          cargoHash = "sha256-+GCZM0YoRM+0XqlX9cZwY1LjWTBWD2zW4AQgst86jck=";
        };
        checks = {
          helloWorld = pkgs.nixosTest {
            name = "hello-world";

            nodes.machine = { config, pkgs, ... }: {
              system.stateVersion = "23.11";
            };

            testScript = ''
              machine.wait_for_unit("default.target")
            '';
          };
        };
      };
      flake = {};
    };
}
