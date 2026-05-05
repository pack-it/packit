# Packit

Packit is a universal package manager, designed to streamline the experience of installing packages on your system.

Please note Packit is still in early development, breaking changes are possible in future versions.

## Install
Packit can be installed by simply copying one of the commands below in your terminal.

After install you need to make sure you add the Packit prefix directory to your PATH, to ensure the `pit` command is available in your terminal. The command for this is shown after the install script finished installing.

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


## License
The Packit repository is licensed under the GNU General Public License v3.0. See [LICENSE](LICENSE) for the full license.


## Usage
The general usage of Packit is: `pit <COMMAND>`.

#### `pit install <PACKAGE-NAME>[@<VERSION>] [--build] [--build-all] [--keep-build] [--skip-symlinking] [--skip-active] [--verbose]`
Installs the specified packages, if a version is given that version will be installed, if not the latest available version will be installed. Multiple packages can be specified by entering multiple names, split by a space.
<br>
If the `--build` option is given, the package is build from source, instead of installing a prebuild version.
If the `--build-all` option is given, the package and all its dependencies are build from source, instead of installing prebuild versions.
If the `--keep-build` option is given, the build dependencies will not be deleted after building.
If the `--skip-symlinking` option is enabled, the package is not symlinked into the /bin, /lib, /share, etc. directories.
If the `--skip-active` option is enabled, the package is not set to active and the current active version is kept. If there is no current active version, this flag is ignored and the package is set to active.
If the `--verbose` option is given, extra verbose output is shown, like build output.

#### `pit uninstall <PACKAGE-NAME>[@<VERSION>]`
Uninstalls the specified packages, if a version is given that version will be uninstalled, if not, you will be asked if you want to delete all versions of `<PACKAGE-NAME>` in case there are multiple versions installed. Multiple packages can be specified by entering multiple names, split by a space.

#### `pit list`
Lists all the installed packages.

#### `pit search <PACKAGE-NAME>[@<VERSION>]`
Searches a package with `<PACKAGE-NAME>` and shows information based on the package metadata. If the version is given that specific version is searched for.

#### `pit update <PACKAGE-NAME>[@<VERSION>] [<NEW-VERSION>]`
Updates the specified package to the new version, or the latest version if no new version is specified. If multiple versions of the same package are installed, the `<VERSION>` option is required.

#### `pit info <PACKAGE-NAME>[@<VERSION>] [-v] [--tree]`
Shows info about the specified installed package. If the `-v` option is given, extra information is shown. If the `--tree` option is enabled, the whole dependency tree is shown.

#### `pit check [<PACKAGE-NAME>@<VERSION> ...]`
Checks the Packit installation for issues. When package name(s) and version(s) are given, only those package(s) are checked for issues. 

#### `pit fix`
Fix all issues found by the check command. You will be asked if you want to fix an issue for each issue type. When package name(s) and version(s) are given, only those package(s) are checked and fixed. 

#### `pit switch <PACKAGE-NAME> <VERSION> [--skip-symlinking]`
Switches the active version of the specified package to the specified version. If the `--skip-symlinking` option is given, the new active version is not symlinked into the /bin, /lib, /share, etc. directories.

#### `pit link <PACKAGE-NAME> [--force]`
Links the specified package into the /bin, /lib, /share, etc. directories. If the package metadata does not allow a package to be symlinked, the `--force` option is required to force the symlinking of the package. Please be careful with using the `--force` option, since there is most likely a good reason to skip symlinking.

#### `pit unlink <PACKAGE-NAME>`
Unlinks the specified package, causing the package to be unavailable from the `PATH` environment variable.

#### `pit package <PACKAGE-NAME>@<VERSION> <DESTINATION>`
Packages the specified package into a prebuild and stores it in the destination directory, together with a checksum of the prebuild.

#### `pit util checksum <URL>`
Calculates the checksum of the file at the given url.

#### `pit config show`
Shows the current configuration.

#### `pit config set-prefix <NEW-PREFIX>`
Sets the prefix to the given directory. Currently not supported when there are already installed packages.

#### `pit config set-multiuser <MULTIUSER>`
Sets the multiuser setting to true or false. Currently not supported when there are already installed packages.

#### `pit config repositories list`
Lists all configured repositories.

#### `pit config repositories set-rank <REPOSITORY-ID>`
Sets the repositories rank in the config. Multiple `<REPOSITORY-ID>` can be given for multiple repositories in the rank.

#### `pit config repositories add <ID> <URL> [PROVIDER]`
Adds a new repository to the config. Also adds the new repository to the back of the repositories rank.


## Config
All available fields in the config are listed below. The [`pit config`](#pit-config-show) command can also be used to change the config.

| Field               | Explanation                                                                                                                  |
|---------------------|------------------------------------------------------------------------------------------------------------------------------|
| `prefix_directory`  | Defines the directory used for installing packages, see [File structure](#file-structure) for the defaults on each platform. |
| `repositories_rank` | Defines the order of repositories to search for a package.                                                                   |
| `multiuser`         | True to run Packit in multiuser mode, false for single user mode.                                                            |

### Repositories

| Field                | Explanation                                                              |
|----------------------|--------------------------------------------------------------------------|
| `path`               | Defines the path to the repository.                                      |
| `provider`           | Defines the provider of the repository, defaults to `web`.               |
| `prebuilds_url`      | Defines the url of the prebuilds repository for this package repository. |
| `prebuilds_provider` | Defines the provider of the prebuilds repository, defaults to `fs`.      |

Specifying a prebuild repository is optional and overrides the value specified in the repository metadata.


### Example config

```
repositories_rank = ["core"]

[repositories]

[repositories.core]
provider = "web"
path = "https://raw.githubusercontent.com/pack-it/core/main/"
```


## File structure
You might be interested in how (and why) Packit manages dependencies, configs and most importantly the installs. We explain that here, ofcourse this differs a bit for each platform as they have different file structures. Luckly Packit manages this for you!

### Files and directories

#### Prefix
The prefix directory of Packit contains all data of installed packages.
On Unix systems we use `/opt/packit`, on Windows we use `C:\Program Files\packit`

#### Package install files
All installed packages will go in `<prefix>/packages/<PACKAGE-NAME>/<PACKAGE-VERSION>/`.

#### Installed.toml
The `Installed.toml` file is located inside the prefix and stores information about all installed packages. This file is managed by Packit and should not be changed directly.

#### Active packages
The currently active version of a package will be symlinked in `<prefix>/active/<PACKAGE-NAME>`. This will link to `<prefix>/packages/<PACKAGE-NAME>/<ACTIVE-PACKAGE-VERSION>`

#### Symlinks
The active binaries will be symlinked in: `<prefix>/bin/<EXECUTABLE-NAME>`. This directory needs to be present in the users `PATH` in order for installed binaries to be detected by the system.

#### Packit configs
On Linux we use `/etc/packit` for the configs, on macOS we use `/Library/Application Support/packit` and on Windows we use `C:\Program Files\packit`.
Currently the only config is `Config.toml`, containing all configured repositories.


## Documentation

See [the docs directory](docs/README.md) for more detailed documentation of Packit.
