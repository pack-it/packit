# Security

This file explains the current security measures, the overall security design and future improvements to the security model.

Packit is package manager and therefore resposible for installing packages on a users system that run under the permissions of this user. Packit cannot guarantee that software contained in a trusted package is harmless. By being open in the security design choiches of Packit, we hope to work towards a securer ecosystem.
<br>
This security model assumes that repositories and their maintainers are trusted. Users need to make sure they trust the repositories they are using, aswell as their maintainers.

If you found a vulnerability in Packit, please do not report it publicly. See the [Security Policy](https://github.com/pack-it/.github/blob/main/SECURITY.md) for more instructions about reporting a vulnerability.

## Packit Permissions
Packit uses a very Unix like approach to permissions, but ensured the same model also works on Windows.

The permission model consists of a single user and multi user mode. Single user modes means that a single user is owner of all installation files of Packit, only this user is then able to change the files. This means only this single user can install or uninstall packages. Other users are able to read the files and thus use the packages.
<br>
The multi user mode means that a group is owner of all Packit installation files, all users in this group are able to change the files and thus install and uninstall packages. All other users are still able to read the files and use the packages. This means that all users in this group should be trusted, as they can change the systemwide package installation.

On Unix we specifically recommend to not use `sudo` when running Packit, as this is not required. On Windows Administrator rights are needed to make sure symlinks can be created.
Running using `sudo` results in the scripts executing with root privileges, this is a serious risk as this could contain harmful code if using unsecure repositories.

## Metadata safety
The metadata of Packit describes the complete build process. It contains fields that describe file paths and scripts which are executed by Packit.

Important security measures for the metadata include:
- Ensuring build paths do not escape the build directory.
- Ensuring package sources are checked with a SHA-256 checksum to ensure integrity of the source. Note that this does not ensure the package itself is secure, if the creator of the package is malicious, this can still mean users install malicious software.
- Manual review of package scripts by the repository maintainers, to prevent mailicious code inside package scripts.

Packit relies on scripts that describe specific build and test logic for each package. These scripts are executed with the same privileges as the user executing Packit. This requires extra caution when the user has root privileges, since the scripts then also run with root privileges. The uninstall and test scripts are stored on the users system as part of the local metadata, for later use by Packit. The other scripts are not stored and only used during installation.

Build scripts run in an environment that is managed lightly by managing the environment variables, this helps with reproducibility of the source. Packit aims to make builds as reproducible as possible, to facilitate better integrity checking on builds.

Metadata repositories can contain test files, which are required for running package tests. After package installation they are stored on the users system as part of the local metadata. If a repository is compromised and contains malicious test files, these can end up on the users system.

Multiple repositories are allowed in the configuration. Packit has an extensive algorithm for deciding which repository should be used for the installation of a package. If the config contains a compromised repository, which has a malicious package with the same name of an important core package, this package could end up being chosen for installation on the system.

## Future improvements
This section lists all improvements that can be made to improve the security of Packit. Feel free to make notes about these improvements or help to get them into Packit.

### Script sandboxing
The scripts and especially the build script need to be run in a sandbox. A sandbox could restrict network access, access to system paths and monitor what a script wants to do.

It not only has a good impact on the security, it could also make the build environment more robust and reproducible.

### Repository trust
A specific repository trust mechanism, requiring users to explicitly trust a certain repository or only a specific list of packages from a repository. This can prevent package collision attacks, by narrowing down the list of packages that can be installed from each repository.

### Repository signing
Signing each file in a metadata repository and the prebuilds in a prebuild repository can help in checking that the metadata and prebuilds were not changed by any malicious actor. A valid signature ensures authenticity and integrity of the repository files and indicates the repository was signed by a trusted key.

### Automatic vulnerability detection
Automatically detecting vulnerabilities in the list of installed packages and notifying the user about it can help in ensuring the installations of a user remain secure. By checking the installed packages against an online database of vulnerabilities, the user can be notified immediately and the vulnerability can be fixed as fast as possible.
