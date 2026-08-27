# Config

The `config` command starts with:<br>
`pit config`

It has multiple sub-commands, which are explained below. If you want to learn more about the config structure, checkout [config](../structure.md#config).

## Show

#### `pit config show`
Shows the current configuration.

## Prefix

#### `pit config set-prefix <NEW-PREFIX>`
Sets the prefix to the given directory. Currently not supported when packages are already installed. So this must be done directly after installing Packit.

Learn more about the [prefix](../structure.md#prefix).

## Multiuser

#### `pit config set-multiuser <MULTIUSER>`
Sets the multiuser setting to true or false. Currently not supported when packages are already installed. So this must be done directly after installing Packit.

## Repositories

This sub-command is for config repositories, find out more about [repositories here](../structure.md#repositories).

#### `pit config repositories list`
Lists all configured repositories.

---

#### `pit config repositories set-rank <REPOSITORY-ID> ...`
Sets the repositories rank in the config. Multiple `<REPOSITORY-ID>` can be given for multiple repositories in the rank.

---

#### `pit config repositories add <ID> <URL> [PROVIDER] [--unchecked]`
Adds a new repository to the config. Also adds the new repository to the back of the repositories rank. If the `--unchecked` flag is given, the new repository is not checked for availability and compatibility.

---

#### `pit config repositories remove <ID>`
Removes a repository from the config. Also removes the repository from the repositories rank.

---

#### `pit config repositories set-url <ID> <URL> [PROVIDER] [--unchecked]`
Sets the url of a repository in the config. If no provider is given, the old provider is used. If the `--unchecked` flag is given, the new repository is not checked for availability and compatibility.

---

#### `pit config repositories set-prebuilds <ID> <PREBUILDS-URL> [PREBUILDS-PROVIDER]`
Sets the prebuilds url of a repository in the config. If no provider is given, the old provider is used.

---

#### `pit config repositories disable-prebuilds <ID> <VALUE> [--remove-urls]`
Disables or enables the prebuilds url of a repository in the config. If the `--remove-urls` flag is given, the urls are removed if `<VALUE>` is true.