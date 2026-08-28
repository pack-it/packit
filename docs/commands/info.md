# Info

The `info` command has the following command line syntax:<br>
`pit info [<PACKAGE-NAME>[@<VERSION>] [--verbose | -v] [--tree] [--active]]`

## Basic info
The `info` command shows information about the Packit installation or a package, using the following syntax:<br>
`pit info [<PACKAGE-NAME>[@<VERSION>]]`

### Examples
To show information about the Packit installation, use:<br>
`pit info`

To show information about a package named `foo` use:<br>
`pit info foo`

To show information about a specific package version use:<br>
`pit info foo@1.3.7`

## Flags
This is a complete list of all flags that can be used with the `pit info` command.

### `--verbose`
The `--verbose` flag or `-v` for short can be used to show verbose output if available.

### `--tree`
The `--tree` flag will show you the whole dependency tree of a package. This flag requires a package version to be specified.

### `--active`
The `--active` flag can be used to use the active version of a specified package, instead of specifying a version. When both `--active` and a version specification are used, Packit returns an error indicating that the versions are ambiguous. 
