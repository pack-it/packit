# Info

The `info` command has the following command line syntax:<br>
`pit info [<PACKAGE-NAME>[@<VERSION>] [-v] [--tree] [--active]]`

## Basic info
The `info` command shows Packit install info or the info of a package, using the following syntax:<br>
`pit info [<PACKAGE-NAME>[@<VERSION>]]`

### Examples
The info about the Packit install use:<br>
`pit info`

To show info about a package named `foo` use:<br>
`pit info foo`

To show info about a specific package version use:<br>
`pit info foo@1.3.7`

## Flags

This is a complete list of all flags which can be used with the `pit info` command.

### `--verbose`

The `--verbose` flag or `-v` for short can be used to show verbose output if it exists.

### `--tree`

The `--tree` flag will show you the whole dependency tree of a package. This flag requires a package version to be specified.

### `--active`

The `--active` flag can be used to use the active version of a specified package, instead of specifying a version.
