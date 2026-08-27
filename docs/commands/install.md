# Install

The `install` command has the following command line syntax:<br>
`pit install <PACKAGE-NAME>[@<VERSION>] ... [--build] [--build-all] [--keep-build] [--skip-symlinking] [--skip-active] [--verbose] [--skip-test] [--skip-build-test] [--pause-build]`

## Basic install
The `install` command installs a package, using the following syntax:<br>
`pit install <PACKAGE-NAME>[@<VERSION>] ...`

### Examples
To install a package called `foo` use:<br>
`pit install foo`

The command above will assume the latest version. To specify a specific version of a package use:<br>
`pit install foo@1.3.7`

It's possible to install multiple packages at once:<br>
`pit install foo@1.3.7 bar buz`

Note that packages with a version specified can be used with packages without a version specified.

## Flags

This is a complete list of all flags which can be used with the `pit install` command.

### `--build`

In a normal install a package prebuild is installed. If the `--build` flag is given the package is build from the source.

### `--build-all`

When `--build` is specified only the specified package is build. For the dependencies prebuilds are still used. `--build-all` will build everything from source, including the package dependencies.

### `--keep-build`

There are two kinds of dependencies, runtime dependencies and build dependencies. The later are generally not needed after an installation, so they are removed. However, if this is not desired `--keep-build` can be specified to keep the build dependencies installed.

### `--skip-symlinking`

Packit packages are symlinked, to learn about this see [symlinks](../structure.md#symlinks). To skip symlinking `--skip-symlinking` can be used.

### `skip-active`

To support multiple installed versions of a package Packit has [active versions](../structure.md#active-packages).
If the `--skip-active` option is enabled, the package is not set as the active version and the current active version is kept. If there is no current active version, this flag is ignored and the package is set to active.

### `--verbose`

The `--verbose` flag or `-v` for short can be used to show verbose output.

### `--skip-test`

When installing packages Packit executes a Packit test afterwards to test if the package was succesfully installed. To skip this the `--skip-test` flag can be used.

Note that this is different from `--skip-build-test`.

### `--skip-build-test`

When building a package from source the packages build tests are executed if they exist. To skip these `--skip-build-test` can be used. This flag is ignored if the package is not build from source.

Note that this is different from `--skip-test`.

### `--pause-build`

This flag is mostly used during development of Packit. When `--pause-build` is specified Packit pauses directly after the build script has executed. It will also show the temporary directory where the build was done. This makes it possible to go to this directory and manually run stuff. This is handy when debugging problems for a certain package.
