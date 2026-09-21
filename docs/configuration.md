# Configuration

This file explains how to configure Packit with the `Config.toml`.

- [Location](#location)
- [Available fields](#available-fields)
    - [Repositories](#repositories)

## Location
The `Config.toml` contains the Packit configuration, it can be edited manually or with the [`pit config`](./commands/config.md) command. We recommend to use the `config` command, because it does extra checks, avoiding invalid configurations. Its location differs for each platform:
| Platform | Location                              |
| ---------| ------------------------------------- |
| Linux    | `/etc/packit`                         |
| MacOS    | `/Library/Application Support/packit` |
| Windows  | `C:\Program Files\packit`             |

## Available fields
All available fields in the config are listed below. 

| Field               | Explanation                                                                                                          |
| ------------------- | -------------------------------------------------------------------------------------------------------------------- |
| `prefix_directory`  | Defines the directory used for installing packages, see [prefix](./structure.md#prefix) for the defaults on each platform. |
| `repositories_rank` | Defines the order of repositories to search for a package.                                                           |
| `multiuser`         | True to run Packit in multiuser mode, false for single user mode.                                                    |

### Repositories
| Field                       | Explanation                                                                             |
| --------------------------- | --------------------------------------------------------------------------------------- |
| `url`                       | Defines the url of the repository.                                                      |
| `provider`                  | Defines the provider of the repository which can be `fs` or `web`, defaults to `web`.   |
| `prebuilds_url`             | Defines the url of the prebuilds repository for this package repository.                |
| `prebuilds_provider`        | Defines the provider of the prebuilds repository, defaults to `fs`.                     |
| `disable_prebuilds`         | True to disable prebuild usage for the repository, false to use prebuild if available.  |
| `compatible_repositories`   | A list of [compatible repository](./metadata.md#multiple-repositories) names.           |

## Example config
```
repositories_rank = ["core"]

[repositories]

[repositories.core]
provider = "web"
url = "https://raw.githubusercontent.com/pack-it/core/main/"
```
