// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    str::FromStr,
};

use crate::{
    cli::display::{QuestionResponse, ask_user, ask_user_input, logging::warning},
    config::{Config, EditableConfig, Repository},
    installer::{
        Installer, InstallerOptions, Symlinker,
        types::{Dependency, PackageId, PackageName, Version},
    },
    packager,
    platforms::{
        DEFAULT_PREFIX, Target,
        permissions::{does_packit_group_exist, set_packit_permissions},
        symlink::{self, SymlinkError},
    },
    repositories::{
        manager::RepositoryManager,
        types::{Checksum, PackageVersionMeta},
    },
    storage::package_register::PackageRegister,
    utils::constants::{DEFAULT_METADATA_REPOSITORY_NAME, REGISTER_FILENAME},
    verifier::{
        Issue,
        error::{Result, VerifierError},
        utils::get_storage_packages,
    },
};

/// Repairer that fixes issues found by the verifier.
pub struct Repairer;

impl Repairer {
    /// Creates a new repairer.
    pub fn new() -> Self {
        Self
    }

    pub fn fix_initial_issues(&mut self, issue: Issue) -> Result<()> {
        match issue {
            Issue::MissingConfig => self.fix_missing_config()?,
            Issue::IncorrectPermissions(directories) => self.fix_unwritable_directories(directories)?,
            Issue::MissingRegister => self.fix_missing_register()?,

            _ => warning!("Fix not executed, because it is not an initial issue"),
        }

        Ok(())
    }

    /// Fixes the given issue by executing the fix for that issue.
    /// Note: The register is not saved after the fix is applied.
    pub fn fix(&mut self, issue: Issue, register: &mut PackageRegister, config: &Config, manager: &RepositoryManager) -> Result<()> {
        match issue {
            Issue::BrokenTree(missing) => self.fix_broken_tree(missing, register, config, manager)?,
            Issue::InconsistentStorage(missing) => self.fix_inconsistent_storage(missing, register, config, manager)?,
            Issue::InconsistentRegister(missing) => self.fix_inconsistent_register(missing, register, config, manager)?,
            Issue::StrayDirectories(strays) => self.fix_stray_directories(strays)?,
            Issue::MissingDependencies(missing) => self.fix_missing_dependencies(missing, register, manager)?,
            Issue::InvalidDependencies(invalid) => self.fix_invalid_dependencies(invalid, register)?,
            Issue::MissingDependents(missing) => self.fix_missing_dependents(missing, register),
            Issue::InvalidDependents(invalid) => self.fix_invalid_dependents(invalid, register),
            Issue::InvalidActive(invalid) => self.fix_invalid_active(invalid, register, config)?,
            Issue::ForbiddenLink(forbidden) => self.fix_forbidden_link(forbidden, register, config)?,
            Issue::MissingLinks(missing) => self.fix_missing_links(missing, register, config)?,
            _ => warning!("Fix not executed, because the issue fix is not yet implemented"),
        }

        Ok(())
    }

    /// Fixes a missing Config.toml. Either by rebuilding the config from known information or using default values.
    fn fix_missing_config(&self) -> Result<()> {
        // Create a default config and adjust when fields can be recovered so new config fields don't create bugs
        let mut default_config = EditableConfig::default();

        // Figure out the prefix path
        let mut prefix_path = PathBuf::from(DEFAULT_PREFIX);
        loop {
            if fs::exists(&prefix_path)? {
                let question = format!("Prefix directory '{}' was found, do you wish to use this?", prefix_path.display());
                if ask_user(&question, QuestionResponse::Yes)?.is_yes() {
                    break;
                }
            }

            let question = "Please provide a different prefix path".to_string();
            match ask_user_input(&question)? {
                Some(path) => {
                    prefix_path = PathBuf::from(path);
                },

                // Return if no valid prefix path can be found (no possibility for reconstruction)
                None => return self.confirm_config_construction(&mut default_config),
            }
        }

        default_config.set_prefix_directory(prefix_path.clone());

        // Try to recover the repositories, repository names cannot be recovered
        let register_dir = prefix_path.join(REGISTER_FILENAME);
        if fs::exists(&register_dir)? {
            if let Ok(register) = PackageRegister::from(&register_dir) {
                default_config.remove_repository(DEFAULT_METADATA_REPOSITORY_NAME);
                let mut new_rank = Vec::new();
                for (i, repository) in self.get_used_repositories(&register).into_iter().enumerate() {
                    // Create a unique name for each repository (we can't infer this from anything)
                    let name = format!("repository_{}", i);
                    default_config.set_repository(&name, repository);
                    new_rank.push(name);
                }

                default_config.set_repositories_rank(new_rank);
            }
        } else {
            println!(
                "Could not open or parse '{REGISTER_FILENAME}' from '{}', using the default repositories instead",
                prefix_path.display()
            );
        }

        // Set multi-user to true if the packit group exists
        default_config.set_multiuser(does_packit_group_exist()?);

        self.confirm_config_construction(&mut default_config)
    }

