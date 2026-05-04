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
        Installer, InstallerOptions,
        types::{PackageId, Version},
    },
    platforms::{
        DEFAULT_PREFIX, Target,
        permissions::{packit_group_exists, set_packit_permissions},
    },
    repositories::{manager::RepositoryManager, types::PackageVersionMeta},
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
            _ => warning!("Fix not executed, because the issue fix is not yet implemented"),
        }

        Ok(())
    }

    /// Fixes a missing Config.toml. Either by rebuilding the config from known information or using default values.
    fn fix_missing_config(&self) -> Result<()> {
        // Create a default config and adjust when fields can be recovered so new config fields don't create bugs
        let mut default_config = EditableConfig::default()?;

        // Figure out the prefix path
        let mut prefix_string = DEFAULT_PREFIX.to_string();
        let mut prefix_path = PathBuf::from(&prefix_string);
        loop {
            if fs::exists(&prefix_path)? {
                let question = format!("Prefix directory '{prefix_string}' was found, do you wish to use this?");
                if ask_user(&question, QuestionResponse::Yes)?.is_yes() {
                    break;
                }
            }

            let question = format!("Please provide a different prefix path");
            match ask_user_input(&question)? {
                Some(path) => {
                    prefix_string = path.to_string();
                    prefix_path = PathBuf::from(&prefix_string);
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
            println!("Could not open or parse '{REGISTER_FILENAME}' from '{prefix_string}', using the default repositories instead");
        }

        // Set multi-user to true if the packit group exists
        default_config.set_multiuser(packit_group_exists());

        self.confirm_config_construction(&mut default_config)?;

        Ok(())
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
                if ask_user(question, QuestionResponse::No)?.is_no() { false } else { true }
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
        for (_, missing_package) in missing {
            // There could be duplicates in the missing packages, so skip when already seen
            if register.get_package_version(&missing_package).is_some() {
                continue;
            }

            // Install the package
            let installer_options = InstallerOptions::default().skip_symlinking(true);
            let mut installer = Installer::new(config, register, manager, installer_options);
            installer.install(&missing_package.clone().into())?;
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
        let mut register = PackageRegister::new();
        let missing_packages = get_storage_packages(&config)?;
        let manager = RepositoryManager::new(&config);
        self.fix_inconsistent_register(missing_packages, &mut register, &config, &manager)?;
        register.save_to(&PackageRegister::get_default_path(&config))?;
        Ok(())
    }

    /// Fixes an inconsistent register by gathering still existing data from the Packit directories.
    /// TODO: Expand with a check with package checksums, to make sure that the found repository has the same checksum
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
            let (_, package_meta) = manager.read_package(&package_id.name)?;
            let (repository_id, package_version_meta) = manager.read_package_version(&package_id, &Target::current())?;
            let dependencies = self.get_latest_satisfying_packages(&package_version_meta, &storage_packages);
            let source_repository = config.repositories.get(&repository_id).expect("Expected repository in config");
            let install_path = &package_directory.join(&package_id.name).join(package_id.version.to_string());
            let prebuild_url = manager.get_prebuild_url(
                &repository_id,
                &package_id,
                package_version_meta.revisions.len() as u64,
                &Target::current(),
            )?;

            // Make sure that all dependencies are registered as well
            let missing_dependencies = dependencies.iter().filter(|d| missing.contains(d)).cloned().collect();
            self.fix_inconsistent_register(missing_dependencies, register, config, manager)?;

            register.add_package(
                &package_meta,
                &package_version_meta,
                dependencies,
                source_repository,
                install_path,
                symlinked,
                active,
                prebuild_url.is_some(),
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
}
