# Commands

This file contains all of Packits commands.

#### `pit install <PACKAGE-NAME>[@<VERSION>] ... [--build] [--build-all] [--keep-build] [--skip-symlinking] [--skip-active] [--verbose] [--skip-test] [--skip-build-test] [--pause-build]`
Installs the specified packages, if a version is given that version will be installed, if not the latest available version will be installed. Multiple packages can be specified by entering multiple names, split by a space.
<br>
If the `--build` option is given, the package is build from source, instead of installing a prebuild version.
If the `--build-all` option is given, the package and all its dependencies are build from source, instead of installing prebuild versions.
If the `--keep-build` option is given, the build dependencies will not be deleted after building.
If the `--skip-symlinking` option is enabled, the package is not symlinked into the /bin, /lib, /share, etc. directories.
If the `--skip-active` option is enabled, the package is not set to active and the current active version is kept. If there is no current active version, this flag is ignored and the package is set to active.
If the `--skip-test` option is enabled, Packit tests are skipped.
If the `--skip-build-test` option is enabled, build tests are skipped.
If the `--verbose` option is given, extra verbose output is shown, like build output.
If the `--pause-build` option is enabed, the build is paused after build script execution to debug builds.

#### `pit uninstall <PACKAGE-NAME>[@<VERSION>] ...`
Uninstalls the specified packages, if a version is given that version will be uninstalled, if not, you will be asked if you want to delete all versions of `<PACKAGE-NAME>` in case there are multiple versions installed. Multiple packages can be specified by entering multiple names, split by a space.

#### `pit list [--updatables] [--active]`
Lists all the installed packages. If the `--updatables` flag is specified, all updatable packages are listed. If the `--active` flag is specified, only the active package versions are listed.

#### `pit search <QUERY> [--regex] [--verbose] [--tree] [--latest] [--target-only]`
Searches a package with `<QUERY>`. If `--regex` is not enabled, the query is expected to be `<PACKAGE-NAME>[@<VERSION>]` and information based on the package metadata is shown, if the version is given that specific version is searched for. If `--regex` is given, all packages that match the given regular expression query are shown. The `--verbose` flag can be used to show more output. The `--tree` flag can be used to show the tree of a package. Note that the package version also needs to be given in this case and that the latest version is assumed for the dependencies. The `--latest` flag can be used to use the latest version of a specified package, instead of specifying a version. If the `--target-only` flag is enabled with the `--regex` flag, only the packages that are available for the current target are shown.

#### `pit update [<PACKAGE-NAME>[@<VERSION>] ...] [--new-version <NEW-VERSION>] [--all] [--exclude <PACKAGE-NAME> ...]`
Updates the specified package to the new version, or the latest version if no new version is specified. If multiple packages are specified they are all updated to the latest version (and `--new-version` cannot be used). If multiple versions of the same package are installed, the latest installed version is assumed. The `--new-version` flag can be used to specify the new version to install. The `--all` flag can be used to update all packages (the latest installed version will be updated). The `--exclude` flag can be used to exclude certain packages when using the `--all` flag.

#### `pit info [<PACKAGE-NAME>[@<VERSION>] [-v] [--tree] [--active]]`
Shows info about the specified installed package. If the `-v` option is given, extra information is shown. If the `--tree` option is enabled, the whole dependency tree is shown. If no arguments are given, information about the current Packit install is shown. The `--active` flag can be used to use the active version of a specified package, instead of specifying a version.

#### `pit check [<PACKAGE-NAME>[@<VERSION>] ...]`
Checks the Packit installation for issues. When package name(s) and version(s) are given, only those package(s) are checked for issues. 

#### `pit fix [<PACKAGE-NAME>[@<VERSION>] ...]`
Fix all issues found by the check command. You will be asked if you want to fix an issue for each issue type. When package name(s) and version(s) are given, only those package(s) are checked and fixed. 

#### `pit switch <PACKAGE-NAME> <VERSION> [--skip-symlinking]`
Switches the active version of the specified package to the specified version. If the `--skip-symlinking` option is given, the new active version is not symlinked into the /bin, /lib, /share, etc. directories.

#### `pit switch-dependency <PACKAGE-NAME>@<VERSION> <DEPENDENCY-NAME> <NEW-DEPENDENCY-VERSION>`
Switches the dependency version of the specified package to the specified version.

#### `pit link <PACKAGE-NAME> [--force] [--overwrite]`
Links the specified package into the /bin, /lib, /share, etc. directories. If the package metadata does not allow a package to be symlinked, the `--force` option is required to force the symlinking of the package. Please be careful with using the `--force` option, since there is most likely a good reason to skip symlinking. The `--overwrite` option can be used to overwrite existing symlinks from another package, please note that this should normally not be used, as conflicts between packages should be avoided.

#### `pit unlink <PACKAGE-NAME>`
Unlinks the specified package, causing the package to be unavailable from the `PATH` environment variable.

#### `pit util package <DESTINATION> <PACKAGE-NAME>@<VERSION> ... [--structured] [--all]`
Packages the specified package(s) into a prebuild and stores it in the destination directory, together with a checksum of the prebuild. When `--structured` is used the packages will be put into a prebuild directory structure. `--all` will package all installed packages.

#### `pit util checksum <URL>`
Calculates the checksum of the file at the given url. Also shows the size of the downloaded file in bytes.

#### `pit util portable-repo <DESTINATION> <PACKAGE-NAME>@<VERSION> ... [--exclude-prebuilds] [--skip-dependency-resolution]`
Generates a portable repository at the given destination, containing the specified packages. Normally all dependencies of the packages are added automatically, when the `--skip-dependency-resolution` flag is given, this step is skipped. If the `--exclude-prebuilds` flag is given, prebuilds are not included in the portable repository and are not required for the generation.

#### `pit util meta-check <REPOSITORY> [PACKAGE-NAME]`
Checks the metadata from the given repository. The `<REPOSITORY>` argument can be a URL or a path to the repository or a repository id specified in `Config.toml`. If a package name is given only that package and the given repository are checked. If no package name is given all packages specified in the `index.toml` from the repository are checked.

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

#### `pit config repositories add <ID> <URL> [PROVIDER] [--unchecked]`
Adds a new repository to the config. Also adds the new repository to the back of the repositories rank. If the `--unchecked` flag is given, the new repository is not checked for availability and compatibility.

#### `pit config repositories remove <ID>`
Removes a repository from the config. Also removes the repository from the repositories rank.

#### `pit config repositories set-url <ID> <URL> [PROVIDER] [--unchecked]`
Sets the url of a repository in the config. If no provider is given, the old provider is used. If the `--unchecked` flag is given, the new repository is not checked for availability and compatibility.

#### `pit config repositories set-prebuilds <ID> <PREBUILDS-URL> [PREBUILDS-PROVIDER]`
Sets the prebuilds url of a repository in the config. If no provider is given, the old provider is used.

#### `pit config repositories disable-prebuilds <ID> <VALUE> [--remove-urls]`
Disables or enables the prebuilds url of a repository in the config. If the `--remove-urls` flag is given, the urls are removed if `<VALUE>` is true.

#### `pit init [--prefix <PREFIX>]`
Initializes the Packit environment by setting up all required files and directories. If the `--prefix` option is given, the given path is used as prefix, instead of the [default prefix](#prefix).
