{pkgs, ...}: {
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
    settings.shared_preload_libraries = "timescaledb";
  };
}
