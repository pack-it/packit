# Packit

Packit is a universal package manager, designed to streamline the experience of installing packages on your system.


## Install
TODO


## Usage
The general usage of Packit is: `pit <COMMAND>`.

#### `pit install <PACKAGE-NAME>[@<VERSION>] [--build] [--keep-build] [--skip-symlinking] [--skip-active]`
Installs the specified packages, if a version is given that version will be installed, if not the most recent version will be installed. Multiple packages can be specified by simple entering multiple names, split by a space.
<br>
If the `--build` option is given, the package is build from source, instead of installing a prebuild version.
If the `--keep-build` option is given, the build dependencies will not be deleted after building.
If the `--skip-symlinking` option is enabled, the package is not symlinked into the /bin, /lib, /share, etc. directories.
If the `--skip-active` option is enabled, the package is not set to active and the current active version is kept.

#### `pit uninstall <PACKAGE-NAME>[@<VERSION>]`
Uninstalls the specified packages, if a version is given that version will be uninstalled, if not, you will be asked if you want to delete all versions of `<PACKAGE-NAME>`. Multiple packages can be specified by simple entering multiple names, split by a space.

#### `pit list`
Lists all the installed packages.

#### `pit repositories`
Lists all configured repositories.

#### `pit search <PACKAGE-NAME>[@<VERSION>]`
Searches a package with `<PACKAGE-NAME>`. If the version is given that specific version is searched for.

#### `pit update <PACKAGE-NAME>[@<VERSION>] [<NEW-VERSION>]`
Updates the specified package to the new version, or the latest version if no new version is specified. If multiple versions of the same package are installed, the `<VERSION>` option is required.

#### `pit info <PACKAGE-NAME>[@<VERSION>] [-v] [--tree]`
Shows info about the specified installed package. If the `-v` option is given, extra information is shown. If the `--tree` option is enabled, the whole dependency tree is shown.

#### `pit check [<PACKAGE-NAME>@<VERSION>]`
Checks the Packit installation for issues. When a package name and version is given, only that package is checked for issues.

#### `pit fix`
Fix all issues found by the check command. You will be asked if you want to fix an issue for each issue type.

#### `pit switch <PACKAGE-NAME> <VERSION> [--skip-symlinking]`
Switches the active version of the specified package to the specified version. If the `--skip-symlinking` option is given, the new active version is not symlinked into the /bin, /lib, /share, etc. directories.

#### `pit link <PACKAGE-NAME> [--force]`
Links the specified package into the /bin, /lib, /share, etc. directories. If the package metadata does not allow a package to be symlinked, the `--force` option is required to force the symlinking of the package. Please be careful with using the `--force` option, since there is most likely a good reason to skip symlinking.

#### `pit unlink <PACKAGE-NAME>`
Unlinks the specified package, causing the package to be unavailable from the `PATH` environment variable.

#### `pit package <PACKAGE-NAME>@<VERSION> <DESTINATION>`
Packages the specified package into a prebuild and store it in the destination directory, together with a checksum of the prebuild.


## File structure
You might be interested in how (and why) Packit manages build dependencies, configs and most importantly the installs. We explain that here, ofcourse this differs a bit for each platform as they have different file structures. Luckly Packit manages this for you!


### Unix systems
For these directories we use `/opt/packit`, to support usage of Packit by other users then the superuser.

#### Package install files
The installs will go to: `/opt/packit/packages/<PACKAGE-NAME>/<PACKAGE-VERSION>/`.

#### Active packages
The currently active version of a package will be symlinked in `/opt/packit/active/<PACKAGE-NAME>`. This will link to `/opt/packit/packages/<PACKAGE-NAME>/<ACTIVE-PACKAGE-VERSION>`

#### Symlinks
The active binaries will be symlinked in: `/opt/packit/bin/<EXECUTABLE-NAME>`. This directory needs to be present in the users `PATH` in order for installe binaries to be detected by your system.

#### Packit configs
The Packit configs are located in: `/etc/packit/`, this will include `Config.toml` and `Installed.toml`. 


### Windows
There is no clear directory where packages, binaries or configs are supposed to go in Windows. For this reason we chose to use `%APPDATA%`. 

#### Package install files
The installs will go to: `%APPDATA%/packit/packages/<PACKAGE-NAME>/<PACKAGE-VERSION>/`.

#### Active packages
The currently active version of a package will be symlinked in `%APPDATA%/packit/active/<PACKAGE-NAME>`. This will link to `%APPDATA%/packit/packages/<PACKAGE-NAME>/<ACTIVE-PACKAGE-VERSION>`

#### Symlinks
The active binaries will be symlinked in: `%APPDATA%/packit/bin/<EXECUTABLE-NAME>`

#### Packit configs
The Packit configs are located in: `%APPDATA%/packit/`, this will include `Config.toml` and `Installed.toml`. 
