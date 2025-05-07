# Git Project Updater

<!--toc:start-->

- [Git Project Updater](#git-project-updater)
  - [Overview](#overview)
  - [Features](#features)
  - [File Structure](#file-structure)
  - [Setup](#setup)
  - [Usage](#usage)
  - [Nix](#nix)
  - [Contributing](#contributing)
  - [License](#license)

## Overview

Git Project Updater is a Rust-based application designed to streamline the process of managing multiple Git repositories. It provides functionalities for cloning repositories, checking out branches, and pulling updates efficiently.

## Features

- Clone Git repositories.
- Checkout specific branches.
- Pull updates from remote repositories.
- Restore original branches after updates.

## File Structure

```text
git-project-updater
├── src
│   ├── main.rs           # Entry point, CLI handling, main loop orchestration
│   ├── config.rs         # Configuration structs, loading, and validation
│   ├── git_utils.rs      # All git-related operations
│   ├── project_logic.rs  # Core logic for processing a single project
│   ├── logger.rs         # Logging enum and function
│   └── error.rs          # Custom error types
├── flake.nix             # Nix flake for reproducible builds.
└── README.md             # Documentation for the project.
```

## Setup

To set up the project, ensure you have [Nix](https://nixos.org/download.html) installed. You can then use the following commands to enter the development environment:
The project uses [Devenv](https://devenv.sh/) together with [direnv](https://direnv.net/) to manage the development environment.

## Usage

To run the application, execute the following command:

```bash
cargo run -- ./projects.json
```

Make sure to configure your `ProjectConfig` with the necessary parameters before running the application.

**Config file format example:**

```json
{
  "global_config": {
    "default_clone_parent_directory": "~/projects/work"
  },
  "projects": [
    {
      "project": "MyCoolApp",
      "url": "https://github.com/user/mycoolapp.git",
      "path": "mycoolapp",
      "pull_branches": ["main", "develop"]
    },
    {
      "project": "AnotherProject",
      "url": "https://github.com/user/anotherproject.git",
      "path": "/absolute/path/to/anotherproject"
    },
    {
      "project": "LegacySystem",
      "url": "https://github.com/user/legacysystem.git",
      "path": "old_stuff/legacy",
      "pull_branches": []
    }
  ]
}
```

## Nix

Run using `nix run`

```bash
nix run github:/nico-swan-com/git-project-updater -- ./projects.json

```

Add to your `flake.nix`

```nix
# In your nix-config/flake.nix
inputs = {
  nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  home-manager.url = "github:nix-community/home-manager";
  # ... other inputs ...

  git-project-updater.url = "github:/nico-swan-com/git-project-updater";
  git-project-updater.inputs.nixpkgs.follows = "nixpkgs";
};

outputs = { self, nixpkgs, home-manager, git-project-updater, ... }@inputs: {
  nixosConfigurations.yourHostname = nixpkgs.lib.nixosSystem {
    system = "x86_64-linux"; 
    specialArgs = { inherit inputs; };
    modules = [
      ./configuration.nix
      # ... other modules
    ];
  };

```

```nix
# In your configuration.nix or a similar file
{ pkgs, inputs, ... }: # Make sure 'inputs' is available via specialArgs
let
  gitProjectUpdaterPkg = inputs.k8sServiceScriptFlake.packages.${pkgs.system}.default;
in
{
  environment.systemPackages = with pkgs; [
    gitProjectUpdaterPkg
  ];

  # Option 2 (Alternative): Use an overlay if you want to refer to it by a simpler name
  # nixpkgs.overlays = [
  #   (final: prev: {
  #     git-project-updater = gitProjectUpdaterPkg;
  #   })
  # ];
  # And then in environment.systemPackages:
  # environment.systemPackages = with pkgs; [
  #   git-project-updater
  # ];
}
```

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue for any enhancements or bug fixes.

## License

This project is licensed under the MIT License. See the LICENSE file for more details.
