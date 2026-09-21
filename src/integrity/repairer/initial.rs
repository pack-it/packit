// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use toml_edit::{DocumentMut, Table};

use crate::{
    cli::display::{QuestionResponse, ask_user, ask_user_input},
    config::{Config, EditableConfig, Repository},
    installer::types::{PackageName, Version},
    integrity::{error::Result, repairer::package, toml_repairer, utils::get_storage_packages},
    platforms::{
        DEFAULT_PREFIX,
        permissions::{does_packit_group_exist, set_packit_permissions},
    },
    register::{
        installed_package::InstalledPackage, installed_package_version::InstalledPackageVersion, package_register::PackageRegister,
    },
    repositories::manager::RepositoryManager,
    utils::{
        constants::{DEFAULT_METADATA_REPOSITORY_NAME, REGISTER_FILENAME},
        ioerror::IOResultExt,
    },
};

/// Fixes a missing `Config.toml`. Either by rebuilding the config from known information or using default values.
pub fn fix_missing_config() -> Result<()> {
    // Create a default config and adjust when fields can be recovered so new config fields don't create bugs
    let mut default_config = EditableConfig::default();

    if let Some(prefix_path) = get_config_prefix()? {
        default_config.set_prefix_directory(prefix_path.clone());

        // Try to recover the repositories, note that repository names cannot be recovered
        let repositories = get_config_repositories(&prefix_path)?;
        if !repositories.is_empty() {
            default_config.remove_repository(DEFAULT_METADATA_REPOSITORY_NAME);
            for (id, repository) in repositories {
                default_config.set_repository(&id, repository);
                default_config.add_to_repositories_rank(&id);
            }
        }
    }

    // Set multi-user to true if the packit group exists
    default_config.set_multiuser(does_packit_group_exist()?);

    confirm_config_construction(&default_config)
}

/// Fixes a broken `Config.toml`. For every field it first tries to recover it from the content still in the `Config.toml`.
/// If that doesn't work, it will try to recover from known information. The default value is used if these methods both fail.
pub fn fix_broken_config() -> Result<()> {
    let config_path = Config::get_default_path();
    let content = fs::read_to_string(&config_path).err_with_path("read", &config_path)?;
    let repaired_content = toml_repairer::repair_toml(&content);
    let document: DocumentMut = repaired_content.parse()?;
    let mut default_config = EditableConfig::default();

    // Get and set the prefix (only set the prefix if the prefix is not the default)
    let prefix = use_or_get_prefix(&document)?;
    if let Some(prefix) = &prefix {
        // MSRV: Remove PathBuf::from here when 1.91.0
        #[expect(clippy::cmp_owned)]
        if *prefix != PathBuf::from(DEFAULT_PREFIX) {
            default_config.set_prefix_directory(prefix.clone());
        }
    }

    // Get and set the repository rank (can be overwritten if no repositories can be found in the `repaired_content`)
    let mut rank_set = false;
    if let Some(rank) = document.get("repositories_rank").and_then(|item| item.as_array()) {
        let rank = rank.iter().filter_map(|item| item.as_str().map(String::from)).collect();
        default_config.set_repositories_rank(rank);
        rank_set = true;
    }

    // Get and set the repositories
    let repositories = get_repositories_from(&document)?;
    if !repositories.is_empty() {
        if !rank_set {
            default_config.set_repositories_rank(repositories.keys().map(String::from).collect());
        }

        default_config.remove_repository(DEFAULT_METADATA_REPOSITORY_NAME);
        for (id, repository) in repositories {
            default_config.set_repository(&id, repository);
        }
    } else if let Some(prefix_path) = prefix {
        // When reconstructing the repositories never use the found repository rank
        let repositories = get_config_repositories(&prefix_path)?;
        if !repositories.is_empty() {
            default_config.remove_repository(DEFAULT_METADATA_REPOSITORY_NAME);
            default_config.set_repositories_rank(repositories.keys().map(String::from).collect());
            for (id, repository) in repositories {
                default_config.set_repository(&id, repository);
            }
        }
    }

    // Get and set the multiuser value
    let multiuser = match document.get("multiuser").and_then(|item| item.as_bool()) {
        Some(value) => value,
        None => does_packit_group_exist()?,
    };

    // Only set the field if it's not the default
    if multiuser {
        default_config.set_multiuser(multiuser);
    }

    confirm_config_construction(&default_config)
}

