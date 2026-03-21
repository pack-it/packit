use crate::{
    cli::display::{
        QuestionResponse, ask_user,
        logging::{error, warning},
    },
    config::{Config, Repository},
    installer::{
        builder::Builder,
        error::{InstallerError, Result},
        options::InstallerOptions,
        scripts::{self, ScriptData},
        symlinker::Symlinker,
        types::{OptionalPackageId, PackageId, PackageName, Version},
        unpack::unpack,
    },
    platforms::{Target, permissions, symlink},
    repositories::{
        error::RepositoryError,
        manager::RepositoryManager,
        provider,
        types::{Checksum, PackageMeta, PackageTarget, PackageVersionMeta, TargetBounds},
    },
    storage::{installed_package_version::InstalledPackageVersion, package_register::PackageRegister},
    utils::tree::Node,
};

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

/// The installer of Packit, managing the installation of packages on the system.
pub struct Installer<'a> {
    config: &'a Config,
    register: &'a mut PackageRegister,
    repository_manager: &'a RepositoryManager<'a>,
    options: InstallerOptions,
}

/// A label enum for the install/dependency tree
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DependencyTypes {
    Normal,
    Build,
}

/// A helper struct for the installer to move around nodes from the dependency trees
#[derive(Debug)]
pub struct InstallMeta {
    pub package_metadata: PackageMeta,
    pub version_metadata: PackageVersionMeta,
    pub repository_id: String,
    pub target_bounds: TargetBounds,
}

impl InstallMeta {
    pub fn new(
        package_metadata: PackageMeta,
        version_metadata: PackageVersionMeta,
        repository_id: String,
    ) -> std::result::Result<Self, RepositoryError> {
        let target_bounds = version_metadata.get_best_target(&Target::current())?;

        Ok(Self {
            package_metadata,
            version_metadata,
            repository_id,
            target_bounds,
        })
    }
}

impl<'a> Installer<'a> {
    /// Creates new installer
    pub fn new(
        config: &'a Config,
        register: &'a mut PackageRegister,
        repository_manager: &'a RepositoryManager,
        options: InstallerOptions,
    ) -> Self {
        Self {
            config,
            register,
            repository_manager,
            options,
        }
    }

    /// Installs the given package and its dependencies.
    pub fn install(&mut self, optional_id: &OptionalPackageId) -> Result<()> {
        // Check if we can write to the prefix directory
        if !self.can_write_prefix_dir()? {
            return Err(InstallerError::PermissionsError);
        }

        let (repository_id, package_metadata) = self.repository_manager.read_package(&optional_id.name)?;
        let latest_version = package_metadata.get_latest_version(&Target::current())?;

        // If the version isn't specified check if a package with this package name is already installed (otherwise a user can get two different version installed without knowing)
        if optional_id.version.is_none() {
            if let Some(package) = self.register.get_package(&optional_id.name) {
                if package.get_package_version(latest_version).is_some() {
                    println!("The latest version '{latest_version}' of '{optional_id}' is already installed.");
                    return Ok(());
                }

                let question = format!(
                    "The package '{optional_id}' is already installed, but a newer version '{latest_version}' is available. Do you wish to install the latest version as well?"
                );
                if ask_user(&question, QuestionResponse::No)?.is_no_or_invalid() {
                    return Ok(());
                }
            }
        }

        // Create a package id of the current package
        let package_id = optional_id.versioned_or(latest_version.clone());

        // Check if this package version is already installed
        if self.register.get_package_version(&package_id).is_some() {
            println!("Package '{}' already installed.", package_id);
            return Ok(());
        }

        // Get package version info
        let version_metadata = self.repository_manager.read_repo_package_version(&repository_id, &package_id)?;
        let target_bounds = version_metadata.get_best_target(&Target::current())?;

        // Create flattend dependency sequence
        let root_meta = InstallMeta {
            package_metadata,
            version_metadata,
            repository_id,
            target_bounds,
        };

        let mut installed_build_deps = Vec::new();
        match self.options.build_source {
            true => {
                let dependency_tree = Node::new_from_meta_build(&package_id, root_meta, self.repository_manager, self.register)?;
                self.install_nodes_build(&dependency_tree, &mut installed_build_deps)?;
            },
            false => {
                let mut dependency_tree = Node::new_from_meta(&package_id, root_meta, self.repository_manager, self.register)?;
                self.install_nodes(&mut dependency_tree, &mut installed_build_deps)?;
            },
        }

        if !self.options.keep_build {
            self.remove_build_dependencies(&installed_build_deps)?;
        }

        Ok(())
    }

