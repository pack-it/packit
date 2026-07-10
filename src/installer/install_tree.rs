// SPDX-License-Identifier: GPL-3.0-only
use std::collections::{HashSet, VecDeque};

use console::Term;

use crate::{
    cli::display::{QuestionResponse, ask_user, logging::error, styled::Styled},
    installer::{
        error::{InstallerError, Result},
        types::{Dependency, PackageId},
    },
    platforms::Target,
    register::package_register::PackageRegister,
    repositories::{
        error::RepositoryError,
        manager::RepositoryManager,
        types::{PackageMeta, PackageVersionMeta, TargetBounds},
    },
    utils::{
        ioerror::IOResultExt,
        tree::{Node, Tree},
    },
};

/// Represents the different types of installing a package.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InstallType {
    Prebuild,
    Build,
    BuildAll,
    Installed,
}

/// Represents the label for the install tree.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InstallLabel {
    install_type: InstallType,
    is_dependency: bool,
}

impl InstallLabel {
    pub fn new(install_type: InstallType, is_dependency: bool) -> Self {
        Self {
            install_type,
            is_dependency,
        }
    }

    pub fn get_type(&self) -> &InstallType {
        &self.install_type
    }

    /// Returns true if it is a normal dependency (not a build dependency).
    /// If it is a dependency of a build dependency this also returns true.
    pub fn is_dependency(&self) -> bool {
        self.is_dependency
    }
}

/// Helper struct for the installer, containing the necessary metadata for installation of a package.
#[derive(Debug)]
pub struct InstallMeta {
    pub package_metadata: PackageMeta,
    pub version_metadata: PackageVersionMeta,
    pub repository_id: String,
    pub target_bounds: TargetBounds,
}

impl InstallMeta {
    /// Creates a new `InstallMeta` struct.
    fn new(package_metadata: PackageMeta, version_metadata: PackageVersionMeta, repository_id: String) -> Result<Self> {
        let target_bounds = version_metadata.get_best_target(&Target::current())?;

        Ok(Self {
            package_metadata,
            version_metadata,
            repository_id,
            target_bounds,
        })
    }
}

pub type InstallTree = Tree<Option<InstallMeta>, InstallLabel>;
pub type InstallNode = Node<Option<InstallMeta>, InstallLabel>;

pub struct InstallTreeBuilder<'a> {
    register: &'a PackageRegister,
    repository_manager: &'a RepositoryManager<'a>,
    checked_packages: HashSet<PackageId>,
    asked_packages: HashSet<PackageId>,
    terminal: Term,
}

impl<'a> InstallTreeBuilder<'a> {
    pub fn new(register: &'a PackageRegister, repository_manager: &'a RepositoryManager) -> Self {
        Self {
            register,
            repository_manager,
            checked_packages: HashSet::new(),
            asked_packages: HashSet::new(),
            terminal: Term::stdout(),
        }
    }

    pub fn create_tree(&mut self, package_id: PackageId, root_meta: InstallMeta, root_label: InstallLabel) -> Result<InstallTree> {
        let mut tree_display_string = "".to_string();
        let root_label = match self.check_prebuild(&root_meta, &package_id, &root_label)? {
            Some(adjusted_label) => adjusted_label,
            None => root_label,
        };

        let root = Node::new(package_id, Some(root_meta), root_label);
        let mut tree = Tree::new(root);
        tree_display_string = self.update_tree_display(&tree_display_string, &tree)?;

        let mut package_queue = VecDeque::from([0]);
        while let Some(node_index) = package_queue.pop_front() {
            let node = tree.get_node_by_index_mut(node_index).expect("Expected node to exist");

            // Expand with register if the node value is None and has the Installed label type (meaning that the package is already installed)
            let install_meta = match node.get_value() {
                Some(install_meta) => install_meta,
                None if node.get_label().install_type == InstallType::Installed => {
                    // Note that we expect the package to exist because the node has the `InstallType::Installed` label type
                    let dependencies =
                        &self.register.get_package_version(node.get_package_id()).expect("Expected package version to exist").dependencies;
                    for dependency in dependencies {
                        let new_node = Node::new(dependency.clone(), None, InstallLabel::new(InstallType::Installed, false));
                        let new_index = tree.add_node(node_index, new_node)?;
                        tree_display_string = self.update_tree_display(&tree_display_string, &tree)?;
                        package_queue.push_back(new_index);
                    }

                    continue;
                },
                None => {
                    return Err(InstallerError::UnreachableError {
                        msg: "Node value is None without InstallType::Installed".to_string(),
                    });
                },
            };

            for (dependency, label) in self.expander(node, install_meta)? {
                let (dependency_id, meta, dependency_label) = self.populator(&dependency, label)?;
                let new_node = Node::new(dependency_id, meta, dependency_label);
                let new_index = tree.add_node(node_index, new_node)?;
                tree_display_string = self.update_tree_display(&tree_display_string, &tree)?;
                package_queue.push_back(new_index);
            }
        }

        Ok(tree)
    }

