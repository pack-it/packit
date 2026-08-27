# Packit Structure and Configuration

This file explains the Packit structure and configuration.

## File Structure
The Packit directory, which contains Packit data is called the [prefix](#prefix) directory. This directory contains the following files and directories:
- [Register.toml](#registertoml)
- [packages](#packages)
- [bin, gnubin, lib, include and share](#symlinks)
- [active](#active-packages)
- [dependencies](#dependencies)
- [etc](#etc)

### Prefix
The prefix directory of Packit contains all data of installed packages.<br>
- Unix: `/opt/packit`<br>
- Windows: `C:\Program Files\packit`

### Register.toml
The `Register.toml` file is located inside the prefix and stores information about all installed packages. This file is managed by Packit and should not be changed directly.

### Packages
All installed packages will go in `<prefix>/packages/<PACKAGE-NAME>/<PACKAGE-VERSION>/`.

### Symlinks
The following package directories are symlinked in the prefix directory: bin, gnubin, lib, include and share. The gnubin directory exists on macOS to put GNU packages in which conflict with their macOS variants. In such cases a symlink `<prefix>/bin/g<package-name>` → `<prefix>/gnubin/<package-name>` with a 'g' prefix is created, to differentiate between a system and its GNU variant.

The [active](#active-packages) binaries will be symlinked in `<prefix>/bin/<EXECUTABLE-NAME>`. The `<prefix>/bin` directory needs to be present in the users `PATH` in order for installed binaries to be detected by the system.

### Active packages
Packit can install multiple versions of a package next to each other. A package always has one active version, which will be symlinked as `<prefix>/active/<PACKAGE-NAME>` → `<prefix>/packages/<PACKAGE-NAME>/<ACTIVE-PACKAGE-VERSION>`

### Dependencies
The dependencies directory contains symlinks to the dependencies of a package. The structure is as follows `<prefix>/dependencies/<package-name>/<dependency>`, where `<package-name>` is a package with dependency `<dependency>`. `<dependency>` is a symlink to the current active package. 

For more detail about the reason why this directory exists see [TODO: a link to some explanation about the patching and how it's used for active ]

### Package Data
Some packages have data and configuration files they need to keep. Packit puts those in the `<prefix>/etc` directory.


## Config
The `Config.toml` contains the Packit configuration, it can be edited manually or with the [`pit config`](#pit-config-show) command. Its location differs for each platform:
- Linux: `/etc/packit`<br>
- MacOS: `/Library/Application Support/packit`<br>
- Windows: `C:\Program Files\packit`


All available fields in the config are listed below. 

| Field               | Explanation                                                                                                 |
| ------------------- | ----------------------------------------------------------------------------------------------------------- |
| `prefix_directory`  | Defines the directory used for installing packages, see [File structure](#file-structure) for the defaults on each platform. |
| `repositories_rank` | Defines the order of repositories to search for a package. |
| `multiuser`         | True to run Packit in multiuser mode, false for single user mode. |

### Repositories

| Field                 | Explanation                                                                             |
| --------------------- | --------------------------------------------------------------------------------------- |
| `url`                 | Defines the url to the repository.                                                      |
| `provider`            | Defines the provider of the repository which can be `fs` or `web`, defaults to `web`.   |
| `prebuilds_url`       | Defines the url of the prebuilds repository for this package repository.                |
| `prebuilds_provider`  | Defines the provider of the prebuilds repository, defaults to `fs`.                     |
| `disable_prebuilds`   | True to disable prebuild usage for the repository, false to use prebuild if available.  |


### Example config

```
repositories_rank = ["core"]

[repositories]

[repositories.core]
provider = "web"
url = "https://raw.githubusercontent.com/pack-it/core/main/"
```


