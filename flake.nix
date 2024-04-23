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
        packages.impostare = pkgs.rustPlatform.buildRustPackage {
          pname = "impostare";
          version = "0.1.0";

          src = ./.;
          cargoHash = "sha256-+GCZM0YoRM+0XqlX9cZwY1LjWTBWD2zW4AQgst86jck=";
        };
        checks = {
          helloWorld = pkgs.nixosTest {
            name = "hello-world";

            nodes.machine = { config, pkgs, ... }: {
              services.postgresql = {
                enable = true;
              };

              environment.etc.connection-details.text = ''
                host=/run/postgresql user=postgres
              '';

              environment.etc."database-settings.toml".text = ''
                [[databases]]
                name = "db1"

                [[databases]]
                name = "db2"

                [[users]]
                name = "root"
              '';

              systemd.services.impostare = {
                enable = true;
                wantedBy = ["default.target"];
                requires = ["postgresql.service"];
                after = ["postgresql.service"];
                serviceConfig = {
                  Type = "oneshot";
                  User = "postgres";
                };
                script = ''
                  ${self'.packages.impostare}/bin/impostare \
                    /etc/connection-details \
                    /etc/database-settings.toml
                '';
              };

              system.stateVersion = "23.11";
            };

            testScript = ''
              machine.wait_for_unit("default.target")
              machine.wait_for_open_port(5432)

              # Make sure the databases exist
              machine.succeed("psql -lqt | grep db1")
              machine.succeed("psql -lqt | grep db2")
            '';
          };
        };
      };
      flake = {};
    };
}
