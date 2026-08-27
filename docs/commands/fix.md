# Fix

The `fix` command has the following command line syntax:<br>
`pit fix [<PACKAGE-NAME>[@<VERSION>] ...]`

## Basic fix

The `fix` command fixes the specified packages, using the following syntax:<br>
`pit fix [<PACKAGE-NAME>[@<VERSION>] ...]`

The issues will be shown according to urgency, the first issue being the most urgent. The most urgent issue needs to be fixed first, as it could be causing the other issues.

When an issue is found you will be prompted with a fix, you can choose to apply the fix and continue or ignore it and exit.

More information about the workings of this command can be found at [verifier](../verifier.md). If you only want Packit to show you the issues checkout the [check](./check.md) command.

>Note that when checking a single package other issues are not found. Issues with dependencies of the package are not found for example. So when using the `fix` command we recommend you fix everything.

### Examples
To fix everything use:<br>
`pit fix`

To fix all versions of `foo` use:<br>
`pit fix foo`

To fix a specific version of `foo` use:<br>
`pit fix foo@1.3.7`

It's possible to fix multiple packages at once:<br>
`pit fix foo@1.3.7 bar buz`

Note that packages with a version specified can be used with packages without a version specified.
