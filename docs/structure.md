# Structure

This file explains the Packit structure.

- [Prefix](#prefix)
- [Register](#register)
- [Local metadata](#local-metadata)
- [Packages](#packages)
- [Symlinks](#symlinks)
- [Active](#active-packages)
- [Dependencies](#dependencies)
- [Package Data](#package-data)

## Prefix
The Packit directory, which contains Packit data is called the [prefix](#prefix) directory. The default location of the prefix directory differs for each platform: <br>
| Platform | Directory                 |
| -------- | ------------------------- |
| Unix     | `/opt/packit`             |
| Windows  | `C:\Program Files\packit` |

This prefix directory contains the following files and directories:
- [Register.toml](#register)
- [metadata](#local-metadata)
- [packages](#packages)
- [bin, gnubin, lib, include and share](#symlinks)
- [active](#active-packages)
- [dependencies](#dependencies)
- [etc](#package-data)

## Register
The `Register.toml` file is located inside the prefix and stores information about all installed packages. This file is managed by Packit and should not be changed directly. Find more information about this file [here](./register.md#register)

## Local Metadata
The `metadata` directory contains all of the local metadata of the installed packages. This directory is managed by Packit and should not be changed directly.

## Packages
All installed packages will go in `<prefix>/packages/<PACKAGE-NAME>/<PACKAGE-VERSION>/`.

## Symlinks
The following package directories are symlinked in the prefix directory: bin, gnubin, lib, include and share. The gnubin directory exists on macOS to isolate GNU packages that conflict with their macOS variants. In such case a symlink `<prefix>/bin/g<package-name>` → `<prefix>/gnubin/<package-name>` with a 'g' prefix is created, to differentiate between a system package and its GNU variant. To use the GNU tools and overwrite the macOS variants, you could add the gnubin directory to the front of your `PATH` as well.

The [active](#active-packages) binaries will be symlinked in `<prefix>/bin/<EXECUTABLE-NAME>`. The `<prefix>/bin` directory needs to be present in the users `PATH` in order for installed binaries to be detected by the system.

Some packages should not be symlinked, if this is the case it is specified in the [metadata](./metadata.md). A reason for not symlinking a package could be that it conflicts with another package or with a system library. 

## Active packages
Packit can install multiple versions of a package next to each other. A package always has one active version, which will be symlinked as `<prefix>/active/<PACKAGE-NAME>` → `<prefix>/packages/<PACKAGE-NAME>/<ACTIVE-PACKAGE-VERSION>`

## Dependencies
The dependencies directory contains symlinks to the dependencies of a package. The structure is as follows `<prefix>/dependencies/<package>/<dependency>`, where `<package>` is a package with dependency `<dependency>`. `<dependency>` is a symlink to the current active package. 

For more details about the reason why this directory exists see [building from source](./installation-process.md#building-from-source).

## Package Data
Some packages have data and configuration files they need to keep. Packit puts those in the `<prefix>/etc` directory.
