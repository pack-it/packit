# Init

The `init` command has the following command line syntax:<br>
`pit init [--prefix <PREFIX>]`

## Basic init
The `init` command initializes the Packit environment, using the following syntax:<br>
`pit init`

This command will create the [prefix](../structure.md#prefix) directory and sets up all the [required files and directories](../structure.md). You should only use this command if you want to handle the Packit install yourself. Otherwise just use the install script provided by Packit. You can find them at the top of the `README`.

## Flags

This is a complete list of all flags which can be used with the `pit init` command.

### `--prefix <PREFIX>`
The [default prefix](../structure.md#prefix) can be overwriten with the use of the `--prefix` flag. The flag expects a directory path as argument.

#### Example
To initialize Packit at /foo/bar/buz use:<br>
`pit init --prefix /foo/bar/buz`
