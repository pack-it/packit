# Search

The `search` command has the following command line syntax:<br>
`pit search <QUERY> [--regex] [--verbose | -v] [--tree] [--latest] [--target-only]`

## Basic search
The `search` command searches for a package, using the following syntax:<br>
`pit search <QUERY>`

The `<QUERY>` needs to be a package name or a package name with a version.

### Examples
To search a package called `foo`, use:<br>
`pit search foo`

To search for a specific version of `foo`, use:<br>
`pit search foo@1.3.7`

## Flags
This is a complete list of all flags that can be used with the `pit search` command.

### `--regex`
When the `--regex` flag is enabled, the `<QUERY>` can be a regular expression. The regex search doesn't account for the current target. So it could show packages which are not available for your target. You can check whether a package is available for the current target by searching for that specific package.

### `--verbose`
The `--verbose` flag or `-v` for short can be used to show verbose output if available.

### `--tree`
The `--tree` flag displays the complete dependency tree of a package. This flag requires a package version to be specified. For the dependencies, it assumes the latest version. It cannot be used with the `--regex` flag.

### `--latest`
The `--latest` flag can be used to select the latest version of a specified package instead of specifying a version. It cannot be used with the `--regex` flag.

### `--target-only`
When the `--target-only` flag is used only the packages that are available for the current target are shown. This flag requires the `--regex` flag.