/// Gets the prefix from the given document. If the prefix cannot be found in the document, it tries to
/// get it with `get_config_prefix`. `None` is returned if both attempts fail.
fn use_or_get_prefix(document: &DocumentMut) -> Result<Option<PathBuf>> {
    match document.get("prefix_directory").and_then(|item| item.as_str()) {
        Some(prefix_path) => Ok(Some(PathBuf::from(prefix_path))),
        None => get_config_prefix(),
    }
}

/// Tries to get the repositories from the given document.
fn get_repositories_from(document: &DocumentMut) -> Result<HashMap<String, Repository>> {
    let mut found_repositories = HashMap::new();
    let Some(repositories) = document.get("repositories").and_then(|item| item.as_table()) else {
        return Ok(found_repositories);
    };

    for (key, value) in repositories {
        let Some(value) = value.as_table() else { continue };
        let Some(repository) = get_repository(value) else { continue };
        found_repositories.insert(key.to_string(), repository);
    }

    Ok(found_repositories)
}

/// Tries to find the prefix directory. When found, the user is prompted to confirm. If it cannot be found
/// the user is prompted to provide the prefix path. If the path cannot be found `None` is returned.
fn get_config_prefix() -> Result<Option<PathBuf>> {
    // Figure out the prefix path
    let mut prefix_path = PathBuf::from(DEFAULT_PREFIX);
    loop {
        if fs::exists(&prefix_path).err_with_path("check existence of", &prefix_path)? {
            let question = format!("Prefix directory '{}' was found, do you wish to use this?", prefix_path.display());
            if ask_user(&question, QuestionResponse::Yes)?.is_yes() {
                return Ok(Some(prefix_path));
            }
        }

        let question = "Please provide a different prefix path".to_string();
        match ask_user_input(&question)? {
            Some(path) => prefix_path = PathBuf::from(path),

            // Return `None` if no valid prefix path can be found (no possibility for reconstruction)
            None => return Ok(None),
        }
    }
}

/// Tries to get the repositories for the config, based on the repositories that are used to install packages.
fn get_config_repositories(prefix_path: &Path) -> Result<HashMap<String, Repository>> {
    let mut found_repositories = HashMap::new();
    let register_dir = PackageRegister::get_path(prefix_path);
    let Ok(register) = PackageRegister::from(&register_dir) else {
        println!(
            "Could not use '{REGISTER_FILENAME}' to retrieve repositories from '{}', using the default repositories instead",
            prefix_path.display()
        );

        return Ok(found_repositories);
    };

    let used_repositories = get_used_repositories(&register);
    if used_repositories.is_empty() {
        println!("Could not find used repositories, using the default repositories instead");
        return Ok(found_repositories);
    }

    for (i, repository) in used_repositories.into_iter().enumerate() {
        // Create a unique name for each repository (we can't infer this from anything)
        let name = format!("repository_{}", i);
        found_repositories.insert(name, repository);
    }

    Ok(found_repositories)
}

/// Saves the reconstructed `Config.toml` to the default config path if the user confirms it.
fn confirm_config_construction(default_config: &EditableConfig) -> Result<()> {
    println!();
    println!("Reconstructed Config.toml:");
    default_config.get_config().display();
    println!();

    let question = "The Config.toml above has been constructed. Do you wish to use this as your config?";
    if ask_user(question, QuestionResponse::Yes)?.is_yes() {
        default_config.save_to(&Config::get_default_path())?;
    }

    Ok(())
}

