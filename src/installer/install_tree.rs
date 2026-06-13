use std::collections::VecDeque;

// SPDX-License-Identifier: GPL-3.0-only
use crate::{
    cli::display::{QuestionResponse, ask_user, logging::error},
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
    utils::tree::{Node, Tree},
};

/// Represents the different types of installing a package.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InstallType {
    Prebuild,
    Build,
    BuildAll,
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

// TODO: Rename to resolver?
impl InstallTree {
    pub fn create_tree(
        package_id: PackageId,
        install_meta: InstallMeta,
        install_label: InstallLabel,
        register: &PackageRegister,
        manager: &RepositoryManager,
    ) -> Result<Self> {
        let root_label = InstallTree::check_prebuild(manager, &install_meta, &package_id, install_label)?;
        let root = Node::new(package_id, Some(install_meta), root_label);
        let mut tree = Tree::new(root);

        let mut package_queue = VecDeque::from([0 as usize]);
        while let Some(node_index) = package_queue.pop_front() {
            let node = tree.get_node_by_index_mut(node_index).expect("Expected node to exist");
            let dependencies = InstallTree::expander(node)?;
            for (dependency, label) in dependencies {
                let (dependency_id, meta, dependency_label) = InstallTree::populator(register, manager, &dependency, label)?;
                let new_node = Node::new(dependency_id, meta, dependency_label);
                let new_index = tree.add_node(node_index, new_node);
                package_queue.push_back(new_index);
            }
        }

        Ok(tree)
    }

    /// Expands a tree based on metadata and also takes into account already installed packages.
    /// The children install types are based on the parent install type and determine how the tree
    /// is further expanded (with or without build dependencies).
    fn expander(parent: &InstallNode) -> Result<Vec<(Dependency, InstallLabel)>> {
        // Return early if the node value is None (meaning that the package is already installed)
        // TODO: Introduce an extra type (Installed) for already installed packages, this section of the tree should be expanded with help of the regsiter (because requests are more expansive)
        let install_meta = match parent.get_value() {
            Some(install_meta) => install_meta,
            None => return Ok(Vec::new()),
        };

        // Determine the (build) dependency types of the children based on the parent
        let install_type = match *parent.get_label().get_type() {
            InstallType::Prebuild => InstallType::Prebuild,
            InstallType::Build => InstallType::Prebuild,
            InstallType::BuildAll => InstallType::BuildAll,
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
    /// to the tree without a value and children.
    fn populator(
        register: &PackageRegister,
        manager: &RepositoryManager,
        dependency: &Dependency,
        label: InstallLabel,
    ) -> Result<(PackageId, Option<InstallMeta>, InstallLabel)> {
        // If the package is already satisfied don't expand the dependency tree further
        // TODO Expand with register and installed label type
        if let Some(package) = register.get_latest_satisfying_package(dependency) {
            return Ok((package.package_id.clone(), None, label));
        }

        // Use the latest version if the dependency is not yet satisfied
        let (repository_id, package_metadata) = manager.read_package(dependency.get_name())?;
        let version = package_metadata.get_latest_dependency_version(dependency, &Target::current())?;
        let dependency_id = PackageId::new(dependency.get_name().clone(), version.clone());
        let version_metadata = manager.read_repo_package_version(&repository_id, &dependency_id)?;
        let install_meta = InstallMeta::new(package_metadata, version_metadata, repository_id)?;

        // Don't check for prebuild if the package should be build
        if matches!(label.get_type(), InstallType::Build | InstallType::BuildAll) {
            return Ok((dependency_id, Some(install_meta), label));
        }

        let label = InstallTree::check_prebuild(manager, &install_meta, &dependency_id, label)?;
        Ok((dependency_id, Some(install_meta), label))
    }

    fn check_prebuild(
        manager: &RepositoryManager,
        install_meta: &InstallMeta,
        package_id: &PackageId,
        label: InstallLabel,
    ) -> Result<InstallLabel> {
        // Check if a prebuild for the package is available
        let revision = install_meta.version_metadata.get_revision_count();
        match manager.get_prebuild_url(&install_meta.repository_id, &package_id, revision, &Target::current()) {
            Ok(Some(_)) => return Ok(label),
            Ok(None) | Err(RepositoryError::RepositoryNotFoundError { .. }) => (),
            Err(e) => error!(e),
        }

        // Return an error if the user doesn't want to build from source as alternative install method
        let question = format!("Prebuild package for {package_id} cannot be found, would you like to build from source instead?");
        if ask_user(&question, QuestionResponse::Yes)?.is_no_or_invalid() {
            return Err(InstallerError::InstallationCanceled {
                reason: format!("package '{package_id}' cannot be installed without building from source"),
            });
        }

        // Create an adjusted label
        let adjusted_label = InstallLabel {
            install_type: InstallType::Build,
            is_dependency: label.is_dependency(),
        };

        Ok(adjusted_label)
    }
}
