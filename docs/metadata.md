# Repository Structure

### `repository.toml`
This file should be present in every Packit repository, it quickly describes what the repository is for.

| Field                | Explanation                                                         |
| -------------------- | ------------------------------------------------------------------- |
| `name`               | The name of the repository. (required)                              |
| `description`        | A small description of the repository. (required)                   |
| `license`            | The license of the repository.                                      |
| `maintainers`        | A list of maintainers of the repository. (required)                 |
| `prebuilds_url`      | Defines the url of the prebuilds repository for this repository.    |
| `prebuilds_provider` | Defines the provider of the prebuilds repository, defaults to `fs`. |


### `packages`
The packages directory contains the metadata of all packages which are supported by this repository.


### `package.toml`
Each package contains this file, it describes the package as whole. It shows the following general package information:
- Name
- Short description
- Package homepage url
- Available versions
- Supported versions (for each target, see [Target bounds](#target-bounds))


### `targets.toml`
Each package version directory contains a `targets.toml` file. This file describes version specific information. This information can be the same for all targets (global) or target specific. In some cases the target specific information will override the global information in other cases it's additive, so global and target specific will be used together.
See the tables below for all different fields, see [Target fields](#target-fields) to get more information about additive and overrides.

#### Global fields
| Field                           | Explanation                                                                    |
| ------------------------------- | ------------------------------------------------------------------------------ |
| `version`                       | Defines the version of the package.                                            |
| `license`                       | The license of this version of the package.                                    |
| `dependencies`                  | Defines all the dependencies of the package, that are shared by all targets.   |
| `build_dependencies`            | Defines all build dependencies of the package, that are shared by all targets. |
| `use_version_specific_<script>` | When set to yes, the specified script is read from the package version directory, instead of the package directory. |
| `skip_symlinking`               | When set to yes, the package is not symlinked after installation, preventing the package to be detectable through the PATH. |
| `revisions`                     | A list of strings containing a description of what changed in each metadata or script revision. |
| `script_args`                   | A table of key-value pairs containing arguments passed to scripts.             |

#### Sources
The targets.toml file can contain one or multiple sources, specified in the following format. When multiple sources are defined, they need to be named.

| Field         | Explanation                                                                                                  |
| ------------- | ------------------------------------------------------------------------------------------------------------ |
| `url`         | Defines the url of the archive containing the sourcecode of the package.                                     |
| `checksum`    | Defines the sha256 checksum of the source archive.                                                           |
| `mirrors`     | Defines a list of mirrors which could be used to download the sourcecode if the original url is unavailable. |
| `skip_unpack` | True to skip the unpack step and just download the source file, false to use the build in unpack.            |

#### Target fields

Targets are specified as `[targets.<bounds>]`, where bounds specify the support target as described in [Target bounds](#target-bounds).

| Field                           | Explanation                                                                          |
| ------------------------------- | ------------------------------------------------------------------------------------ |
| `dependencies`                  | Defines all the dependencies of the package for the target, additional to the dependencies specified in the global field. |
| `build_dependencies`            | Defines all build dependencies of the package for the target, additional to the build dependencies specified in the global field. |
| `skip_symlinking`               | When set to yes, the package is not symlinked after installation, preventing the package to be detectable through the PATH. Overrides the value defined in the global field. |
| `<script-type>_script`          | Defines the name of the script to use instead of the default script name.            |
| `script_args`                   | A table of key-value pairs containing arguments passed to scripts, additional to the args defined in the global field. |
| `source`                        | Defines which source to use, required when multiple sources are defined.             |

### Target bounds

The target bounds consist of a name, an addition and version bounds, the name is split up in three different categories.

When selecting the target to use, there is a certain priority, from lowest priority to highest priority:
- OS group
- OS name
- Target architecture
- OS name with version bounds
- Target architecture with version bounds
- OS name with addition and version bounds
- Target architecture with addition and version bounds

The syntax of defining a target bound is as follows, where target names are required and additions and version bounds are optional: <br>
`<name>[:<addition>][@<version-bounds>]`

#### Target names 

| Name                | Supported values            |
| ------------------- | --------------------------- |
| OS group            | `unix`                      |
| OS name             | `macos`, `linux`, `windows` |
| Target architecture | `x86_64-apple-darwin`, `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` |

#### Target additions

Currently additions are only supported for the `linux` target name and for the target architectures which reference a linux OS. The addition specifies a Linux distro, for example `debian` or `arch`.

#### Target version bounds

Version bounds specify the version of the target which is required for the target bounds to be satisfied. Version bounds are not allowed on OS group target names.

The version bounds specify the OS version on macOS and Windows. On Linux it specifies the kernel version when no addition is given, or the distro version when an addition is given.

Within the same target name, overlapping version bounds are not allowed and will result in invalid package metadata.

See [Version bounds](#version-bounds) for the version bounds syntax.


### Version bounds

Version bounds are used by target bounds, by dependencies and by the supported versions to specify which version satisfy a target or dependency.

Please note version bounds are required to be in order of versions.

The following operators are available:
| Operator    | Explanation                                                                          |
| ----------- | ------------------------------------------------------------------------------------ |
| No operator | Specifies a specific version.                                                        |
| `-`         | Specifies a version range, for example `1-2` (does not include 2).                   |
| `-=`        | Specifies an including version range, for example `1-2` (does include 2).            |
| `<=`        | Specifies a version upper bound including the specified version, for example `<=2`.  |
| `<`         | Specifies a version upper bound excluding the specified version, for example `<2`.   |
| `>=`        | Specifies a version lower bound including the specified version, for example `>=1`.  |
| `>`         | Specifies a version lower bound excluding the specified version, for example `>1`.   |
| `\|`        | Can be used to chain multiple bounds, works as an or operator, for example `3\|5-7`. |


### Scripts

The scripts define the specific behaviour to install, uninstall or test a specific package. They can be defined globally for a package, per version or per target. On unix systems the script are written in `sh` and have the `.sh` extension. On Windows the scripts are written in `batch` and have the `.bat` extension.

The available scripts are:

| Script name          | Explanation                                                                                     |
| -------------------- | ----------------------------------------------------------------------------------------------- |
| `preinstall`         | The preinstall script is run before installing a package.                                       |
| `build`              | The build script is run to build a package.                                                     |
| `postinstall`        | The postinstall script is run after the package is installed.                                   |
| `test`               | The test script is called after the package is installed to test if the install was successful. |
| `uninstall`          | The uninstall script is run after an uninstall to cleanup all package data.                     |

#### Script environment

Scripts get certain environment variables from Packit:

| Variable name                      | Explanation                                                                                |
| ---------------------------------- | ------------------------------------------------------------------------------------------ |
| `PACKIT_PREFIX_PATH`               | The Packit prefix path, as set in the configuration.                                       |
| `PACKIT_TARGET`                    | The current target architecture, one of the values of the target architecture target name. |
| `PACKIT_OS`                        | The current operating system, `mac`, `linux` or `windows`.                                 |
| `PACKIT_PACKAGE_PATH`              | The path where the package to which the script belongs is installed to.                    |
| `PACKIT_PACKAGE_VERSION`           | The version of the package the script belongs to.                                          |
| `PACKIT_PACKAGE_DEPENDENCIES_PATH` | The path containing symlinks to all dependencies of the package.                           |
| `PACKIT_VERBOSE`                   | True (1) if verbose output is enabled, false (0) otherwise.                                |

Please note that the build script output is only shown to the user when the verbose mode is turned on. All other scripts always show their output, the output of these scripts should thus be clean. Optional verbose output can be printed when the `PACKIT_VERBOSE` is `1`.

The script arguments that are defined in the metadata are passed to the script as environment variable as `PACKIT_ARGS_<argument-name>`.

The environment of build scripts are managed more extensively to ensure reproducible builds.