    /// Expands a tree based on metadata and also takes into account already installed packages.
    /// The children install types are based on the parent install type and determine how the tree
    /// is further expanded (with or without build dependencies).
    fn expander(&self, parent: &InstallNode, install_meta: &InstallMeta) -> Result<Vec<(Dependency, InstallLabel)>> {
        // Determine the (build) dependency types of the children based on the parent
        let install_type = match *parent.get_label().get_type() {
            InstallType::Prebuild => InstallType::Prebuild,
            InstallType::Build => InstallType::Prebuild,
            InstallType::BuildAll => InstallType::BuildAll,
            _ => {
                return Err(InstallerError::UnreachableError {
                    msg: "InstallType::Installed should be unreachable".to_string(),
                });
            },
        };

        let target = install_meta.version_metadata.get_target(&install_meta.target_bounds)?;
        let dependencies = install_meta
            .version_metadata
            .dependencies
            .iter()
            .chain(target.dependencies.iter())
            .cloned()
            .map(|d| (d, InstallLabel::new(install_type.clone(), true)));

        // Only return normal dependencies when the parent is a prebuild
        if *parent.get_label().get_type() == InstallType::Prebuild {
            return Ok(dependencies.collect());
        }

        // Get the build dependencies
        let build_dependencies = install_meta
            .version_metadata
            .build_dependencies
            .iter()
            .chain(target.build_dependencies.iter())
            .cloned()
            .map(|d| (d, InstallLabel::new(install_type.clone(), false)));

        Ok(build_dependencies.chain(dependencies).collect())
    }

    /// Populates the tree with metadata info. If a package is already installed it is added
    /// to the tree without a value and with LabelType::Installed.
    fn populator(&mut self, dependency: &Dependency, label: InstallLabel) -> Result<(PackageId, Option<InstallMeta>, InstallLabel)> {
        // Return early with empty value if the package is already satisfied
        if let Some(package) = self.register.get_latest_satisfying_package(dependency) {
            let adjusted_label = InstallLabel::new(InstallType::Installed, label.is_dependency());
            return Ok((package.package_id.clone(), None, adjusted_label));
        }

        // Use the latest version if the dependency is not yet satisfied
        let target = Target::current();
        let (repository_id, package_meta) = self.repository_manager.read_package(dependency.get_name())?;
        let version_meta =
            self.repository_manager.read_latest_supported_dependency_version(&repository_id, &package_meta, dependency, &target)?;
        let dependency_id = PackageId::new(dependency.get_name().clone(), version_meta.version.clone());
        let install_meta = InstallMeta::new(package_meta, version_meta, repository_id)?;

        let label = match self.check_prebuild(&install_meta, &dependency_id, &label)? {
            Some(adjusted_label) => adjusted_label,
            None => label,
        };

        Ok((dependency_id, Some(install_meta), label))
    }

    fn check_prebuild(&mut self, install_meta: &InstallMeta, package_id: &PackageId, label: &InstallLabel) -> Result<Option<InstallLabel>> {
        // Don't check for prebuild if the package should not use a prebuild
        if !matches!(label.get_type(), InstallType::Prebuild) {
            return Ok(None);
        }

        // Note that if we have asked before and the program is still running we can assume the user agreed to do a build
        if self.asked_packages.contains(package_id) {
            return Ok(Some(InstallLabel::new(InstallType::Build, label.is_dependency())));
        }

        if self.checked_packages.contains(package_id) {
            return Ok(None);
        }

        self.checked_packages.insert(package_id.clone());

        // Check if a prebuild for the package is available
        let revision = install_meta.version_metadata.get_revision_count();
        match self.repository_manager.get_prebuild_url(&install_meta.repository_id, package_id, revision, &Target::current()) {
            Ok(Some(_)) => return Ok(None),
            Ok(None) | Err(RepositoryError::RepositoryNotFoundError { .. }) => {},
            Err(e) => error!(e),
        }

        // Return an error if the user doesn't want to build from source as alternative install method
        let question = format!(
            "Prebuild package for {} cannot be found, would you like to build from source instead?",
            package_id.style()
        );
        if ask_user(&question, QuestionResponse::Yes)?.is_no_or_invalid() {
            return Err(InstallerError::InstallationCanceled {
                reason: format!("package {} cannot be installed without building from source", package_id.style()),
            });
        }

        self.asked_packages.insert(package_id.clone());

        // Remove the question
        self.terminal.clear_last_lines(1).err_operation("clear terminal lines")?;

        // Return an adjusted label
        Ok(Some(InstallLabel::new(InstallType::Build, label.is_dependency())))
    }

    /// Updates the tree structure shown in the terminal.
    fn update_tree_display(&self, tree_string: &str, tree: &InstallTree) -> Result<String> {
        // Remove previous display content
        let number_of_lines = tree_string.lines().count();
        self.terminal.clear_last_lines(number_of_lines).err_operation("clear terminal lines")?;

        print!("{tree}");

        Ok(tree.to_string())
    }
}
