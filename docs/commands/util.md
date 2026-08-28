# Util

The `util` command starts with:<br>
`pit util`

It has multiple sub-commands, which are explained below. The `util` commands are meant for developing purposes or advanced users.

## Package
The `package` command has the following syntax:<br>
`pit util package <DESTINATION> [<PACKAGE-NAME>@<VERSION> ...] [--structured] [--all] [--exclude]`

The `package` command packages the specified packages into a prebuild and stores it in the `<DESTINATION>` directory, together with a checksum of the prebuild.

## Package flags
### `--structured`
The `--structured` flag organizes the packages into a prebuilds directory structure.

### `--all`
The `--all` flag will package all installed packages. This flag cannot be used when packages are specified.

### `--exclude`
The `--exclude` flag specifies packages to exclude when using the `--all` flag. This flag requires the `--all` flag.

## Checksum
The `checksum` command has the following syntax:<br>
`pit util checksum <URL>`

The command calculates the checksum of the file at the given URL. It also shows the size of the downloaded file in bytes.

## Portable repository
The `portable-repo` command has the following syntax:<br>
`pit util portable-repo <DESTINATION> <PACKAGE-NAME>@<VERSION> ... [--exclude-prebuilds] [--skip-dependency-resolution]`

The command generates a portable repository at the given destination, containing the specified packages. 

## Portable repository flags
### `--skip-dependency-resolution`
Normally all dependencies of the packages are added automatically. When the `--skip-dependency-resolution` flag is given, this step is skipped.

### `--exclude-prebuilds`
When the `--exclude-prebuilds` flag is given, prebuilds are not included in the portable repository and are not required to generate it.

## Metadata checks
The `meta-check` command has the following syntax:<br>
`pit util meta-check <REPOSITORY> [<PACKAGE-NAME> ...]`

The command checks the metadata from the given repository. The `<REPOSITORY>` argument can be a URL or a path to the repository or a repository id specified in `Config.toml`. If package names are given, only that package and the given repository are checked. If no package names are given, all packages specified in the repository's `index.toml` are checked.
