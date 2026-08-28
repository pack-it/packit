# Switch

The `switch` command has the following command line syntax:<br>
`pit switch <PACKAGE-NAME> <VERSION> [--skip-symlinking]`

## Basic switch
The `switch` command switches the active version of a package, using the following syntax:<br>
`pit switch <PACKAGE-NAME> <VERSION>`

Where `<PACKAGE-NAME>` is the name of the package to switch and `<VERSION>` is the new active version. To learn more about active versions, check out [active packages](../structure.md#active-packages).

Note that the new active version should already be installed.

### Examples
To switch the version of a package called `foo`, use:<br>
`pit switch foo 1.3.8`

## Flags
This is a complete list of all flags that can be used with the `pit switch` command.

### `--skip-symlinking`
Packit packages are symlinked, to learn more about this see [symlinks](../structure.md#symlinks). To skip symlinking `--skip-symlinking` can be used.
