# Search

The `search` command has the following command line syntax:<br>
`pit search <QUERY> [--regex] [--verbose] [--tree] [--latest] [--target-only]`

## Basic search
The `search` command searches a package, using the following syntax:<br>
`pit search <QUERY>`

The `<QUERY>` needs to be a package name or package name and version.

### Examples

To search a package called `foo` use:<br>
`pit search foo`

To search for a specific version of `foo` use:<br>
`pit search foo@1.3.7`

## Flags

This is a complete list of all flags which can be used with the `pit search` command.

### `--regex`
When the `--regex` flag is enabled the `<QUERY>` can be a regex expression.

### `--verbose`
The `--verbose` flag or `-v` for short can be used to show verbose output if it exists.

### `--tree`
The `--tree` flag will show you the whole dependency tree of a package. This flag requires a package version to be specified.

### `--latest`
The `--latest` flag can be used to use the latest version of a specified package, instead of specifying a version.

### `--target-only`
When the `--target-only` flag is used only the packages that are available for the current target are shown. This flag requires the `--regex` flag to be used as well.