    /// Saves the reconstructed Config.toml to the default config path if the user confirms it.
    fn confirm_config_construction(&self, default_config: &mut EditableConfig) -> Result<()> {
        println!();
        println!("Reconstructed Config.toml");
        default_config.get_config().display();
        println!();

        let question = "The Config.toml above has been constructed. Do you wish to use this as your config?";
        if ask_user(question, QuestionResponse::Yes)?.is_yes() {
            default_config.save_to(&Config::get_default_path())?;
        }

        Ok(())
    }

    /// Gets the used repositories from the register metadata in order based on occurance rate.
    fn get_used_repositories(&self, register: &PackageRegister) -> Vec<Repository> {
        // Find used repositories in package metadata, and keep track of how many times they are used
        let mut seen_repositories = HashMap::new();
        for package in register.iterate_all() {
            let repository = Repository {
                path: package.source_repository_url.clone(),
                provider: package.source_repository_provider.clone(),
                prebuilds_url: package.source_prebuild_repository_url.clone(),
                prebuilds_provider: package.source_prebuild_repository_provider.clone(),
            };

            match seen_repositories.get_mut(&repository) {
                Some(count) => *count += 1,
                None => _ = seen_repositories.insert(repository, 1),
            };
        }

        // Return the repositories in the correct order
        let mut repositories: Vec<_> = seen_repositories.into_iter().collect();
        repositories.sort_by_key(|(_, v)| *v);
        repositories.into_iter().map(|(k, _)| k).collect()
    }

    /// Fix unwritable directories by setting the permissions again.
    fn fix_unwritable_directories(&self, directories: HashSet<PathBuf>) -> Result<()> {
        // Check for multiuser, promt the user if the config doesn't work
        let multiuser = match Config::from(&Config::get_default_path()) {
            Ok(config) => config.multiuser,
            Err(_) => {
                let question = "Config.toml could not be loaded, do you wish to set permissions for multiuser?";
                ask_user(question, QuestionResponse::No)?.is_yes()
            },
        };

        // Set permissions for all unwritable directories
        for directory in directories {
            set_packit_permissions(&directory, multiuser, false)?;
        }

        Ok(())
    }

    /// Fixes broken dependency trees by installing the missing packages.
    fn fix_broken_tree(
        &self,
        missing: Vec<(PackageId, PackageId)>,
        register: &mut PackageRegister,
        config: &Config,
        manager: &RepositoryManager,
    ) -> Result<()> {
        for (package, missing_package) in missing {
            let package_version = match register.get_package_version_mut(&missing_package) {
                Some(package_version) => package_version,
                None => {
                    // Install the package
                    let installer_options = InstallerOptions::default().skip_symlinking(true);
                    let mut installer = Installer::new(config, register, manager, installer_options);
                    installer.install(&missing_package.clone().into())?;

                    // Assume installation worked
                    let Some(package_version) = register.get_package_version_mut(&missing_package) else {
                        continue;
                    };

                    package_version
                },
            };

            // Insert the package which misses the dependency as dependent
            package_version.dependents.insert(package.clone());

            // Fix the dependencies directory
            let dependencies_dir = config.prefix_directory.join("dependencies").join(package.to_string()).join(missing_package.name);
            match symlink::remove_symlink(&dependencies_dir) {
                Ok(_) => {},
                Err(SymlinkError::NonSymlink) => {},
                Err(e) => Err(e)?,
            }

            symlink::create_symlink(&package_version.install_path, &dependencies_dir)?;
        }

        Ok(())
    }

    /// Fixes inconsistent storage by temporarily removing the missing package from the register and then re-installing the packages.
    fn fix_inconsistent_storage(
        &mut self,
        missing: Vec<PackageId>,
        register: &mut PackageRegister,
        config: &Config,
        manager: &RepositoryManager,
    ) -> Result<()> {
        for missing_package in missing {
            // Gather the package settings before removing the package from the register
            let (symlinked, active) = match register.get_package(&missing_package.name) {
                Some(package) => (package.symlinked, package.active_version == missing_package.version),
                None => {
                    warning!("Inconsistent package cannot be found in Installed.toml anymore, eventhough it was found before.");
                    (false, false)
                },
            };

            // Temporarily remove the package from the register
            register.remove_package_version(&missing_package);

            let installer_options = InstallerOptions::default().skip_symlinking(!symlinked).skip_active(!active);
            let mut installer = Installer::new(config, register, manager, installer_options);

            installer.install(&missing_package.into())?;
        }

        Ok(())
    }

