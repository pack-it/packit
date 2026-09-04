# Config

The `config` command starts with:<br>
`pit config`

It has multiple sub-commands, which are explained below. If you want to learn more about the configuration structure, take a look at the [config](../structure.md#config).

## Show
#### `pit config show`
Shows the current configuration.

## Prefix
#### `pit config set-prefix <NEW-PREFIX>`
Sets the prefix to the given directory. This is currently not supported when packages are already installed, so this must be done directly after installing Packit.

Learn more about the [prefix](../structure.md#prefix).

## Multiuser
#### `pit config set-multiuser <MULTIUSER>`
Sets the multiuser setting to true or false. This is currently not supported when packages are already installed, so this must be done directly after installing Packit.

## Repositories
This sub-command is for configuring repositories. Learn more about [repositories here](../structure.md#repositories).

#### `pit config repositories list`
Lists all configured repositories.

---

#### `pit config repositories set-rank <REPOSITORY-ID> ...`
Sets the repositories rank in the config. Multiple `<REPOSITORY-ID>` values can be given for multiple repositories in the rank.

---

#### `pit config repositories add <ID> <URL> [PROVIDER] [--unchecked]`
Adds a new repository to the config. Also adds the new repository to the back of the repositories rank. If the `--unchecked` flag is given, the new repository is not checked for availability and compatibility.

---

#### `pit config repositories remove <ID>`
Removes a repository from the config. Also removes the repository from the repositories rank.

---

#### `pit config repositories set-url <ID> <URL> [PROVIDER] [--unchecked]`
Sets the URL of a repository in the config. If no provider is given, the existing provider is used. If the `--unchecked` flag is given, the new repository is not checked for availability and compatibility.

---

#### `pit config repositories set-prebuilds <ID> <PREBUILDS-URL> [PREBUILDS-PROVIDER]`
Sets the prebuilds URL of a repository in the config. If no provider is given, the existing provider is used.

---

#### `pit config repositories disable-prebuilds <ID> <VALUE> [--remove-urls]`
Enables or disables prebuild usage for a repository in the config. If the `--remove-urls` flag is given, the URLs are removed if `<VALUE>` is true.
