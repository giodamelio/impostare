{pkgs, lib, ...}: {
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
    ]);
}