    fn install_nodes(
        &mut self,
        node: &mut Node<Option<InstallMeta>, DependencyTypes>,
        installed_build_deps: &mut Vec<PackageId>,
    ) -> Result<()> {
        // Install childs first
        // TODO: Implement parallelization here
        for child in node.get_children_mut() {
            self.install_nodes(child, installed_build_deps)?;
        }

        // Get the value or return early if there is no value (package is already satisfied)
        let node_value = match node.get_value() {
            Some(value) => value,
            None => return Ok(()),
        };

        // Install the package with a prebuild if possible
        let revision = node_value.version_metadata.revisions.len() as u64;
        match self.repository_manager.get_prebuild_url(&node_value.repository_id, node.get_id(), revision, &Target::current()) {
            Ok(Some(_)) => {
                self.install_package(node_value, node.get_children_ids(Some(DependencyTypes::Normal)), true)?;
                return Ok(());
            },
            Ok(None) | Err(RepositoryError::RepositoryNotFoundError { .. }) => (),
            Err(e) => error!(e),
        }

        // Return early if the user doesn't want to build from source as alternative install method
        let question = format!(
            "Prebuild package for {} cannot be found, would you like to build from source instead?",
            node.get_id()
        );
        if ask_user(&question, QuestionResponse::Yes)?.is_no_or_invalid() {
            return Err(InstallerError::InstallationCanceled {
                reason: format!("package '{}' cannot be installed without building from source", node.get_id()),
            });
        }

        node.expand_node_with_build(self.repository_manager, self.register)?;
        self.install_nodes_build(node, installed_build_deps)?;

        Ok(())
    }

    fn install_nodes_build(
        &mut self,
        node: &Node<Option<InstallMeta>, DependencyTypes>,
        installed_build_deps: &mut Vec<PackageId>,
    ) -> Result<()> {
        // Install childs first
        // TODO: Implement parallelization here
        for child in node.get_children() {
            self.install_nodes_build(child, installed_build_deps)?;
        }

        // Get the value or return early if there is no value (package is already satisfied)
        let node_value = match node.get_value() {
            Some(value) => value,
            None => return Ok(()),
        };

        if *node.get_label() == DependencyTypes::Build {
            installed_build_deps.push(node.get_id().clone());
        }

        // Install the current node
        self.install_package(node_value, node.get_children_ids(Some(DependencyTypes::Normal)), false)?;

        Ok(())
    }

    fn install_package(&mut self, install_meta: &InstallMeta, children: HashSet<PackageId>, use_prebuild: bool) -> Result<()> {
        // Create the package id and install directory
        let package_id = PackageId::new(
            install_meta.package_metadata.name.clone(),
            install_meta.version_metadata.version.clone(),
        );
        let install_directory = self.config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());

        // Return early if the package has been installed by another node in the sequence (duplicates can exist)
        if self.register.get_package_version(&package_id).is_some() {
            return Ok(());
        }

        // Create install directory if it does not exist
        if !fs::exists(&install_directory)? {
            fs::create_dir_all(&install_directory)?;
        }

        let version_meta = &install_meta.version_metadata;
        let script_args = version_meta.get_script_args(&install_meta.target_bounds)?;

        // Download and run pre install script if it exists
        let script_path = version_meta.get_preinstall_script_path(&install_meta.target_bounds)?;
        let downloaded_script =
            scripts::download_script(self.repository_manager, &script_path, &package_id.name, &install_meta.repository_id)?;
        if let Some(script_file) = downloaded_script {
            let script_data = ScriptData::new(&script_file, &install_directory, &version_meta.version, self.config, &script_args);
            scripts::run_pre_script(&script_data, &install_directory)?;
        }

