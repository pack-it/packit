# Update

The `update` command has the following command line syntax:<br>
`pit update [<PACKAGE-NAME>[@<VERSION>] ...] [--new-version <NEW-VERSION>] [--all] [--exclude <PACKAGE-NAME> ...]`

## Basic update
The `update` command updates the specified packages, using the following syntax:<br>
`pit update <PACKAGE-NAME>[@<VERSION>] ...`

Updates the specified packages to the latest version. If a version is specified, that version is updated. If no version is specified and only one version is installed that version is updated. If multiple versions are installed and no version is specified only the latest installed version is updated.

Note that you can only update to a later version (so updating to an earlier version is not possible). However, you can install the earlier version.

### Examples
To update a package called `foo`, use:<br>
`pit update foo`

Suppose we have `foo@1.3.7` and `foo@1.3.8` installed. If we don't specify the version and use the command above, `foo@1.3.8` will be updated. To specifically update `foo@1.3.7` we can use:<br>
`pit update foo@1.3.7`

Multiple packages can be updated at once:<br>
`pit update foo@1.3.7 bar buz`

## Flags
This is a complete list of all flags that can be used with the `pit update` command.

### `--new-version <NEW-VERSION>`
With this flag you can specify the new version to update to.

Note that this flag can only be used when a single package is specified.

### `--all`
The `--all` flag can be used to update all latest installed versions to the latest available version. It cannot be used when packages are specified.

### `--exclude <PACKAGE-NAME> ...`
The `--exclude` flag can be used to exclude certain packages when using the `--all` flag.