    /// Fixes a missing register. It considders all packages as missing and makes use of the inconsistent register fix.
    fn fix_missing_register(&mut self) -> Result<()> {
        // Note that the config can be used, because the check for a missing register depends on the config checks
        let config = Config::from(&Config::get_default_path())?;
        let mut register = PackageRegister::new_empty();
        let missing_packages = get_storage_packages(&config)?;
        let manager = RepositoryManager::new(&config);
        self.fix_inconsistent_register(missing_packages, &mut register, &config, &manager)?;
        register.save_to(&PackageRegister::get_default_path(&config.prefix_directory))?;
        Ok(())
    }

    /// Fixes an inconsistent register by gathering still existing data from the Packit directories.
    fn fix_inconsistent_register(
        &mut self,
        missing: HashSet<PackageId>,
        register: &mut PackageRegister,
        config: &Config,
        manager: &RepositoryManager,
    ) -> Result<()> {
        let storage_packages = get_storage_packages(config)?;
        let active_directory = config.prefix_directory.join("active");
        let bin_directory = config.prefix_directory.join("bin");
        let package_directory = config.prefix_directory.join("packages");
        for package_id in &missing {
            // Skip if the package already exists in the register (from recursive step)
            if register.get_package_version(package_id).is_some() {
                continue;
            }

            // Use checksum to check if a prebuild was used (use checksum to make sure it's the same prebuild)
            let install_path = &package_directory.join(&package_id.name).join(package_id.version.to_string());
            let (repository_id, package_version_meta) = manager.read_package_version(package_id, &Target::current())?;
            let revisions = package_version_meta.get_revision_count();
            let used_prebuild = match manager.get_prebuild_checksum(&repository_id, package_id, revisions, &Target::current())? {
                Some(correct_checksum) => {
                    let compressed = packager::compress(&install_path)?;
                    let checksum = Checksum::from_bytes(&compressed);
                    correct_checksum == checksum
                },
                None => false,
            };

            // Figure out the active version
            let active_target = fs::read_link(active_directory.join(&package_id.name))?;
            let target_name = active_target.file_name().ok_or(VerifierError::InvalidSymlink)?;
            let version = Version::from_str(target_name.to_str().ok_or(VerifierError::InvalidUnicodeError)?)?;

            // Check if the package is the active package version
            let active = package_id.version == version;

            // Figure out if symlinked
            let symlinked = fs::symlink_metadata(bin_directory.join(&package_id.name)).is_ok();

            // Get information with the manager
            // Note that this information is valid, but necessarily the same as before the issue arised
            let package_meta = manager.read_repo_package(&repository_id, &package_id.name)?;
            let dependencies = self.get_latest_satisfying_packages(&package_version_meta, &storage_packages);
            let source_repository = config.repositories.get(&repository_id).expect("Expected repository in config");

            // Make sure that all dependencies are registered as well
            let missing_dependencies = dependencies.iter().filter(|d| register.get_package_version(d).is_none()).cloned().collect();
            self.fix_inconsistent_register(missing_dependencies, register, config, manager)?;

            register.add_package(
                &package_meta,
                &package_version_meta,
                dependencies,
                source_repository,
                install_path,
                symlinked,
                active,
                used_prebuild,
            )?;
        }

        Ok(())
    }

    /// Gets the latest satisfying dependencies for a given package from the given storage packages.
    /// Assumes that all dependencies are present in the given storage packages.
    /// Returns a `HashSet` with all the latest packages which satisfy the dependencies of the given package id.
    fn get_latest_satisfying_packages(
        &self,
        package_version_meta: &PackageVersionMeta,
        storage_packages: &HashSet<PackageId>,
    ) -> HashSet<PackageId> {
        // Find the latest satisfying dependency
        let mut dependencies = HashSet::new();
        for dependency in &package_version_meta.dependencies {
            let mut latest: Option<&PackageId> = None;
            for package in storage_packages {
                if !dependency.satisfied(&package.name, &package.version) {
                    continue;
                }

                match latest {
                    Some(id) if id.version < package.version => latest = Some(package),
                    Some(_) => {},
                    None => latest = Some(package),
                }
            }

            // This assumes that all dependencies can be satisfied with packages in the given storage packages
            if let Some(latest) = latest {
                dependencies.insert(latest.clone());
            }
        }

        dependencies
    }

