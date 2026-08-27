# List

The `link` command has the following command line syntax:<br>
`pit list [--updatables] [--active]`

## Basic list
The `list` command lists the installed packages, using the following syntax:<br>
`pit list`

## Flags

This is a complete list of all flags which can be used with the `pit list` command.

### `--updatables`
The `--updatables` flag can be used to list all updatable packages. This flag conflicts with the `--active` flag.

### `--active`
The `--active` will only list the active packages. This flag conflicts with the `--updatables` flag.
