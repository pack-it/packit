# Uninstall

The `uninstall` command has the following command line syntax:<br>
`pit uninstall <PACKAGE-NAME>[@<VERSION>] ...`

## Basic uninstall
The `uninstall` command uninstalls the specified packages, using the following syntax:<br>
`pit install <PACKAGE-NAME>[@<VERSION>] ...`

When multiple versions of a package are installed and the version isn't specified you will be asked if you want to uninstall all installed versions of `<PACKAGE-NAME>`.

When uninstalling multiple packages at once Packit will automatically determine the correct order. Meaning it will uninstall dependencies after their dependents.

### Examples
To uninstall a package named `foo` use:<br>
`pit uninstall foo`

To uninstall a specific version of `foo` use:<br>
`pit uninstall foo@1.3.7`

It's possible to uninstall multiple packages at once:<br>
`pit uninstall foo@1.3.7 bar buz`

Note that packages with a version specified can be used with packages without a version specified.