    /// Fixes stray directories by removing them.
    fn fix_stray_directories(&self, strays: HashSet<PathBuf>) -> Result<()> {
        for directory in strays {
            if !fs::exists(&directory)? {
                warning!(
                    "Skipping deletion of stray directory '{}' because it doesn't exist.",
                    directory.display()
                );
            }

            match directory.is_dir() {
                true => fs::remove_dir_all(directory)?,
                false => fs::remove_file(directory)?,
            }
        }

        Ok(())
    }

    /// Fixes missing dependencies by adding them to the register.
    /// If an installed satisfying package can be found it's used, otherwise the latest version is used instead.
    fn fix_missing_dependencies(
        &self,
        missing: Vec<(PackageId, Dependency)>,
        register: &mut PackageRegister,
        manager: &RepositoryManager,
    ) -> Result<()> {
        for (package_id, missing_dependency) in missing {
            // Try to find an installed satisfying package, if not found use latest version
            let package = match register.get_latest_satisfying_package(&missing_dependency) {
                Some(package) => package.package_id.clone(),
                None => {
                    let (_, package_metadata) = manager.read_package(missing_dependency.get_name())?;
                    let latest_version = package_metadata.get_latest_version(&Target::current())?;
                    PackageId::new(missing_dependency.get_name().clone(), latest_version.clone())
                },
            };

            // Set the dependency in the register
            let Some(package_version) = register.get_package_version_mut(&package_id) else {
                continue;
            };

            package_version.dependencies.insert(package);
        }

        Ok(())
    }

    /// Fixes the invalid dependencies issue.
    fn fix_invalid_dependencies(&self, invalid: Vec<(PackageId, PackageId)>, register: &mut PackageRegister) -> Result<()> {
        for (package, invalid_dependency) in invalid {
            // Assume the given package exists (continue otherwise)
            let Some(package_version) = register.get_package_version_mut(&package) else {
                continue;
            };

            package_version.dependencies.remove(&invalid_dependency);
        }

        Ok(())
    }

    /// Fixes the missing dependents issue.
    fn fix_missing_dependents(&self, missing: Vec<(PackageId, PackageId)>, register: &mut PackageRegister) {
        for (child, parent) in missing {
            let Some(package_version) = register.get_package_version_mut(&child) else {
                warning!("Could not fix missing dependents for {child}");
                continue;
            };

            package_version.dependents.insert(parent);
        }
    }

    /// Fixes the invalid dependents issue.
    fn fix_invalid_dependents(&self, invalid: Vec<(PackageId, PackageId)>, register: &mut PackageRegister) {
        for (child, parent) in invalid {
            let Some(package_version) = register.get_package_version_mut(&child) else {
                warning!("Could not fix invalid dependents for {child}");
                continue;
            };

            package_version.dependents.remove(&parent);
        }
    }

    /// Fixes the invalid active issue.
    fn fix_invalid_active(&self, invalid: Vec<PackageName>, register: &mut PackageRegister, config: &Config) -> Result<()> {
        let symlinker = Symlinker::new(config);
        for package_name in invalid {
            let Some(package) = register.get_package_mut(&package_name) else {
                warning!("Could not fix invalid active for {package_name}");
                continue;
            };

            // Set the active to the latest installed version of the package
            if let Some(version) = package.versions.keys().into_iter().max() {
                package.active_version = version.clone();
            }

            let package_id = PackageId::new(package_name.clone(), package.active_version.clone());
            let symlinked = package.symlinked.clone();
            symlinker.set_active(register, &package_id, symlinked)?;
        }

        Ok(())
    }

    /// Fixes the forbidden link issue.
    fn fix_forbidden_link(&self, forbidden: Vec<PackageName>, register: &mut PackageRegister, config: &Config) -> Result<()> {
        let symlinker = Symlinker::new(config);

        // Unlink all packages which shouldn't be symlinked
        for package_name in forbidden {
            symlinker.unlink_package(register, &package_name)?;
        }

        Ok(())
    }

    /// Fixes the missing links issue.
    fn fix_missing_links(&self, missing: Vec<PackageName>, register: &mut PackageRegister, config: &Config) -> Result<()> {
        let symlinker = Symlinker::new(config);

        // Re-link all packages which have missing symlinks
        for package_name in &missing {
            let Some(package) = register.get_package(package_name) else {
                warning!("Could not find package {package_name} for fix, skipping");
                continue;
            };

            let package_id = PackageId::new(package_name.clone(), package.active_version.clone());
            let Some(package_version) = register.get_package_version(&package_id) else {
                warning!("Could not find package {package_id} for fix, skipping");
                continue;
            };

            let install_path = package_version.install_path.clone();
            symlinker.unlink_package(register, package_name)?;
            symlinker.create_symlinks(&install_path)?;

            if let Some(package) = register.get_package_mut(package_name) {
                package.symlinked = true;
            };
        }

        Ok(())
    }
}
