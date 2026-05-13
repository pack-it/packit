# Verifier

## How to use
To use the verifier you can use `pit check`, this will return all the issues that the verifier can find.
Important to note is the order of the issues. The most urgent issue will be listed first, meaning this issue 
needs to be solved before the other issues. The following issues can be a result of the first issue. 
To check issues for a specific package you can use `pit check <package-name>`.
<br>

To fix the issues the `pit fix` command can be used or `pit fix <package-name>` to fix issues for a specific package.

## All Checks
This is a list of checks which are currently implemented in the verifier. With a type and a short explanation for each check. 

| Check Type           | Type    | Short Explanation                                                                                                                                                                                                        |
|----------------------|---------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Permissions          | Initial | Checks the permissions of the prefix directory and all its subdirectories.                                                                                                                                               |
| Config Existence     | Initial | Checks if the `Config.toml` exists.                                                                                                                                                                                      |
| Config Syntax        | Initial | Checks if the `Config.toml` can be parsed.                                                                                                                                                                               |
| Register Existence   | Initial | Checks if the `Register.toml` exists.                                                                                                                                                                                   |
| Register Syntax      | Initial | Checks if the `Register.toml` can be parsed.                                                                                                                                                                            |
| Stray Directory      | General | Checks for directories which shouldn't be in the `prefix/packages` directory. This will be any directory which is empty or doesn't have the `<package-name>/<version>` structure.                                        |
| Packit Group         | General | Checks if the packit group exists if multiuser is enabled in the `Config.toml`.                                                                                                                                          |
| Package Existence    | Package | Checks if given package exists.                                                                                                                                                                                          |
| Storage Consistency  | Normal  | Checks if packages in the register also exist in the package storage in the prefix directory.                                                                                                                            |
| Register Consistency | Normal  | Checks if packages in storage also exist in the register.                                                                                                                                                                |
| Dependency Tree      | Normal  | Checks if the dependency tree is broken based on the dependencies specified in the register.                                                                                                                             |
| Alterations          | Normal  | Checks for alterations in packages using a checksum which is compared to the checksum from the pre-build.                                                                                                                |
| Missing Dependents   | Normal  | Checks for missing dependents of packages in the register.                                                                                                                                                               |
| Invalid Dependents   | Normal  | Checks for invalid dependents of a package in the register. Where an invalid dependent is a package which doesn't exist or a package which doesn't have the given package as a dependency.                               |
| Invalid Active       | Normal  | Checks if a package active version is invalid. It's invalid if: the link (destination) doesn't exist, if the package version doesn't exist or if the version specified in the register doesn't match the linked version. |
| Forbidden Link       | Normal  | Checks packages with a forbidden link. Where a forbidden link is a package which is symlinked while it shouldn't be according to the repository metadata.                                                                |
| Missing Link         | Normal  | Checks if symlinks are missing for packages.                                                                                                                                                                             |
| Missing Dependencies | Normal  | Checks for missing dependencies in packages. A dependency is missing if one of the dependencies specified in the repository metadata is not satisfied by the dependencies from the register.                             |
| Invalid Dependencies | Normal  | Checks for invalid dependencies in packages. A dependency from the register is invalid if it doesn't satisfy any of the dependencies specified in the repository metadata.                                               |

## Checks & Issues
The verifier has a variaty of checks available. A check can have dependencies, which are checks which have to be done before it. 
Each check returns an issue which is then passed along to the repairer.

The following types of checks exist:
- **Initial**: Checks critical features of Packit, happens before all other checks.
- **General**: Checks Packit, but no critical features. 
- **Package**: Checks for a specific package.
- **Normal**: Other non-critical package related checks.
