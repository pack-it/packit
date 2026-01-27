# Packit

Packit is a universal package manager, designed to streamline the experience of installing packages on your system.


## Install
TODO


## Usage
The general usage of Packit is: `pit <COMMAND>`.

#### `pit install <PACKAGE-NAME> [<VERSION>]`
Installs a package with `<PACKAGE-NAME>`. If the version is given Packit will use that version, if not it will use the most recent version. 

#### `pit uninstall <PACKAGE-NAME> [<VERSION>]`
Uninstalls a package with `<PACKAGE-NAME>`. If the version is given Packit will uninstall that specific version, if not, it will ask you if you want to delete all versions of `<PACKAGE-NAME>`.

#### `pit list [-u]`
Lists all the installed packages. `-u` specifies the use of the install directory, instead of the `Installed.toml`.

#### `pit repositories`
Lists all configured repositories.

#### `pit search <PACKAGE-NAME> [<VERSION>]`
Searches a package with `<PACKAGE-NAME>`. If the version is given Packit will search for that specific version.


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