        // Get source repository for installed storage before actually installing package
        let source_repository = self.config.repositories.get(&install_meta.repository_id).expect("Expected repository in config");

        // Get the target information from the package version info
        let target = version_meta.get_target(&install_meta.target_bounds)?;

        // Get build version of package
        match use_prebuild {
            true => {
                let revision = install_meta.version_metadata.revisions.len() as u64;
                self.download_prebuild(&install_meta.repository_id, &package_id, revision, &install_directory)?
            },
            false => Builder::new(self.config, self.register, self.repository_manager).build(
                &install_meta.target_bounds,
                &install_meta.package_metadata,
                &version_meta,
                &install_meta.repository_id,
                &install_directory,
            )?,
        }

        // Set correct permissions for the installed package
        permissions::set_packit_permissions(&install_directory, self.config.multiuser, true)?;

        // Add and save package to installed storage toml
        self.register.add_package(
            &install_meta.package_metadata,
            &install_meta.version_metadata,
            children,
            source_repository,
            &install_directory,
            false,
            false,
            use_prebuild,
        )?;
        self.register.save_to(&PackageRegister::get_default_path(self.config))?;

        // Download and run post install script if it exists
        let script_path = version_meta.get_postinstall_script_path(&install_meta.target_bounds)?;
        let downloaded_script =
            scripts::download_script(self.repository_manager, &script_path, &package_id.name, &install_meta.repository_id)?;
        if let Some(script_file) = downloaded_script {
            let script_data = ScriptData::new(&script_file, &install_directory, &version_meta.version, self.config, &script_args);
            scripts::run_post_script(&script_data)?;
        }

        self.determine_active(install_meta, &package_id, target)?;

        // Download and run test script if it exists
        let script_path = version_meta.get_test_script_path(&install_meta.target_bounds)?;
        let downloaded_script =
            scripts::download_script(self.repository_manager, &script_path, &package_id.name, &install_meta.repository_id)?;
        if let Some(script_file) = downloaded_script {
            let script_data = ScriptData::new(&script_file, &install_directory, &version_meta.version, self.config, &script_args);
            scripts::run_test_script(&script_data)?;
        }

