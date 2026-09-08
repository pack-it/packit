# Register and Local Metadata

This file explains the `Register.toml` file and the local metadata storage.

## `Register.toml`
The `Register.toml` file stores all information that Packit needs to keep track of which packages are installed, which dependencies they have and where they were installed from.

This file stores package information, for example the active version and the symlinked state.

It also stores package version information, which consists of installation path, dependencies, source metadata repository and other fields that describe the package installation.

## Local Metadata
The local metadata is stored in the `metadata` directory. This storage consists of the relevant package metadata that is required for all operations on packages that require the metadata. By storing this data locally, we do not rely on a remote repository. If the device is offline or the repository is not available anymore, the packages keep working as before.

This storage is structured in the same way as the `packages` directory, it contains directories for each package, with directories for each version of the package inside. Inside these version directories, the metadata for that version is stored. Scripts and other files that are needed are downloaded to this directory. The `metadata.toml` file stores all relevant metadata fields of the package.
