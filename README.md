# Packit

Packit is a universal package manager, designed to streamline the experience of installing packages on your system.

Please note Packit is still in early development, breaking changes are possible in future versions.

### Why use Packit?
You might be asking yourself: why use this new package manager when I already have one? 
As mentioned before, Packit is a universal package manager that works on macOS, Linux and native Windows (no subsystem for Linux required!). Its interface and usage remain consistent across all platforms, making it ideal for developers who frequently switch between operating systems. In addition to being a universal package manager Packit also offers some unique features:
- **Active versions**, which allows multiple package versions to be installed next to each other without conflict.
- **Flexible dependency resolution**, each package dependency can be satisfied with any satisfying package, instead of requiring a single fixed version.
- **(Un)linked packages**, which allows a package to be installed, but not in the `PATH` to avoid conflicts with existing packages when necessary.
- **Portable repositories** which can be used to create repositories for offline or air-gapped systems.

## Install
Packit can be installed by simply copying one of the commands below in your terminal.

In most cases Packit can add its [prefix directory](./docs/structure.md#prefix) to the `PATH` automatically. Only a restart of the shell will be required. If you're on Unix and the shell is not one of: `bash`, `zsh` or `fish`, this doesn't happen and you will need to add Packit to your `PATH` manually.

### Unix
```
curl -fsSL https://raw.githubusercontent.com/pack-it/packit/main/install.sh | sh
```

### Windows
```
curl -fsSL https://raw.githubusercontent.com/pack-it/packit/main/install.bat --output packit-install.bat && call packit-install.bat
```

### Building from source
You can also build Packit from source locally, by simply using Cargo. Please note that Rust needs link.exe on Windows, which is part of the Visual C++ toolchain.

1. First download the provided source code or clone the Git repository.
2. Open the terminal inside the source folder and run `cargo build-install`. Use `cargo build-install --destination=<DESTINATION>` to use a different destination than the default.
3. After building and installing, there will be a `target/build` directory (or the destination you specified in the command) which contains the `bin` directory, containing the `packit` binary (`packit.exe` on Windows).

If you only need the `packit` binary itself, you could build it directly using `cargo build`, or `cargo build --release` for a release build. This will result in the `packit` binary (`packit.exe` on Windows) which will be located at `target/debug/packit` or at `target/release/packit` for a release build.

To install and initialize your locally built version of Packit, you need to:
1. Move the build destination directory (`target/build`) to `<prefix>/packages/packit/<version>`.
2. Run `<prefix>/packages/packit/<version>/packit init`. If you used another prefix than the [default prefix](./docs/structure.md#prefix), you need to specify your prefix in the `--prefix` flag to this command.
3. Add `<prefix>/bin` to your `PATH`. The `pit` command should now be available and working. You can test this using `pit list`, this should only show Packit as installed package.

## Usage
The general usage of Packit is: `pit <COMMAND>`. Underneath is a quick overview of the most important Packit commands. A lot more commands and flags are available. They can be found in the [commands documentation](./docs/commands).

#### `pit install <PACKAGE-NAME>[@<VERSION>] ...` [🔗](./docs/commands/install.md)
Installs the specified packages, if a version is given that version will be installed, if not the latest available version will be installed. Multiple packages can be specified by passing multiple names.

#### `pit uninstall <PACKAGE-NAME>[@<VERSION>] ...` [🔗](./docs/commands/uninstall.md)
Uninstalls the specified packages, if a version is given that version will be uninstalled, if not, you will be asked if you want to delete all versions of `<PACKAGE-NAME>` in case there are multiple versions installed. Multiple packages can be specified by passing multiple names.

When uninstalling multiple packages at once, Packit will automatically determine the correct order. Dependencies are uninstalled after their dependents.

#### `pit update <PACKAGE-NAME>[@<VERSION>] ...` [🔗](./docs/commands/update.md)
Updates the specified packages to the latest version. If a version is specified, that version is updated. If no version is specified and only one version is installed that version is updated. If multiple versions are installed and no version is specified only the latest installed version is updated.

#### `pit list` [🔗](./docs/commands/list.md)
Lists all the installed packages.

#### `pit info [<PACKAGE-NAME>[@<VERSION>]]` [🔗](./docs/commands/info.md)
Shows info about the specified installed package. If the version is specified, version specific information is shown. If no arguments are given, information about the current Packit installation is shown.

#### `pit search <PACKAGE-NAME>[@<VERSION>]` [🔗](./docs/commands/search.md)
Searches for a package with `<PACKAGE-NAME>[@<VERSION>]` and information based on the package metadata is shown, if the version is given that specific version is searched for.

#### `pit switch <PACKAGE-NAME> <VERSION>` [🔗](./docs/commands/switch.md)
Switches the active version of the specified package to the specified version.

#### `pit switch-dependency <PACKAGE-NAME>@<VERSION> <DEPENDENCY-NAME> <NEW-DEPENDENCY-VERSION>` [🔗](./docs/commands/switch-dependency.md)
Switches the dependency (`<DEPENDENCY-NAME>`) version used by a package (`<PACKAGE-NAME>@<VERSION>`) to another version (`NEW-DEPENDENCY-VERSION`).

#### `pit link <PACKAGE-NAME>` [🔗](./docs/commands/link.md)
Symlinks the specified package in the prefix directory. To learn more about Packit symlinks, take a look at [symlinks](./docs/structure.md#symlinks).

#### `pit unlink <PACKAGE-NAME>` [🔗](./docs/commands/unlink.md)
Unlinks the specified package, causing the package to be unavailable from the `PATH` environment variable.

#### `pit check` [🔗](./docs/commands/check.md)
Checks the Packit installation for issues.

#### `pit fix` [🔗](./docs/commands/fix.md)
Fixes all issues found by the check command. You will be asked if you want to fix an issue for each issue type.

## Config
The `Config.toml` contains the Packit configuration, it can be edited manually or with the [`pit config`](./docs/commands/config.md) command. Its location differs for each platform:
| Platform | Location                              |
| ---------| ------------------------------------- |
| Linux    | `/etc/packit`                         |
| MacOS    | `/Library/Application Support/packit` |
| Windows  | `C:\Program Files\packit`             |

The `Config.toml` contains options such as the Packit prefix directory, multiuser mode, repository definitions and repository search order.

For a complete overview of all available configuration fields, their defaults, and an example configuration, see the [Config documentation](./docs/structure.md#config).

## File structure
You might be interested in where all your packages are installed to. Almost everything is stored in the Packit prefix directory. Of course this directory and structure differs a bit for each platform. Luckily Packit manages this for you!

| Platform | Directory                 |
| -------- | ------------------------- |
| Unix     | `/opt/packit`             |
| Windows  | `C:\Program Files\packit` |

For a complete overview of all files and their content, see the [file structure documentation](./docs/structure.md#file-structure).

## License
The Packit repository is licensed under the GNU General Public License v3.0. See [LICENSE](LICENSE) for the full license.

## Documentation
See [the docs directory](docs/README.md) for more detailed documentation of Packit.
