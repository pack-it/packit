# Switch dependencies

The `switch-dependency` command has the following command line syntax:<br>
`pit switch-dependency <PACKAGE-NAME>@<VERSION> <DEPENDENCY-NAME> <NEW-DEPENDENCY-VERSION>`

## Basic dependency switch
The `switch-dependency` command switches the active version from the dependency of a package, using the following syntax:<br>
`pit switch-dependency <PACKAGE-NAME>@<VERSION> <DEPENDENCY-NAME> <NEW-DEPENDENCY-VERSION>`

Where `<PACKAGE-NAME>@<VERSION>` is the package version from which the dependency should be switched. `<DEPENDENCY-NAME>` is the name of the dependency to switch and `<NEW-DEPENDENCY-VERSION>` is the new active version of the dependency. To learn more about active versions checkout [active packages](../structure.md#active-packages).

Note that the new active version should already be installed.

### Examples
If we have a package `foo@1.3.7` which has as dependency `bar@4` and we want to switch this dependency to `bar@5`. Then we can use the following command:<br>
`pit switch-dependency foo@1.3.7 bar 5`