        Ok(())
    }

    fn determine_active(&mut self, install_meta: &InstallMeta, package_id: &PackageId, target: &PackageTarget) -> Result<()> {
        // Check if symlinking should be skipped
        let mut should_symlink = !self.options.skip_symlinking
            && !match target.skip_symlinking {
                Some(skip_symlinking) => skip_symlinking,
                None => install_meta.version_metadata.skip_symlinking,
            };

        let mut should_set_active = !self.options.skip_active;

        // Check if we have a previous active install
        if let Some(installed_package) = self.register.get_package(&package_id.name) {
            if installed_package.versions.len() > 1 {
                // Prompt user if the installed version is newer than the version currently installing
                if installed_package.active_version > install_meta.version_metadata.version {
                    let question = format!(
                        "A newer version ({}) of this package is currently active, do you want to change the active version to the older version ({})?",
                        installed_package.active_version, install_meta.version_metadata.version
                    );
                    should_set_active = ask_user(&question, QuestionResponse::No)?.is_yes();
                }

                // Prompt user if the installed version is not symlinked and we're not skipping symlinking
                if should_set_active && !installed_package.symlinked && should_symlink {
                    let question = format!(
                        "The current active version of '{}' ({}) is not symlinked, do you want to proceed with symlinking the newly installed version",
                        package_id.name, installed_package.active_version
                    );
                    should_symlink = ask_user(&question, QuestionResponse::No)?.is_yes();
                }

                // Show warning if the not symlinking but package was previously symlinked
                if should_set_active && installed_package.symlinked && !should_symlink {
                    warning!(
                        "The new active package version will not be symlinked, while the previously active version was symlinked. The package will not be automatically findable by your system anymore."
                    );
                }
            }
        }

        // If package is installed succesfully, set it to active
        if should_set_active {
            Symlinker::new(self.config).set_active(self.register, &package_id, should_symlink)?;
        }

        Ok(())
    }

    fn remove_build_dependencies(&mut self, installed: &Vec<PackageId>) -> Result<()> {
        // Loop through the installed build dependencies in reverse, because then the dependents
        // are uninstalled before the dependencies due to the install order.
        for package_id in installed.iter().rev() {
            let optional_id = &OptionalPackageId::from(package_id.clone());

            // Continue if the package is None (already removed in a previous iteration) or if it's also a dependency
            if self.register.get_package_version(package_id).is_some() && self.register.is_dependency(optional_id) {
                continue;
            }

            self.uninstall(optional_id)?;
        }

        Ok(())
    }

    fn download_prebuild(&self, repository_id: &str, package: &PackageId, revision: u64, destination_dir: impl AsRef<Path>) -> Result<()> {
        let (extension, bytes) = self.repository_manager.read_prebuild(repository_id, package, revision, &Target::current())?;
        let checksum = self.repository_manager.get_prebuild_checksum(repository_id, package, revision, &Target::current())?;

        // Calculate the checksum
        let calculated_checksum = Checksum::from_bytes(&bytes);

        // Check equality of checksum
        match checksum {
            Some(checksum) if checksum == calculated_checksum => (),
            _ => return Err(InstallerError::ChecksumError),
        }

        // Unpack the prebuild to the destination
        unpack(extension, bytes, &destination_dir)?;

        Ok(())
    }

    /// Uninstalls a package version if specified, otherwise it will uninstall the entire package directory.
    pub fn uninstall(&mut self, optional_id: &OptionalPackageId) -> Result<()> {
        // Check if we can write to the prefix directory
        if !self.can_write_prefix_dir()? {
            return Err(InstallerError::PermissionsError);
        }

        // Check if the current package to delete is a dependency, if so, give dependency error
        if self.register.is_dependency(optional_id) {
            return Err(InstallerError::DependencyError {
                package_name: optional_id.name.to_string(),
            });
        }

        // This determines the directory to remove. If there are multiple versions and the version is
        // specified only the specified version directory will be deleted. The entire package directory
        // is deleted if the version isn't specified or if the package directory only contains one version.
        match optional_id.versioned() {
            Some(package_id) => self.uninstall_single(&package_id)?,
            None => self.uninstall_all(&optional_id.name)?,
        }

        Ok(())
    }

    /// Checks if the directory exists. If so, it gets the remove directory for a package version, if there only exists one
    /// version it will return the package directory.
    fn uninstall_single(&mut self, package_id: &PackageId) -> Result<()> {
        // Return an existError if the package to uninstall doesn't exist
        if self.register.get_package_version(package_id).is_none() {
            return Err(InstallerError::PackageNotFound {
                package_name: package_id.name.to_string(),
                version: Some(package_id.version.to_string()),
            });
        }

        // Remove entire package directory if there is only one version, otherwise only remove the package version directory
        let installed_versions = self.register.get_all_package_versions(&package_id.name);
        let directory = match installed_versions.len() {
            1 => self.config.prefix_directory.join("packages").join(&package_id.name),
            _ => self.config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string()),
        };

        let installed_package = match self.register.get_package(&package_id.name) {
            Some(package) => package,
            None => {
                return Err(InstallerError::UnreachableError {
                    msg: "Package cannot be found eventhough it was found before".to_string(),
                });
            },
        };

        // Load source repository
        let repository = match installed_package.get_package_version(&package_id.version) {
            Some(package_version) => Repository::new(&package_version.source_repository_url, &package_version.source_repository_provider),
            None => {
                return Err(InstallerError::UnreachableError {
                    msg: "Package version cannot be found eventhough it was found before".to_string(),
                });
            },
        };

        // Run uninstall script
        self.run_uninstall_script(&repository, package_id, &directory)?;

        // Check if the package was symlinked
        if installed_package.active_version == package_id.version && installed_package.symlinked {
            Symlinker::new(self.config).remove_symlinks(Path::new(&self.config.prefix_directory), Path::new(&directory))?;
        }

        // Change active package when uninstalled package is currently active
        if installed_package.active_version == package_id.version {
            let mut other_versions = self.register.get_all_package_versions(&package_id.name);
            other_versions.retain(|x| x.package_id.version != package_id.version);
            other_versions.sort_by_key(|x| &x.package_id.version);

            if let Some(newest) = other_versions.last() {
                Symlinker::new(self.config).set_active(self.register, &newest.package_id.clone(), installed_package.symlinked)?;
            }
        }

        // Delete the determined directory
        self.remove_dir_all(&directory, &package_id.name)?;

        // Remove package from installed package toml
        self.register.remove_package_version(package_id);

        Ok(())
    }

    // Checks if there exists at least one version of the specified package. If so, it returns the package directory.
    fn uninstall_all(&mut self, package_name: &PackageName) -> Result<()> {
        let installed_versions = self.register.get_all_package_versions(package_name);

        // Ask the user if he/she wants to continue when version isn't specified and there are multiple versions installed
        let question = "Version is not specified, do you wish to uninstall all versions of this package?";
        if installed_versions.len() > 1 && ask_user(question, QuestionResponse::No)?.is_no_or_invalid() {
            println!("Canceled uninstall of package: {package_name}");
            return Ok(());
        }

        // Make sure at least one version exists
        if installed_versions.is_empty() {
            return Err(InstallerError::PackageNotFound {
                package_name: package_name.to_string(),
                version: None,
            });
        }

        // Path to the determined directory
        let directory = self.config.prefix_directory.join("packages").join(&package_name);

        // Remove active path symlink
        let active_path = Path::new(&self.config.prefix_directory).join("active").join(&package_name);
        match active_path.exists() {
            true => symlink::remove_symlink(&active_path)?,
            false => warning!("Active symlink did not exist, was the package even installed succesfully?"),
        }

        // Check if package was symlinked
        if let Some(package) = self.register.get_package(package_name) {
            if package.symlinked {
                Symlinker::new(self.config).remove_symlinks(Path::new(&self.config.prefix_directory), Path::new(&directory))?;
            }
        }

        // Run uninstall scripts for all versions
        for package_version in installed_versions {
            // Load source repository
            let repository = Repository::new(&package_version.source_repository_url, &package_version.source_repository_provider);

            // Run uninstall script
            self.run_uninstall_script(&repository, &package_version.package_id, &directory)?;
        }

        self.remove_dir_all(&directory, package_name)?;

        // Delete the installed package from toml
        self.register.remove_package(package_name);

        Ok(())
    }

    fn run_uninstall_script(&self, repository: &Repository, package_id: &PackageId, install_directory: &PathBuf) -> Result<()> {
        // Create repository provider for source repository
        let provider = match provider::create_metadata_provider(&repository) {
            Some(provider) => provider,
            None => {
                error!(msg: "Unable to create repository provider to retrieve uninstall script");
                return Ok(());
            },
        };

        // Load package version from source repository
        let package_version = match provider.read_package_version(&package_id.name, &package_id.version) {
            Ok(package_version) => package_version,
            Err(e) => {
                error!(e, "Unable to read package version from source repository");
                return Ok(());
            },
        };

        let target_bounds = package_version.get_best_target(&Target::current())?;

        // Get script data from package version metadata
        let script_path = package_version.get_uninstall_script_path(&target_bounds)?;

        // Download and run script
        if let Some(script_text) = provider.read_script(&package_id.name, &script_path)? {
            let script_path = scripts::write_script_to_tempfile(&script_text)?;

            // Run script
            let script_args = package_version.get_script_args(&target_bounds)?;
            let script_data = ScriptData::new(&script_path, &install_directory, &package_id.version, self.config, &script_args);
            scripts::run_uninstall_script(&script_data)?
        }

        Ok(())
    }

    /// Wraps around the fs::remove_dir_all to map its error.
    fn remove_dir_all(&self, directory: &PathBuf, package_name: &str) -> Result<()> {
        fs::remove_dir_all(directory).map_err(|e| InstallerError::UninstallError {
            package_name: package_name.into(),
            e,
        })?;

        Ok(())
    }

    fn can_write_prefix_dir(&self) -> Result<bool> {
        Ok(permissions::is_writable(&self.config.prefix_directory)?)
    }

    pub fn update(&mut self, optional_id: &OptionalPackageId, new_version: &Version) -> Result<()> {
        let old_package = self.get_specific_package_update(optional_id)?;
        let new_package_id = PackageId::new(old_package.package_id.name.clone(), new_version.clone());

        // Check if the new version is lower then the current
        if old_package.package_id.version > *new_version {
            return Err(InstallerError::VersionTooLowError {
                new_version: old_package.package_id.version.clone(),
            });
        }

        // Check if the new version is already installed
        if old_package.package_id.version == *new_version {
            return Err(InstallerError::AlreadyInstalledError {
                package_id: old_package.package_id.clone(),
            });
        }

        // Check if the new version still satisfies all dependents
        for dependent in &old_package.dependents {
            let (repository_id, _) = self.repository_manager.read_package(&dependent.name)?;
            let package_version_meta = self.repository_manager.read_repo_package_version(&repository_id, dependent)?;
            let dependency = match package_version_meta.dependencies.iter().find(|d| *d.get_name() == old_package.package_id.name) {
                Some(dependency) => dependency,
                None => {
                    warning!(
                        "Dependent is not a dependent of '{}' eventhough it should be.",
                        old_package.package_id
                    );
                    continue;
                },
            };

            if !dependency.satisfied(&new_package_id.name, new_version) {
                return Err(InstallerError::SatisfyError {
                    new_version: new_version.clone(),
                });
            }
        }

        // Use the old package reference before another borrow from self.install
        // Clone to avoid borrowing issues
        let dependents = old_package.dependents.clone();
        let old_package_id = old_package.package_id.clone();

        // Install the newer packager first
        self.install(&new_package_id.clone().into())?;

        // Add dependents to new_package
        let package_version = match self.register.get_package_version_mut(&new_package_id) {
            Some(package_version) => package_version,
            None => {
                // Theoretically unreachable
                return Err(InstallerError::UnreachableError {
                    msg: format!("New package version '{new_version}' cannot be retrieved from the register"),
                });
            },
        };

        // Set the dependents of the old package for the new package
        package_version.dependents = dependents.clone();

        // Change the register dependency entries to the new package version
        for package_id in &dependents {
            if let Some(dependent) = self.register.get_package_version_mut(package_id) {
                dependent.dependencies.remove(&old_package_id);
                dependent.dependencies.insert(new_package_id.clone());
            }
        }

        // Set the active and symlinked state for the new package (to the old package state)
        let package = self.register.get_package(&old_package_id.name).expect("Expected old package to still exist.");
        if package.active_version == *new_version {
            Symlinker::new(self.config).set_active(self.register, &new_package_id, package.symlinked)?;
        }

        println!("The new package version '{new_version}' has been succesfully installed, uninstalling the old version now.");

        // Remove the old package dependents, because the old package is no longer a dependency
        // Note that this is necessary before doing an uninstall.
        let old_package = self.register.get_package_version_mut(&old_package_id).expect("Expected old package to still exist.");
        old_package.dependents.clear();

        // Uninstall the package
        self.uninstall(&old_package_id.into())?;

        Ok(())
    }

    fn get_specific_package_update(&self, optional_id: &OptionalPackageId) -> Result<&InstalledPackageVersion> {
        // If a version is specified multiple installed versions are okay
        if let Some(package_id) = optional_id.versioned() {
            return Ok(
                self.register.get_package_version(&package_id).ok_or(InstallerError::PackageNotFound {
                    package_name: package_id.name.to_string(),
                    version: Some(package_id.version.to_string()),
                })?,
            );
        }

        // Get installed versions
        let installed_versions = self.register.get_all_package_versions(&optional_id.name);

        // Check if version is specified when multiple versions are installed
        if installed_versions.len() > 1 && optional_id.version.is_none() {
            return Err(InstallerError::SpecificityError);
        }

        // Get the installed package version and simultaniously check if any version of the package exists
        Ok(installed_versions.get(0).ok_or(InstallerError::PackageNotFound {
            package_name: optional_id.name.to_string(),
            version: Some("any".to_string()),
        })?)
    }
}