/// Gets the used repositories from the register metadata in order based on occurrence rate.
fn get_used_repositories(register: &PackageRegister) -> HashSet<Repository> {
    // Find used repositories in package metadata, and keep track of how many times they are used
    let mut seen_repositories = HashMap::new();
    for package in register.iterate_all() {
        let compatible_repositories =
            register.get_repository_names().into_iter().filter(|name| **name != package.metadata_repository_name).cloned().collect();

        let repository = Repository {
            url: package.metadata_repository_url.trim_end_matches('/').to_string(),
            provider: package.metadata_repository_provider.clone(),
            prebuilds_url: package.prebuilds_repository_url.clone(),
            prebuilds_provider: package.prebuilds_repository_provider.clone(),
            disable_prebuilds: false,
            compatible_repositories,
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

/// Tries to get the repository from a given table. If a field is optional and its value cannot be
/// found the default is used. If a required field cannot be found `None` is returned.
fn get_repository(table: &Table) -> Option<Repository> {
    // Try to get all the fields, return early if a field cannot be found and is not optional
    let url = table.get("url")?.as_str()?.to_string();
    let provider = table.get("provider")?.as_str()?.to_string();
    let prebuilds_url = table.get("prebuilds_url").and_then(|item| item.as_str()).map(String::from);
    let prebuilds_provider = table.get("prebuilds_provider").and_then(|item| item.as_str()).map(String::from);
    let disable_prebuilds = table.get("disable_prebuilds").and_then(|item| item.as_bool()).unwrap_or(false);
    let compatible_repositories: Vec<String> = table
        .get("compatible_repositories")
        .and_then(|item| item.as_array())
        .map(|array| array.iter().filter_map(|item| item.as_str().map(String::from)).collect())
        .unwrap_or_default();

    Some(Repository {
        url,
        provider,
        prebuilds_url,
        prebuilds_provider,
        disable_prebuilds,
        compatible_repositories,
    })
}

/// Fixes a missing register. It considers all packages as missing and makes use of the inconsistent register fix.
pub fn fix_missing_register() -> Result<()> {
    // Note that the config can be used, because the check for a missing register depends on the config checks
    let config = Config::from(&Config::get_default_path())?;
    let mut register = PackageRegister::new_empty();
    let missing_packages = get_storage_packages(&config)?;
    let manager = RepositoryManager::new(&config);
    package::fix_inconsistent_register(missing_packages, &mut register, &config, &manager)?;
    register.save_to(&PackageRegister::get_path(&config.prefix_directory))?;
    Ok(())
}

/// Fixes a broken register. It recovers the valid parts of the toml file. The broken part will then be missing.
/// A later check for register consistency will catch this and fix it.
pub fn fix_broken_register() -> Result<()> {
    let config = Config::from(&Config::get_default_path())?;
    let register_path = PackageRegister::get_path(&config.prefix_directory);
    let content = fs::read_to_string(&register_path).err_with_path("read", &register_path)?;
    let repaired_content = toml_repairer::repair_toml(&content);
    let document: DocumentMut = repaired_content.parse()?;

    // If nothing can be found reset the entire register toml file
    let Some(package_table) = document.get("package").and_then(|item| item.as_table()) else {
        PackageRegister::new_empty().save_to(&register_path)?;
        return Ok(());
    };

    // Collect the fields that are still valid
    let mut packages: HashMap<PackageName, InstalledPackage> = HashMap::new();
    for (package_name, package_item) in package_table {
        let Ok(package_name) = PackageName::from_str(package_name) else {
            continue;
        };

        let mut package: InstalledPackage = match toml::from_str(&package_item.to_string()) {
            Ok(package) => package,
            Err(_) => continue,
        };

        let Some(toml_package) = package_item.as_table() else {
            continue;
        };

        // Note that we iterate over all keys in the package not only the version keys
        for (version, version_item) in toml_package {
            let Ok(version) = Version::from_str(version) else {
                continue;
            };

            let package_version: InstalledPackageVersion = match toml::from_str(&version_item.to_string()) {
                Ok(version) => version,
                Err(_) => continue,
            };

            package.versions.insert(version, package_version);
        }

        // Only add the package if at least one version can be found
        if !package.versions.is_empty() {
            packages.insert(package_name, package);
        }
    }

    let register = PackageRegister::new(packages);
    register.save_to(&register_path)?;

    Ok(())
}

/// Fix unwritable directories by setting the permissions again.
pub fn fix_unwritable_directories(directories: HashSet<PathBuf>) -> Result<()> {
    // Check for multiuser, prompt the user if the config doesn't work
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
