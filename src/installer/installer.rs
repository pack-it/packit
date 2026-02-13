use crate::{
    cli::display::{
        ask_user,
        logging::{debug, warning},
        QuestionResponse,
    },
    config::Config,
    installer::{
        builder::Builder,
        error::{InstallerError, Result},
        options::InstallerOptions,
        scripts::{self, ScriptData},
        symlinker::Symlinker,
        types::{Dependency, OptionalPackageId, PackageId, Version},
    },
    platforms::{symlink, TARGET_ARCHITECTURE},
    repositories::{
        manager::RepositoryManager,
        types::{PackageMeta, PackageTarget, PackageVersionMeta},
    },
    storage::package_register::PackageRegister,
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

/// A helper struct for the installer to move around nodes from the dependency trees
struct DependencyNode {
    package_metadata: PackageMeta,
    version_metadata: PackageVersionMeta,
    repository_id: String,
    dependencies: HashSet<PackageId>,
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
    pub fn install(&mut self, package_id: &OptionalPackageId) -> Result<()> {
        // Check if we can write to the prefix directory
        if !self.can_write_prefix_dir()? {
            return Err(InstallerError::PermissionsError);
        }

        let (repository_id, package_metadata) = self.repository_manager.read_package(&package_id.name)?;

        // Use the latest version if the version isn't specified
        let version = match &package_id.version {
            Some(version) => version,
            None => package_metadata.get_latest_version(TARGET_ARCHITECTURE)?,
        };

        // Create a package id of the current package
        let package_id = PackageId::new(&package_id.name, &version);

        // Check if this package version is already installed
        if self.register.get_package_version(&package_id).is_some() {
            println!("Package '{}' already installed.", package_id);
            return Ok(());
        }

        // Get package version info
        let version_metadata = self.repository_manager.read_repo_package_version(&repository_id, &package_id)?;

        // Create flattend dependency sequence
        let mut root = DependencyNode {
            package_metadata,
            version_metadata,
            repository_id,
            dependencies: HashSet::new(),
        };
        let mut flattened_dependencies = self.get_flattened_dependencies(&mut root)?;
        flattened_dependencies.insert(0, root);

        self.install_nodes(&mut flattened_dependencies)?;

        Ok(())
    }

    fn install_nodes(&mut self, nodes: &mut Vec<DependencyNode>) -> Result<()> {
        let mut all_dependencies = Vec::new();
        let mut dependency_ids = Vec::new();
        for node in nodes {
            let package_id = PackageId::new(&node.package_metadata.name, &node.version_metadata.version);
            if !self.options.build_source {
                let prebuild_url = self.repository_manager.get_prebuild_url(&node.repository_id, &package_id);

                // Install the package with a prebuild if possible
                if let Some(url) = prebuild_url {
                    self.install_package(node, Some(&url))?;
                    continue;
                }

                // Return early if the user doesn't want to build from source as alternative install method
                let question = format!("Prebuild package for {package_id} cannot be found, would you like to build from source instead?");
                if ask_user(&question, QuestionResponse::Yes)?.is_no() {
                    return Ok(());
                }
            }

            // Get and install the build dependencies first
            let build_dependencies = self.get_flattened_build_dependencies(node)?;
            for build_node in build_dependencies.iter().rev() {
                self.install_package(build_node, None)?;
            }

            // Build the current dependency node
            self.install_package(node, None)?;

            // Save the current build dependencies and the current node id
            all_dependencies.extend(build_dependencies);
            dependency_ids.push(package_id);
        }

        // Remove build dependencies if --keep-build not used
        if !self.options.keep_build {
            self.remove_build_dependencies(&all_dependencies, &dependency_ids)?;
        }

        Ok(())
    }

    fn install_package(&mut self, node: &DependencyNode, url: Option<&str>) -> Result<()> {
        // Create the package id and install directory
        let package_id = PackageId::new(&node.package_metadata.name, &node.version_metadata.version);
        let install_directory = self.config.prefix_directory.join("packages").join(&package_id.name).join(package_id.version.to_string());

        // Return early if the package has been installed by another node in the sequence (duplicates can exist)
        if self.register.get_package_version(&package_id).is_some() {
            return Ok(());
        }

        // Create install directory if it does not exist
        if !fs::exists(&install_directory)? {
            fs::create_dir_all(&install_directory)?;
        }

        let version_meta = &node.version_metadata;

        let script_args = version_meta.get_script_args(TARGET_ARCHITECTURE)?;

        // Download and run pre install script if it exists
        let script_path = version_meta.get_preinstall_script_path(TARGET_ARCHITECTURE)?;
        let downloaded_script = scripts::download_script(self.repository_manager, &script_path, &package_id.name, &node.repository_id)?;
        if let Some(script_file) = downloaded_script {
            let script_data = ScriptData::new(&script_file, &install_directory, &version_meta.version, self.config, &script_args);
            scripts::run_pre_script(&script_data, &install_directory)?;
        }

        // Get source repository for installed storage before actually installing package
        let source_repository = self.config.repositories.get(&node.repository_id).expect("Expected repository in config");

        // Get the target information from the package version info
        let target = version_meta.get_target(TARGET_ARCHITECTURE)?;

        // Get build version of package
        match url {
            Some(url) => self.download_prebuild(&url, &install_directory)?,
            None => Builder::new(self.config, self.register, self.repository_manager).build(
                &node.package_metadata,
                &version_meta,
                &node.repository_id,
                &install_directory,
            )?,
        }

        // Add and save package to installed storage toml
        self.register.add_package(
            &node.package_metadata,
            &node.version_metadata,
            &node.dependencies,
            source_repository,
            &install_directory,
            false,
            false,
        );
        self.register.save_to(&PackageRegister::get_default_path())?;

        // Download and run post install script if it exists
        let script_path = version_meta.get_postinstall_script_path(TARGET_ARCHITECTURE)?;
        let downloaded_script = scripts::download_script(self.repository_manager, &script_path, &package_id.name, &node.repository_id)?;
        if let Some(script_file) = downloaded_script {
            let script_data = ScriptData::new(&script_file, &install_directory, &version_meta.version, self.config, &script_args);
            scripts::run_post_script(&script_data)?;
        }

        self.determine_active(node, &package_id, target)?;

        // Download and run test script if it exists
        let script_path = version_meta.get_test_script_path(TARGET_ARCHITECTURE)?;
        let downloaded_script = scripts::download_script(self.repository_manager, &script_path, &package_id.name, &node.repository_id)?;
        if let Some(script_file) = downloaded_script {
            let script_data = ScriptData::new(&script_file, &install_directory, &version_meta.version, self.config, &script_args);
            scripts::run_test_script(&script_data)?;
        }

        Ok(())
    }

    fn get_flattened_dependencies(&self, parent_node: &mut DependencyNode) -> Result<Vec<DependencyNode>> {
        let mut dependencies: Vec<DependencyNode> = Vec::new();
        let target = parent_node.version_metadata.get_target(TARGET_ARCHITECTURE)?;
        for dependency in parent_node.version_metadata.dependencies.iter().chain(target.dependencies.iter()) {
            if let Some(package) = self.register.get_satisfying_package(dependency) {
                // First add the package as a dependency to the parent node
                parent_node.dependencies.insert(package.package_id.clone());

                debug!("Dependency '{}' already satisfied, continuing", dependency.get_name());
                continue;
            }

            // Get all the data to create a dependency node
            let version = self.get_latest_dependency_version(&dependency)?;
            let dependency_id = PackageId::new(dependency.get_name(), &version);
            let (repository_id, package_metadata) = self.repository_manager.read_package(dependency.get_name())?;
            let version_metadata = self.repository_manager.read_repo_package_version(&repository_id, &dependency_id)?;
            let mut node = DependencyNode {
                package_metadata,
                version_metadata,
                repository_id,
                dependencies: HashSet::new(),
            };

            // Get all the sub dependencies and add them to the current dependencies as well (after the current node)
            let sub_dependencies = self.get_flattened_dependencies(&mut node)?;
            dependencies.push(node);
            dependencies.extend(sub_dependencies);

            // Add the dependency id to the parent node
            parent_node.dependencies.insert(dependency_id.clone());
        }

        Ok(dependencies)
    }

    fn get_flattened_build_dependencies(&self, parent_node: &mut DependencyNode) -> Result<Vec<DependencyNode>> {
        let mut dependencies = Vec::new();
        let target = parent_node.version_metadata.get_target(TARGET_ARCHITECTURE)?;

        // Get all dependencies from the parent (dependencies from build dependencies are build dependencies from the original package to install)
        let all_dependencies = parent_node
            .version_metadata
            .build_dependencies
            .iter()
            .chain(target.build_dependencies.iter())
            .chain(parent_node.version_metadata.dependencies.iter())
            .chain(target.dependencies.iter());

        // Get the index where build dependencies and dependencies are divided
        let boundary_index = parent_node.version_metadata.build_dependencies.len() + target.build_dependencies.len();

        // Loop over all (build) dependencies
        for (index, dependency) in all_dependencies.enumerate() {
            if let Some(package) = self.register.get_satisfying_package(dependency) {
                // First add the package as a dependency to the parent node
                // Only add if the package is a 'normal' dependency
                if index >= boundary_index {
                    parent_node.dependencies.insert(package.package_id.clone());
                }

                debug!("Dependency '{}' already satisfied, continuing", dependency.get_name());
                continue;
            }

            // Get all the data to create a dependency node
            let version = self.get_latest_dependency_version(&dependency)?;
            let dependency_id = PackageId::new(dependency.get_name(), &version);
            let (repository_id, package_metadata) = self.repository_manager.read_package(dependency.get_name())?;
            let dependency_package = self.repository_manager.read_repo_package_version(&repository_id, &dependency_id)?;
            let mut node = DependencyNode {
                package_metadata,
                version_metadata: dependency_package,
                repository_id,
                dependencies: HashSet::new(),
            };

            let sub_dependencies = self.get_flattened_build_dependencies(&mut node)?;
            dependencies.push(node);
            dependencies.extend(sub_dependencies);

            // Add the dependency id to the parent node (if the dependency is not a build dependency)
            // Only add if the package is a 'normal' dependency
            if index >= boundary_index {
                parent_node.dependencies.insert(dependency_id.clone());
            }
        }

        Ok(dependencies)
    }

    fn determine_active(&mut self, node: &DependencyNode, package_id: &PackageId, target: &PackageTarget) -> Result<()> {
        // Check if symlinking should be skipped
        let mut should_symlink = !self.options.skip_symlinking
            && !match target.skip_symlinking {
                Some(skip_symlinking) => skip_symlinking,
                None => node.version_metadata.skip_symlinking,
            };

        let mut should_set_active = !self.options.skip_active;

        // Check if we have a previous active install
        if let Some(installed_package) = self.register.get_package(&package_id.name) {
            if installed_package.versions.len() > 1 {
                // Prompt user if the installed version is newer than the version currently installing
                if installed_package.active_version > node.version_metadata.version {
                    let question = format!(
                            "A newer version ({}) of this package is currently active, do you want to change the active version to the older version ({})?", 
                            installed_package.active_version, node.version_metadata.version
                        );
                    should_set_active = ask_user(&question, QuestionResponse::No)?.is_yes();
                }

                // Prompt user if the installed version is not symlinked and we're not skipping symlinking
                if should_set_active && !installed_package.symlinked && should_symlink {
                    let question = format!("The current active version of '{}' ({}) is not symlinked, do you want to proceed with symlinking the newly installed version", package_id.name, installed_package.active_version);
                    should_symlink = ask_user(&question, QuestionResponse::No)?.is_yes();
                }

                // Show warning if the not symlinking but package was previously symlinked
                if should_set_active && installed_package.symlinked && !should_symlink {
                    warning!("The new active package version will not be symlinked, while the previously active version was symlinked. The package will not be automatically findable by your system anymore.");
                }
            }
        }

        // If package is installed succesfully, set it to active
        if should_set_active {
            Symlinker::new(self.config).set_active(self.register, &package_id, should_symlink)?;
        }

        Ok(())
    }

    fn remove_build_dependencies(&mut self, build_dependencies: &Vec<DependencyNode>, dependencies: &Vec<PackageId>) -> Result<()> {
        for build_dependency in build_dependencies {
            // Get name and version from the current build dependency
            let name = &build_dependency.package_metadata.name;
            let version = &build_dependency.version_metadata.version;

            // Continue if the build dependency is also a dependency in the dependency sequence
            if dependencies.iter().any(|d| d.name == *name && d.version == *version) {
                continue;
            }

            // Continue if it's still a dependency somewhere else in the build dependency sequence (because of the DFS)
            if self.register.is_dependency(name, Some(version)) {
                continue;
            }

            self.uninstall(&OptionalPackageId::new_versioned(
                &build_dependency.package_metadata.name,
                &build_dependency.version_metadata.version,
            ))?;
        }

        Ok(())
    }

    fn download_prebuild(&self, prebuild_url: &str, destination_dir: impl AsRef<Path>) -> Result<()> {
        todo!()
    }

    /// Uninstalls a package version if specified, otherwise it will uninstall the entire package directory.
    pub fn uninstall(&mut self, package_id: &OptionalPackageId) -> Result<()> {
        // Check if we can write to the prefix directory
        if !self.can_write_prefix_dir()? {
            return Err(InstallerError::PermissionsError);
        }

        // Check if the current package to delete is a dependency, if so, give dependency error
        if self.register.is_dependency(&package_id.name, package_id.version.as_ref()) {
            return Err(InstallerError::DependencyError {
                package_name: package_id.name.clone(),
            });
        }

        // This determines the directory to remove. If there are multiple versions and the version is
        // specified only the specified version directory will be deleted. The entire package directory
        // is deleted if the version isn't specified or if the package directory only contains one version.
        match &package_id.version {
            Some(version) => self.uninstall_single(&PackageId::new(&package_id.name, &version))?,
            None => self.uninstall_all(&package_id.name)?,
        };

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
                warning!("Package not found eventhough package version was found, should be unreachable.");
                return Ok(());
            },
        };

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

        self.remove_dir_all(&directory, package_name)?;

        // Delete the installed package from toml
        self.register.remove_package(package_name);

        Ok(())
    }

    /// Wraps around the fs::remove_dir_all to map its error.
    fn remove_dir_all(&self, directory: &PathBuf, package_name: &str) -> Result<()> {
        match fs::remove_dir_all(directory) {
            Ok(_) => Ok(()), // TODO: Log succes with display
            Err(e) => Err(InstallerError::UninstallError {
                package_name: package_name.into(),
                e,
            }),
        }
    }

    fn get_latest_dependency_version(&self, dependency: &Dependency) -> Result<Version> {
        // Get all supported versions for the dependency
        let (_, package) = self.repository_manager.read_package(&dependency.get_name())?;

        // The supported vec isn't necessary in order, so we need to keep track of the current highest version
        let mut current_highest: Option<Version> = None;
        for version in package.versions {
            if !dependency.satisfied(&package.name, Some(&version)) {
                continue;
            }

            current_highest = match current_highest {
                Some(highest) if highest < version => Some(version),
                None => Some(version.clone()),
                _ => continue,
            };
        }

        Ok(current_highest.ok_or(InstallerError::SupportError(dependency.to_string()))?)
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
