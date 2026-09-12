# Security

This file explains the current security measures, the overall security design and future improvements to the security model.

Packit is package manager and therefore resposible for installing packages on a users system that run under the permissions of this user. Packit cannot guarantee that software contained in a trusted package is harmless. By being open in the security design choiches of Packit, we hope to work towards a securer ecosystem.
<br>
This security model assumes that repositories and their maintainers are trusted. Users need to make sure they trust the repositories they are using, aswell as their maintainers.

## Packit Permissions
Packit uses a very Unix like approach to permissions, but ensured the same model also works on Windows.

The permission model consists of a single user and multi user mode. Single user modes means that a single user is owner of all installation files of Packit, only this user is then able to change the files. This means only this single user can install or uninstall packages. Other users are able to read the files and thus use the packages.
<br>
The multi user mode means that a group is owner of all Packit installation files, all users in this group are able to change the files and thus install and uninstall packages. All other users are still able to read the files and use the packages.

On Unix we specifically recommend to not use `sudo` when running Packit, as this is not required. On Windows you need Administrator rights to make sure symlinks can be created.

## Metadata safety
The metadata of Packit describes the complete build process. It contains fields that describe file paths and scripts which are executed by Packit.

Important security measures for the metadata include:
- Ensuring build paths do not escape the build directory.
- Ensuring package sources are checked with a SHA-256 checksum, to detect tampering with source distribution.
- Manual review of package scripts by the repository maintainers, to ensure no mailicious code is inside these scripts.

## Future improvements
This section lists all improvements that can be made to improve the security of Packit. Feel free to make notes about these improvements or help to get them into Packit.

### Script sandboxing
The scripts and especially the build script need to be run in a sandbox. A sandbox could restrict network access, access to system paths and monitor what a script wants to do.

It not only has a good impact on the security, it could also make the build environment more robust and reproducible.

### Repository trust
A specific repository trust mechanism, requiring users to explicitly trust a certain repository or only a specific list of packages from a repository. This can prevent package collision attacks, by narrowing down the list of packages that can be installed from each repository.

### Repository signing
Signing each file in a metadata repository and the prebuilds in a prebuild repository can help in checking that the metadata and prebuilds were not changed by any malicious actor. The signature would prove the trusted maintainers changed and approved the files.

### Automatic vulnerability detection
Automatically detecting vulnerabilities in the list of installed packages and notifying the user about it can help in ensuring the installations of a user remain secure. By checking the installed packages against an online database of vulnerabilities, the user can be notified immediately and the vulnerability can be fixed as fast as possible.
