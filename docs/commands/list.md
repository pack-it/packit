# List

The `list` command has the following command line syntax:<br>
`pit list [--updatables] [--active]`

## Basic list
The `list` command lists the installed packages, using the following syntax:<br>
`pit list`

## Flags
This is a complete list of all flags that can be used with the `pit list` command.

### `--updatables`
The `--updatables` flag can be used to list all installed packages that have updates available. This flag conflicts with the `--active` flag.

### `--active`
The `--active` flag will only list the active package versions. This flag conflicts with the `--updatables` flag.
