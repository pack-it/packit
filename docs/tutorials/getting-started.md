# Getting Started

This tutorial is meant to get you started with using Packit on your system. It lists a few simple actions, explains what they do and how you can use them.

If you do not have Packit installed yet, please do so by running the install script as listed in the [README](https://github.com/pack-it/packit/blob/main/README.md#install).

## Installing a package

To install a package, you can use the `pit install` command, see [install](../commands/install.md).

For example, to install `htop`, run:
```sh
pit install htop
```

If you don't specify a version, Packit installs the latest available version.

You can also install a specific version:
```sh
pit install htop@3.4.1
```

## Uninstalling a package

To uninstall a package, you can use the `pit uninstall` command, see [uninstall](../commands/uninstall.md).

For example, to uninstall `htop`, run:
```sh
pit uninstall htop
```

If you don't specify a version, Packit uninstalls all versions of the package.

You can also uninstall a specific version:
```sh
pit uninstall htop@3.4.1
```

## Updating packages

To check if updates are available, you can use the `pit list` command with the `--updatables` flag:
```sh
pit list --updatables
```

This will show all packages that can be updated. To update all packages, run:
```sh
pit update --all
```

You can also update a specific package:
```sh
pit update htop
```

This will update htop to the latest version available.

## Listing installed packages

To see all packages that are installed, use the `pit list` command, see [list](../commands/list.md).

This command returns a list of all installed packages and their versions.

To see information about a specific package, use the `pit info` command, see [info](../commands/info.md).

For example, to see information about `htop`, run:
```sh
pit info htop
```

This shows generic information about the package `htop`. You can also see information about a specific version:
```sh
pit info htop@3.4.1
```

## Searching for a package

To search for information about a package, use the `pit search` command, see [search](../commands/search.md).
This command will look for the package in all configured repositories.

For example, to search for `htop`, run:
```sh
pit search htop
```

This shows generic information about the package `htop`. You can also search for information about a specific version:
```sh
pit search htop@3.4.1
```

## Changing active version or link state

Packit allows you to switch the active version of a package, and to define if a package should be available in your PATH.

This can be useful when you want to keep a package installed, but avoid conflicts with another installation already available on your system.

To switch the active version of a package, use the `pit switch` command, see [switch](../commands/switch.md).

For example, to switch `htop` to version `3.5.0`, run:
```sh
pit switch htop 3.5.0
```

Note that this requires `htop` version `3.5.0` to be installed.

To remove a package from your PATH, use the `pit unlink` command, see [unlink](../commands/unlink.md).
```sh
pit unlink htop
```

This will result in package `htop` not being available in your PATH anymore, therefore you cannot run it anymore.

To restore `htop` in the PATH, use the `pit link` command, see [link](../commands/link.md).
```sh
pit link htop
```

In short: `pit switch` changes which installed version is active. `pit link` and `pit unlink` control whether the package is available in your PATH.

## Adding a new repository to your config

Packit allows you to add multiple repositories to your configuration.

This can be useful when you want to install packages that are not distributed by the Packit team itself.

> [!WARNING]  
> Third party repositories can contain harmful content and should therefore be trusted before adding it to the configuration of Packit.

To add a repository to your config, use the `pit config repositories add` command, see [config](../commands/config.md).

For example, to add the `core` repository, run:
```sh
pit config repositories add core https://raw.githubusercontent.com/pack-it/core/main
```

Note that `core` is the identifier of the new repository and the next argument is the URL to the repository.

## Checking your installation

Packit has an extensive integrity module that can check if your Packit installation has problems.

To check for issues, you can use the `pit check` command, see [check](../commands/check.md).

If this command shows issues, you can use the `pit fix` command to fix the issues, see [fix](../commands/fix.md).

## Next steps

You now know the basics of installing, updating, searching and uninstalling packages with Packit.

For more detailed descriptions of all commands, see the documentation of the commands.

If you want to learn more about the structure of Packit itself, you can take a look at the [structure](../structure.md) and [installation process](../installation-process.md) documentation files.
