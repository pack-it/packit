# Installation Process

This file aims to explain the installation process as accurately as possible. It also includes the build process.

## Dependency resolution
The first thing that happens when installing a package is the dependency resolution. Resolving a single dependency starts with choosing a repository to get the package metadata from. This order is defined with the [repository rank](./structure.md#config). Packit will get the package metadata from the first repository which contains the package. If the package cannot be found an error is returned. Then Packit chooses from the valid dependency versions. It will choose the latest satisfying version that is not deprecated. If all versions are deprecated it will choose the version that got deprecated the latest.

This is done for every dependency to create the dependency tree. In a normal installation (one without a source build) there is a check for the prebuilds of the packages in the tree. If a prebuild cannot be found the user is asked if they want to do a source build for that package instead. If not, the installation is cancelled.

Once the dependency tree is decided it is shown to the user. The tree is color coded as follows:
| Color   | Explanation                                         |
| ------- | --------------------------------------------------- |
| Blue    | Packages which are already installed.               |
| Cyan    | Packages which are being installed with a prebuild. |
| Purple  | Packages which are being build from source.         |

This tree is then traversed bottom up, so dependencies are installed first.

## After an installation
After each package installation the permissions of the package files are set. Then the installation of that package is complete and it is added to the [Register.toml](./structure.md#registertoml). The post-install script is executed after this if it exists. Finally a Packit test script for that package is run to check if the installation was successful. The test script executes some basic functionality of the package.

Once the entire installation is done the build dependencies are removed. It is possible to skip this with use of the [`--keep-build`](./commands/install.md#--keep-build) flag.

## Building from source
The builder starts by downloading the source files, which it then unpacks into a temporary directory. If [patches](./metadata.md#patches) are specified in the metadata, those will be applied according to their order. After this the [build environment](./build-env.md) is created based on the dependencies and requirements of the package. If [script arguments](./metadata.md#script-environment) are defined in the metadata those will be constructed and added to the environment.

At this point the build script of the package can be run. Afterwards the exit status is checked and returned in case of an error. If[`--pause-build`](./commands/install.md#--pause-build) is specified the builder pauses until the user chooses to continue.

Once the build is finished the license files are copied to the `<package-prefix>/share/licenses/<package-name>` directory. The `<package-name>` directory exists to avoid conflicts between packages when symlinking them.

Now the build is almost done. At this point Packit patches the binaries, not to be confused with the patches defined in the metadata. These patches allow Packit to switch between package dependency versions. A package binary will contain paths to its dependencies. These paths point directly to the dependency package. Packit has a `<prefix>/dependencies/<package-name>` directory which contains symlinks to the different dependency versions. When patching the binaries of a package, Packit searches for paths which have the `<prefix>/packages` prefix. If this is the case, the path should be swapped with the corresponding symlink in the `dependencies` directory.

For example: `<prefix>/packages/foo_dependency/3.7.1` would be changed to `<prefix>/dependencies/foo/foo_dependency`. Where `foo_dependency` point to `<prefix>/packages/foo_dependency/<version>`.

The result is that the paths in the patched binaries point to their dependencies indirectly through the symlinks in the `dependencies` directory.

Later when the version of a package dependency needs to be switched, Packit just changes the symlink in the `dependencies` directory to point at the new dependency version.

## Future additions
There are two things that we will try to add in future Packit versions. The installation process is not yet parallelized, doing this would probably result in a significant performance increase. The installation of a single package is also not yet atomic. Meaning that if a package installation is cancelled half way through, Packit will be in a half state. Most of the time it possible to fix this with the [`fix`](./commands/fix.md) command, because it is quite advanced. However, it would be better if this was not the case, so we will try to make package installations atomic in future versions.
