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
                authentication = pkgs.lib.mkAfter ''
                  host all all all scram-sha-256
                '';
              };

              systemd.services.set-encrypted-secret = {
                enable = true;
                wantedBy = ["default.target"];
                before = ["postgresql.service"];
                serviceConfig = {
                  Type = "oneshot";
                };
                script = ''
                  mkdir -p /usr/lib/credstore.encrypted
                  echo -n "haha123" | systemd-creds encrypt - /usr/lib/credstore.encrypted/postgres-password
                '';
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

                [[users]]
                name = "user1"
                systemd_password_credential = "postgres-password"

                [[database_permissions]]
                role = "user1"
                permissions = ["CONNECT"]
                databases = ["db1"]
              '';

              systemd.services.impostare = {
                enable = true;
                wantedBy = ["default.target"];
                requires = ["postgresql.service"];
                after = ["postgresql.service"];
                serviceConfig = {
                  Type = "oneshot";
                  User = "postgres";
                  LoadCredentialEncrypted = "postgres-password";
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
              machine.wait_for_file("/usr/lib/credstore.encrypted/postgres-password")
              machine.wait_for_open_port(5432)

              # Make sure the databases exist
              machine.succeed("psql -lqt | grep db1")
              machine.succeed("psql -lqt | grep db2")

              # Try to connect with the set password
              machine.succeed("psql postgresql://user1:haha123@localhost/db1 -c 'SELECT 1;'")
            '';
          };
        };
      };
      flake = {};
    };
}
