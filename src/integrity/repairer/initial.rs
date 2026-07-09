// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use crate::{
    cli::display::{QuestionResponse, ask_user, ask_user_input},
    config::{Config, EditableConfig, Repository},
    integrity::{error::Result, repairer::package, utils::get_storage_packages},
    platforms::{
        DEFAULT_PREFIX,
        permissions::{does_packit_group_exist, set_packit_permissions},
    },
    register::package_register::PackageRegister,
    repositories::manager::RepositoryManager,
    utils::{
        constants::{DEFAULT_METADATA_REPOSITORY_NAME, REGISTER_FILENAME},
        ioerror::IOResultExt,
    },
};

/// Fixes a missing Config.toml. Either by rebuilding the config from known information or using default values.
pub fn fix_missing_config() -> Result<()> {
    // Create a default config and adjust when fields can be recovered so new config fields don't create bugs
    let mut default_config = EditableConfig::default();

    // Figure out the prefix path
    let mut prefix_path = PathBuf::from(DEFAULT_PREFIX);
    loop {
        if fs::exists(&prefix_path).err_with_path("check existance of", &prefix_path)? {
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
            None => return confirm_config_construction(&mut default_config),
        }
    }

    default_config.set_prefix_directory(prefix_path.clone());

    // Try to recover the repositories, note that repository names cannot be recovered
    set_config_repositories(&prefix_path, &mut default_config)?;

    // Set multi-user to true if the packit group exists
    default_config.set_multiuser(does_packit_group_exist()?);

    confirm_config_construction(&mut default_config)
}

/// Sets the repositories field, if they can be found with the `get_used_repositories`.
fn set_config_repositories(prefix_path: &PathBuf, default_config: &mut EditableConfig) -> Result<()> {
    let register_dir = PackageRegister::get_path(prefix_path);
    if let Ok(register) = PackageRegister::from(&register_dir) {
        let used_repositories = get_used_repositories(&register);
        if !used_repositories.is_empty() {
            default_config.remove_repository(DEFAULT_METADATA_REPOSITORY_NAME);

            let mut new_rank = Vec::new();
            for (i, repository) in used_repositories.into_iter().enumerate() {
                // Create a unique name for each repository (we can't infer this from anything)
                let name = format!("repository_{}", i);
                default_config.set_repository(&name, repository);
                new_rank.push(name);
            }

            default_config.set_repositories_rank(new_rank);

            return Ok(());
        }
    }

    println!(
        "Could not use '{REGISTER_FILENAME}' to reconstruct repositories from '{}', using the default repositories instead",
        prefix_path.display()
    );

    Ok(())
}

/// Saves the reconstructed Config.toml to the default config path if the user confirms it.
fn confirm_config_construction(default_config: &mut EditableConfig) -> Result<()> {
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
fn get_used_repositories(register: &PackageRegister) -> Vec<Repository> {
    // Find used repositories in package metadata, and keep track of how many times they are used
    let mut seen_repositories = HashMap::new();
    for package in register.iterate_all() {
        let repository = Repository {
            url: package.metadata_repository_url.clone(),
            provider: package.metadata_repository_provider.clone(),
            prebuilds_url: package.prebuilds_repository_url.clone(),
            prebuilds_provider: package.prebuilds_repository_provider.clone(),
            disable_prebuilds: false,
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

/// Fixes a missing register. It considders all packages as missing and makes use of the inconsistent register fix.
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

/// Fix unwritable directories by setting the permissions again.
pub fn fix_unwritable_directories(directories: HashSet<PathBuf>) -> Result<()> {
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
