# Verifier

## How to use
To use the verifier you can use `pit check`, this will return all the issues that the verifier can find.
Important to note is the order of the issues. The most urgent issue will be listed first, meaning this issue 
needs to be solved before the other issues. The following issues **or errors** can be a result of the first issue(s). When issues are critical, errors are even to be expected.
To check issues for specific packages you can use `pit check <package-name>@<package-version> ...`.

To fix the issues the `pit fix` command can be used or `pit fix <package-name>@<package-version> ...` to fix issues for specific packages.

When packages are specified only those packages are checked when doing package related checks. Note that the initial checks and general (non-register package related checks) are still done as well. Also note that there is a small chance that a package specific check will miss an issue which indirectly causes problems for the specified package (for example if the issue has to do with a dependency of the specified package). Thats why we recommend using the more general check command (especially when doing a fix).
<br>

## Checks & Issues
The verifier has a variaty of checks available. A check can have dependencies, which are checks which have to be done before it. 
Each check returns an issue which is then passed along to the repairer.

The following categories of checks exist:
- **Initial**: Checks critical features of Packit, happens before all other checks (Config.toml, permissions, etc)
- **General**: Checks Packit, but no critical features. 
- **Package**: Does package related checks.

## All Checks
This is a list of checks which are currently implemented in the verifier. With a type and a short explanation for each check.

| Check Type          | Type    | Short Explanation                                                                                                                                                                                                        |
|---------------------|---------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Permissions         | Initial | Checks the permissions of the prefix directory and all its subdirectories.                                                                                                                                               |
| ConfigExistence     | Initial | Checks if the `Config.toml` exists.                                                                                                                                                                                      |
| ConfigSyntax        | Initial | Checks if the `Config.toml` can be parsed.                                                                                                                                                                               |
| RegisterExistence   | Initial | Checks if the `Register.toml` exists.                                                                                                                                                                                    |
| RegisterSyntax      | Initial | Checks if the `Register.toml` can be parsed.                                                                                                                                                                             |
| StrayDirectory      | General | Checks for directories which shouldn't be in the `prefix/packages` directory. This will be any directory which is empty or doesn't have the `<package-name>/<version>` structure.                                        |
| PackitGroup         | General | Checks if the packit group exists if multiuser is enabled in the `Config.toml`.                                                                                                                                          |
| StorageConsistency  | Package | Checks if packages in the register also exist in the package storage in the prefix directory.                                                                                                                            |
| RegisterConsistency | Package | Checks if packages in storage also exist in the register. Note that this is package related, but cannot only check the specified packages, because packages are based on what's found in storage.                        |
| DependencyTree      | Package | Checks if the dependency tree is broken based on the dependencies specified in the register.                                                                                                                             |
| Alterations         | Package | Checks for alterations in packages using a checksum which is compared to the checksum from the pre-build.                                                                                                                |
| MissingDependents   | Package | Checks for missing dependents of packages in the register.                                                                                                                                                               |
| InvalidDependents   | Package | Checks for invalid dependents of a package in the register. Where an invalid dependent is a package which doesn't exist or a package which doesn't have the given package as a dependency.                               |
| InvalidActive       | Package | Checks if a package active version is invalid. It's invalid if: the link (destination) doesn't exist, if the package version doesn't exist or if the version specified in the register doesn't match the linked version. |
| ForbiddenLink       | Package | Checks packages with a forbidden link. Where a forbidden link is a package which is symlinked while it shouldn't be according to the repository metadata.                                                                |
| MissingLink         | Package | Checks if symlinks are missing for packages.                                                                                                                                                                             |
| MissingDependencies | Package | Checks for missing dependencies in packages. A dependency is missing if one of the dependencies specified in the repository metadata is not satisfied by the dependencies from the register.                             |
| InvalidDependencies | Package | Checks for invalid dependencies in packages. A dependency from the register is invalid if it doesn't satisfy any of the dependencies specified in the repository metadata.                                               |


