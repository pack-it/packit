# Repository Structure

This file explains the Packit metadata repository structure and shows some examples where necessary. Not everything has an example, a simple example package which covers the basics is: [libssh2](https://github.com/pack-it/core/tree/main/packages/libssh2). For something more elaborate look at: [xz](https://github.com/pack-it/core/tree/main/packages/xz).

## `repository.toml`
This file should be present in every Packit repository, it quickly describes what the repository is for.

| Field                     | Explanation                                                                   |
| ------------------------- | ----------------------------------------------------------------------------- |
| `name`                    | The name of the repository. (required)                                        |
| `description`             | A small description of the repository. (required)                             |
| `license`                 | The license of the repository.                                                |
| `maintainers`             | A list of maintainers of the repository. (required)                           |
| `required_packit_version` | The minimum required Packit version to use the repository.                    |
| `prebuilds_url`           | Defines the URL of the suggested prebuilds repository for this repository.    |
| `prebuilds_provider`      | Defines the provider of the suggested prebuilds repository, defaults to `fs`. |

## `index.toml`
This file should be present in every Packit repository, it describes which packages are available.

| Field                | Explanation                                                            |
| -------------------- | ---------------------------------------------------------------------- |
| `supported_packages` | A list of names of all packages that are available in this repository. |

## `packages`
The packages directory contains the metadata of all packages which are supported by this repository.

## `package.toml`
Each package contains this file, it describes the package as whole. It shows the following general package information:
- Name
- Short description
- Package homepage URL
- Available versions
- The minimum required Packit version
- Conflicting packages
- Supported versions (for each target, see [Target bounds](#target-bounds))
- Deprecation information about the package

## `targets.toml`
Each package version directory contains a `targets.toml` file. This file describes version specific information. This information can be the same for all targets (global) or target specific. Some fields in a target section override their global value. Other fields are additive, meaning both the global and target-specific values are used.
See the tables below for all different fields, look at [Target fields](#target-fields) for more information about additive and overrides.

### Global fields
| Field                           | Explanation                                                                    |
| ------------------------------- | ------------------------------------------------------------------------------ |
| `version`                       | Defines the version of the package.                                            |
| `required_packit_version`       | The minimum required Packit version to use the package.                        |
| `license`                       | The license of this version of the package.                                    |
| `dependencies`                  | Defines all the dependencies of the package, that are shared by all targets.   |
| `build_dependencies`            | Defines all build dependencies of the package, that are shared by all targets. |
| `use_version_specific_<script>` | When set to true, the specified script is read from the package version directory, instead of the package directory. |
| `use_<script>`                  | Needs to be set to true when the script should be used. (Only for `preinstall`, `postinstall` and `uninstall`) |
| `skip_symlinking`               | When set to true, the package is not symlinked after installation, preventing the package to be detectable through the PATH. |
| `revisions`                     | A list of strings containing a description of what changed in each metadata or script revision. |
| `deprecation`                   | Defines when the version deprecates, disables and the reason.                  |
| `script_args`                   | A table of key-value pairs containing arguments passed to scripts.             |
| `external_test_files`           | A list of external test files that are needed for executing the test script. These files are automatically downloaded. |

> Note that for the `license` field we try to be as accurate as possible. However sometimes the specific version of a license can be difficult to find, so it could be wrong. In such a case please create an issue on Packit.

### Sources
The targets.toml file can contain one or more sources, specified in the following format. When multiple sources are defined, they need to be named.

| Field              | Explanation                                                                                                  |
| ------------------ | ------------------------------------------------------------------------------------------------------------ |
| `url`              | Defines the URL of the archive containing the source code of the package.                                    |
| `checksum`         | Defines the sha256 checksum of the source archive.                                                           |
| `size`             | Defines the size of the source archive in bytes.                                                             |
| `mirrors`          | Defines a list of mirrors which could be used to download the source code if the original URL is unavailable.|
| `skip_unpack`      | True to skip the unpack step and just download the source file, false to use the build-in unpack.            |
| `license_exclude`  | A list of paths to skip when doing automatic license file detection. `*` can be used to skip detection entirely. |
| `license_include`  | A list of files that need to be copied to the package license file directory. These files are copied before the automatic detection. |
| `apply_patches_in` | Defines the directory that should be used to apply all patches in.                                           |
| `patches`          | A list of patches to apply to the source. See patches section below.                                         |

#### Example
In this example there are multiple sources, because Unix and Windows have different source URLs. Notice that both sources are now named (`unix` and `windows` respectively). If there was only one source this would not be necessary.
```
[source.unix]
url = "https://some-package/unix-version/4.3.tar.gz"
checksum = "a0d5b7389f4e067e34ceebabbefbe26dd9b55ffd0a097c26087b4897b25a8659"
size = 87625

[source.windows]
url = "https://some-package/windows-version/4.3.tar.gz"
checksum = "8719374f5a0e8089cd8bf3960d46c6b236d45217509cd07cc6931b41f91b55af"
size = 89825
```

### Patches
The `patches` field in a source is specified in the following format. Patches are indexed with a number, so the first patch is specified by key `patches.0`.

| Field      | Explanation                                                                                             |
| ---------- | ------------------------------------------------------------------------------------------------------- |
| `url`      | Defines the URL of the patch. Can contain a URL, or a local file relative to the package directory.     |
| `checksum` | Defines the sha256 checksum of the patch.                                                               |
| `mirrors`  | Defines a list of mirrors which could be used to download the patch if the original URL is unavailable. |
| `apply_in` | Defines the directory that should be used to apply this specific patch in.                              |

Please note that the `mirrors` field should not be used when the URL points to a file in the repository, this file is then expected to exist.

#### Example
This example shows a source with two patches. Note that the patches are numbered, this determines the execution order.
```
[source]
url = "https://some-package/some-2.7.tar.gz"
checksum = "b307462038a9b908ba93f655b2e3b122e3c701c8cfa04478f848195bd748a5ec"
size = 12971567
patches.0 = {
    url = "https://some-package/some.patch",
    checksum = "b649db750d28a4283c392f9912f60bdb7601ae579bd5ddc20d1fe992b2727f25"
}
patches.1 = {
    url = "https://other-package/other.patch",
    checksum = "e0c5e4c5fe2f2fac8db101e22263a0981506a5237ebf2b22ff79a19a3c399b41"
}
```

### Deprecation
The `deprecation` fields in package.toml and targets.toml describe when a package is deprecated, when it will be disabled and why.

| Field             | Explanation                                                                      |
| ----------------- | -------------------------------------------------------------------------------- |
| `deprecated_from` | Defines the date (in YYYY-MM-DD) when the package will be deprecated (required). |
| `disabled_from`   | Defines the date (in YYYY-MM-DD) when the package will be disabled, installing the package will not be possible anymore after this date. |
| `reason`          | Defines the reason the package is deprecated.                                    |

### Target fields
Targets are specified as `[targets.<bounds>]`, where bounds specify the supported target as described in [Target bounds](#target-bounds).

| Field                           | Explanation                                                                          |
| ------------------------------- | ------------------------------------------------------------------------------------ |
| `dependencies`                  | Defines all the dependencies of the package for the target, additional to the dependencies specified in the global field. |
| `build_dependencies`            | Defines all build dependencies of the package for the target, additional to the build dependencies specified in the global field. |
| `build_requirements`            | Defines all requirements that are needed on the system for building the package, these need to be installed by the user manually. |
| `test_requirements`             | Defines all requirements that are needed on the system for testing the package, these need to be installed by the user manually. |
| `skip_symlinking`               | When set to true, the package is not symlinked after installation, preventing the package to be detectable through the PATH. Overrides the value defined in the global field. |
| `<script-type>_script`          | Defines the name of the script to use instead of the default script name.            |
| `use_<script>`                  | Overwrites the global `use_<script>` field. (Only for `preinstall`, `postinstall` and `uninstall`) |
| `script_args`                   | A table of key-value pairs containing arguments passed to scripts, additional to the args defined in the global field. |
| `source`                        | Defines which source to use, required when multiple sources are defined.             |
| `external_test_files`           | A list of external test files that are needed for executing the test script for this target, additional to the files specified in the global field. These files are automatically downloaded. |

### Available requirements
All available requirements for use with the `build_requirements` and `test_requirements` fields. Requirements are things that need to be available on the system before building or testing, they need to be installed by the user manually.

| Field  | Explanation                                                                                                   |
| ------ | ------------------------------------------------------------------------------------------------------------- |
| `msvc` | The Microsoft Visual C++ toolchain, automatically adds `PACKIT_VS_PATH`, `PACKIT_VCVARSALL`, `PACKIT_VCVARSALL_ARCH` and `PACKIT_MSVC_VERSION` to the build environment. Find out more [here](./build-env.md#windows-msvc) (Windows only) |

## `prebuilds.toml`
The package version directory can contain an optional `prebuilds.toml` file. This file describes which prebuilds can be generated for the package version. If the file is not available, a default list of prebuilds is assumed, consisting of a prebuild for each target architecture that is supported by the package.

### Prebuild fields
A prebuild is specified by using `[prebuild.<prebuild-id>]`. Each prebuild can have the fields listed below.

| Field           | Explanation                                                                                     |
| --------------- | ----------------------------------------------------------------------------------------------- |
| `targets`       | Defines a list of [target bounds](#target-bounds) that the prebuild can be used for. (required) |
| `exclude_paths` | Defines a list of paths that should not be included in the prebuild.                            |

## Target bounds
The target bounds consist of a name, an addition and [version bounds](#target-version-bounds). The target name is required, the addition and version bounds are not. The syntax of a target bound is as follows: `<name>[:<addition>][@<version-bounds>]`.

Packit selects the most specific matching target bound according to the following priority order, from lowest to highest:
- OS group
- OS name
- Target architecture
- OS name with version bounds
- Target architecture with version bounds
- OS name with addition and version bounds
- Target architecture with addition and version bounds

### Target names 
| Name                | Supported values            |
| ------------------- | --------------------------- |
| OS group            | `unix`                      |
| OS name             | `macos`, `linux`, `windows` |
| Target architecture | `x86_64-apple-darwin`, `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` |

### Target additions
Currently additions are only supported for the `linux` target name and for the target architectures which reference a linux OS. The addition specifies a Linux distro, for example `debian` or `arch`.

### Target version bounds
Version bounds specify the version of the target which is required for the target bounds to be satisfied. The version bounds specify the OS version on macOS and Windows. On Linux it specifies the kernel version when no addition is given, or the distro version when an addition is given. 

Note that version bounds are not allowed on OS group target names and overlapping version bounds are not allowed within the same target name. Both will result in invalid package metadata.

See [Version bounds](#version-bounds) for the version bounds syntax.

## Version bounds
Version bounds are used by target bounds, dependencies and the supported versions to specify which versions satisfy a target or dependency.

Please note version bounds are required to be in order of versions.

The following operators are available:
| Operator    | Explanation                                                                            |
| ----------- | -------------------------------------------------------------------------------------- |
| No operator | Specifies a specific version.                                                          |
| `-`         | Specifies a version range, for example `1-2` (does not include 2).                     |
| `-=`        | Specifies an including version range, for example `1-=2` (does include 2).             |
| `<=`        | Specifies a version upper bound including the specified version, for example `<=2`.    |
| `<`         | Specifies a version upper bound excluding the specified version, for example `<2`.     |
| `>=`        | Specifies a version lower bound including the specified version, for example `>=1`.    |
| `>`         | Specifies a version lower bound excluding the specified version, for example `>1`.     |
| `\|`        | Can be used to chain multiple bounds, works as an `OR` operator, for example `3\|5-7`. |

## Licenses
The licenses can be specified in a structured format, that is easy to parse. A license field can have the following values:
- `"<license-name>"`
- `{ name = "<license-name>", with = ["<exception>"] }`
- `{ any = [ <license-value> ] }`
- `{ all = [ <license-value> ] }`

The `any` and `all` fields allow nesting another license value. For example, the following complex license is valid: <br>
`{ all = [ "License-1", { any = [ { name = "License-2", with = ["Exception-1", "Exception-2"] }, "License-3" ] } ] }`

Note that the `<license-name>` and `<exception>` should be an SPDX Identifier if it is available. See https://spdx.org/licenses/.

## Scripts
The scripts define the specific behaviour to install, uninstall or test a specific package. They can be defined globally for a package, per version or per target. On Unix systems the scripts are written in `sh` and have the `.sh` extension. On Windows the scripts are written in `batch` and have the `.bat` extension.

The available scripts are:

| Script name          | Explanation                                                                                     |
| -------------------- | ----------------------------------------------------------------------------------------------- |
| `preinstall`         | The preinstall script is run before installing a package.                                       |
| `build`              | The build script is run to build a package.                                                     |
| `postinstall`        | The postinstall script is run after the package is installed.                                   |
| `test`               | The test script is called after the package is installed to test if the install was successful. |
| `uninstall`          | The uninstall script is run after an uninstall to clean up all package data.                    |

Note that the `build` script must be present and that the `test` script should be present. The other scripts are optional and depend on the package.

### Script environment
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
| `PACKIT_INCLUDE_BUILD_TEST`        | True (1) if the build tests should be executed, false (0) otherwise.                       |
| `PACKIT_BUILD_JOBS_COUNT`          | The number of jobs to use for build processes in the build script. Using `2 * the CPU count` as a heuristic.                           |

Please note that the build script output is only shown to the user when the verbose mode is turned on. All other scripts always show their output, the output of these scripts should thus be clean. Optional verbose output can be printed when the `PACKIT_VERBOSE` is `1`.

The script arguments that are defined in the metadata are passed to the script as environment variable as `PACKIT_ARGS_<argument-name>`.

The environment of build scripts are managed more extensively to make builds more reproducible.

Scripts have the ability to use file descriptor 3 to print verbose output, this output is only shown to the user when verbose mode is turned on. Scripts should only print absolutely necessary output to stdout and stderr, other output should be redirected to this verbose stream.

Please note that on Windows `%PACKIT_OUTPUTS% >&3` is required to redirect output to this verbose stream, while just `>&3` is enough on Unix.
