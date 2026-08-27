# Check

The `check` command has the following command line syntax:<br>
`pit check [<PACKAGE-NAME>[@<VERSION>] ...]`

## Basic check

The `check` command checks the specified packages, using the following syntax:<br>
`pit check [<PACKAGE-NAME>[@<VERSION>] ...]`

The issues will be listed according to urgency, the first issue being the most urgent. The most urgent issue needs to be fixed first, as it could be causing the other issues.

More information about the workings of this command can be found at [verifier](../verifier.md). If you also want Packit to fix the issues for you checkout the [fix](./fix.md) command.

### Examples
To check everything use:<br>
`pit check`

To check all versions of `foo` use:<br>
`pit check foo`

To check a specific version of `foo` use:<br>
`pit check foo@1.3.7`

It's possible to check multiple packages at once:<br>
`pit check foo@1.3.7 bar buz`

Note that packages with a version specified can be used with packages without a version specified.
