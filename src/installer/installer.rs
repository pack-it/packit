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
        types::{Dependency, OptionalPackageId, PackageId},
        unpack::unpack,
    },
    platforms::{Target, symlink},
    repositories::{
        error::RepositoryError,
        manager::RepositoryManager,
        provider,
        types::{Checksum, PackageMeta, PackageTarget, PackageVersionMeta, TargetBounds},
    },
    storage::package_register::PackageRegister,
    utils::tree::Node,
};

use std::{
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
    pub fn new(manager: &RepositoryManager, dependency: &Dependency) -> std::result::Result<Self, RepositoryError> {
        // Get all the data to create a dependency node
        let (repository_id, package_metadata) = manager.read_package(dependency.get_name())?;
        let target_bounds = package_metadata.get_best_target(&Target::current())?;
        let version = package_metadata.get_latest_dependency_version(&dependency)?;
        let dependency_id = dependency.to_package_id(version);
        let version_metadata = manager.read_repo_package_version(&repository_id, &dependency_id)?;

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
        let target_bounds = package_metadata.get_best_target(&Target::current())?;
        let latest_version = package_metadata.get_latest_version(&target_bounds)?;

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

        // Create flattend dependency sequence
        let root_meta = InstallMeta {
            package_metadata,
            version_metadata,
            repository_id,
            target_bounds,
        };

        let mut tree = match self.options.build_source {
            true => {
                let dependency_tree: Node<InstallMeta, DependencyTypes> =
                    Node::new_from_meta_build(&package_id, root_meta, self.repository_manager)?;
                self.install_nodes_build(&dependency_tree)?;
                dependency_tree
            },
            false => {
                let mut dependency_tree = Node::new_from_meta(&package_id, root_meta, self.repository_manager)?;
                self.install_nodes(&mut dependency_tree)?;
                dependency_tree
            },
        };

        if !self.options.keep_build {
            self.remove_build_dependencies(&mut tree)?;
        }

        Ok(())
    }

    fn install_nodes(&mut self, node: &mut Node<InstallMeta, DependencyTypes>) -> Result<()> {
        // Install childs first
        // TODO: Implement parallelization here
        for child in node.get_children_mut() {
            self.install_nodes(child)?;
        }

        let node_value = node.get_value();
        let revision = node_value.version_metadata.revisions.len() as u64;

        // Install the package with a prebuild if possible
        match self.repository_manager.get_prebuild_url(&node_value.repository_id, node.get_id(), revision, &Target::current()) {
            Ok(Some(_)) => {
                self.install_package(node, true)?;
                return Ok(());
            },
            Ok(None) | Err(RepositoryError::RepositoryNotFoundError { .. }) => (),
            Err(e) => error!(e),
        }

        // Return early if the user doesn't want to build from source as alternative install method
        // TODO: Look at this, now it's possible that one of the package dependencies just isn't installed
        let question = format!(
            "Prebuild package for {} cannot be found, would you like to build from source instead?",
            node.get_id()
        );
        if ask_user(&question, QuestionResponse::Yes)?.is_no() {
            return Ok(());
        }

        node.expand_node_with_build(self.repository_manager)?;
        self.install_nodes_build(node)?;

        Ok(())
    }

    fn install_nodes_build(&mut self, node: &Node<InstallMeta, DependencyTypes>) -> Result<()> {
        // Install childs first
        // TODO: Implement parallelization here
        for child in node.get_children() {
            self.install_nodes_build(child)?;
        }

        // Install the current node
        self.install_package(node, false)?;

        Ok(())
    }

    fn install_package(&mut self, node: &Node<InstallMeta, DependencyTypes>, use_prebuild: bool) -> Result<()> {
        let node_value = node.get_value();

        // Create the package id and install directory
        let package_id = PackageId::new(&node_value.package_metadata.name, node_value.version_metadata.version.clone())?;
        let install_directory = self.config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());

        // Return early if the package has been installed by another node in the sequence (duplicates can exist)
        if self.register.get_package_version(&package_id).is_some() {
            return Ok(());
        }

        // Create install directory if it does not exist
        if !fs::exists(&install_directory)? {
            fs::create_dir_all(&install_directory)?;
        }

        let version_meta = &node_value.version_metadata;
        let script_args = version_meta.get_script_args(&node_value.target_bounds)?;

        // Download and run pre install script if it exists
        let script_path = version_meta.get_preinstall_script_path(&node_value.target_bounds)?;
        let downloaded_script =
            scripts::download_script(self.repository_manager, &script_path, &package_id.name, &node_value.repository_id)?;
        if let Some(script_file) = downloaded_script {
            let script_data = ScriptData::new(&script_file, &install_directory, &version_meta.version, self.config, &script_args);
            scripts::run_pre_script(&script_data, &install_directory)?;
        }

        // Get source repository for installed storage before actually installing package
        let source_repository = self.config.repositories.get(&node_value.repository_id).expect("Expected repository in config");

        // Get the target information from the package version info
        let target = version_meta.get_target(&node_value.target_bounds)?;

        // Get build version of package
        match use_prebuild {
            true => {
                let revision = node_value.version_metadata.revisions.len() as u64;
                self.download_prebuild(&node_value.repository_id, &package_id, revision, &install_directory)?
            },
            false => Builder::new(self.config, self.register, self.repository_manager).build(
                &node_value.target_bounds,
                &node_value.package_metadata,
                &version_meta,
                &node_value.repository_id,
                &install_directory,
            )?,
        }

        // Add and save package to installed storage toml
        self.register.add_package(
            &node_value.package_metadata,
            &node_value.version_metadata,
            &node.get_children_ids(Some(DependencyTypes::Normal)),
            source_repository,
            &install_directory,
            false,
            false,
        )?;
        self.register.save_to(&PackageRegister::get_default_path(self.config))?;

        // Download and run post install script if it exists
        let script_path = version_meta.get_postinstall_script_path(&node_value.target_bounds)?;
        let downloaded_script =
            scripts::download_script(self.repository_manager, &script_path, &package_id.name, &node_value.repository_id)?;
        if let Some(script_file) = downloaded_script {
            let script_data = ScriptData::new(&script_file, &install_directory, &version_meta.version, self.config, &script_args);
            scripts::run_post_script(&script_data)?;
        }

        self.determine_active(node_value, &package_id, target)?;

        // Download and run test script if it exists
        let script_path = version_meta.get_test_script_path(&node_value.target_bounds)?;
        let downloaded_script =
            scripts::download_script(self.repository_manager, &script_path, &package_id.name, &node_value.repository_id)?;
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

    fn remove_build_dependencies(&mut self, node: &mut Node<InstallMeta, DependencyTypes>) -> Result<()> {
        // Remove current before childs (parents before children)
        let optional_id = &OptionalPackageId::from(node.get_id().clone());
        if self.register.get_package_version(node.get_id()).is_some()
            && !self.register.is_dependency(optional_id)
            && *node.get_label() == DependencyTypes::Build
        {
            self.uninstall(optional_id)?;
        }

        // Remove children
        for child in node.get_children_mut() {
            self.remove_build_dependencies(child)?;
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
                package_name: optional_id.name.clone(),
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
                warning!("Package cannot be found eventhough it was found before, should be unreachable.");
                return Ok(());
            },
        };

        // Load source repository
        let repository = match installed_package.get_package_version(&package_id.version) {
            Some(package_version) => Repository::new(&package_version.source_repository_url, &package_version.source_repository_provider),
            None => {
                warning!("Package version cannot be found eventhough it was found before, should be unreachable.");
                return Ok(());
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
    fn uninstall_all(&mut self, package_name: &str) -> Result<()> {
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
                package_name: package_name.into(),
                version: None,
            });
        }

        // Path to the determined directory
        let directory = self.config.prefix_directory.join("packages").join(package_name);

        // Check if package was symlinked
        if let Some(package) = self.register.get_package(package_name) {
            if package.symlinked {
                Symlinker::new(self.config).remove_symlinks(Path::new(&self.config.prefix_directory), Path::new(&directory))?;
            }
        }

        // Remove active path symlink
        let active_path = Path::new(&self.config.prefix_directory).join("active").join(&package_name);
        match active_path.exists() {
            true => symlink::remove_symlink(&active_path)?,
            false => warning!("Active symlink did not exist, was the package even installed succesfully?"),
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
        if !fs::exists(&self.config.prefix_directory)? {
            return Ok(false);
        }

        let metadata = fs::metadata(&self.config.prefix_directory)?;
        let permissions = metadata.permissions();

        // TODO: Use something else then readonly, because it can be different for super user and group
        Ok(!permissions.readonly())
    }
}
