{ pkgs, lib, config, inputs, ... }: {
  # See full reference at https://devenv.sh/reference/options/

  packages = with pkgs; [ git cargo-watch ];

  languages.rust.enable = true;

  processes.cargo-watch.exec = "cargo-watch";

  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
  '';

  git-hooks.hooks = { shellcheck.enable = false; };
}
