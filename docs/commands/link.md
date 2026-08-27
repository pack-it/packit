# Link

The `link` command has the following command line syntax:<br>
`pit link <PACKAGE-NAME> [--force] [--overwrite]`

## Basic link
The `link` command links a package, using the following syntax:<br>
`pit link <PACKAGE-NAME>`

The `link` command symlinks the specified packages. To learn more about Packit symlinks checkout [symlinks](../structure.md#symlinks).

### Examples
To link a package called `foo` use:<br>
`pit link foo`

Note that only a package name should be given.

## Flags

This is a complete list of all flags which can be used with the `pit link` command.

### `--force`
Some packages are not allowed to be symlinked (defined in the [metadata](../metadata.md)). In such a case the `--force` option is required to force the symlinking of the package. Please be careful with using the `--force` option, since there is most likely a good reason to skip symlinking.

### `--overwrite`
Some packages conflict with each other, this should be defined in the [metadata](../metadata.md). In this case it's not possible for both packages to be symlinked at the same time. The `--overwrite` option can be used to overwrite existing symlinks from another conflicting package, please note that this should normally not be used, as conflicts between packages should be avoided.
