use std::{collections::HashSet, fmt::Display};

use thiserror::Error;

use crate::{
    installer::{
        DependencyTypes, InstallMeta,
        types::{Dependency, PackageId},
    },
    platforms::Target,
    repositories::{error::RepositoryError, manager::RepositoryManager},
    storage::package_register::PackageRegister,
};

/// A node to create a dependency tree with generic values and labels.
#[derive(Debug)]
pub struct Node<V, L: Eq> {
    id: PackageId,
    value: V,
    children: Vec<Node<V, L>>,
    label: L,
}

/// Errors which could occur while doing tree operations
#[derive(Error, Debug)]
pub enum TreeError {
    #[error("Package id '{0}' cannot be found")]
    NotFound(PackageId),

    #[error("Cannot expand a node which does not have a value to expand with.")]
    ExpansionError,

    #[error("Cannot create tree, because of an error reading the repository")]
    RepositoryError(#[from] RepositoryError),
}

type Result<T> = std::result::Result<T, TreeError>;

// Static string prefixes for the tree display
const BRANCH: &str = "\u{251C}\u{2500}\u{2500}\u{2500} ";
const LAST_BRANCH: &str = "\u{2514}\u{2500}\u{2500}\u{2500} ";
const VERTICAL_LINE: &str = "\u{2502}    ";
const EMPTY_SPACE: &str = "     ";

/// Generic node implementation.
impl<V, L: Eq> Node<V, L> {
    /// Creates a tree based on the installed packages, the given package id will be the root.
    /// It also takes a closure which will get the necessary value and label for each node.
    fn new_impl<F>(package_id: &PackageId, register: &PackageRegister, value_closure: &F) -> Result<Self>
    where
        F: Fn(&PackageId) -> (V, L),
    {
        let package = register.get_package_version(package_id).ok_or(TreeError::NotFound(package_id.clone()))?;
        let children: Vec<Node<V, L>> =
            package.dependencies.iter().map(|d| Node::new_impl(&d, register, value_closure)).collect::<Result<Vec<_>>>()?;

        let (value, label) = value_closure(package_id);

        Ok(Self {
            id: package_id.clone(),
            value,
            children,
            label,
        })
    }

    /// A wrapper method to create a tree with nodes that hold values.
    pub fn new_with_value<F>(package_id: &PackageId, register: &PackageRegister, value_closure: F) -> Result<Self>
    where
        F: Fn(&PackageId) -> (V, L),
    {
        Self::new_impl(package_id, register, &value_closure)
    }

    /// The implementation of the tree display.
    fn display_impl(&self, f: &mut std::fmt::Formatter<'_>, node: &Node<V, L>, prefix: &str) -> std::fmt::Result {
        write!(f, "{prefix}{}\n", node.id)?;

        // Note that when the input prefix is "" then this prefix will also be ""
        let prefix = match prefix.ends_with(BRANCH) {
            true => prefix.replace(BRANCH, VERTICAL_LINE),
            false => prefix.replace(LAST_BRANCH, EMPTY_SPACE),
        };

        for (index, child) in node.children.iter().enumerate() {
            let branch_section = match index == node.children.len() - 1 {
                true => LAST_BRANCH,
                false => BRANCH,
            };

            self.display_impl(f, child, format!("{prefix}{branch_section}").as_str())?;
        }

        Ok(())
    }

    /// Gets the package id.
    pub fn get_id(&self) -> &PackageId {
        &self.id
    }

    /// Gets the value of the node.
    pub fn get_value(&self) -> &V {
        &self.value
    }

    /// Gets the children ids of a node in a hashset. If a label has been given it will only return the children with that label.
    pub fn get_children_ids(&self, label: Option<L>) -> HashSet<PackageId> {
        if let Some(label) = label {
            return self.children.iter().filter(|c| c.label == label).map(|c| c.id.clone()).collect();
        }

        self.children.iter().map(|c| c.id.clone()).collect()
    }

    /// Gets references to the child nodes.
    pub fn get_children(&self) -> &Vec<Node<V, L>> {
        &self.children
    }

    /// Gets mutable references to the child nodes.
    pub fn get_children_mut(&mut self) -> &mut Vec<Node<V, L>> {
        &mut self.children
    }

    /// Gets the label.
    pub fn get_label(&self) -> &L {
        &self.label
    }
}

/// An empty node implementation (node without values or labels).
impl Node<(), ()> {
    pub fn new(package_id: &PackageId, register: &PackageRegister) -> Result<Self> {
        Node::new_impl(package_id, register, &|_| ((), ()))
    }
}

/// A node implementation based on metadata instead of installed packages. Meant specifically for that installer.
impl Node<Option<InstallMeta>, DependencyTypes> {
    /// Creates a tree based on metadata with the given package id as root.
    /// If include_build is true the build dependencies are included in the tree (with the appropriate labels).
    fn new_from_meta_impl(
        package_id: &PackageId,
        install_meta: InstallMeta,
        manager: &RepositoryManager,
        register: &PackageRegister,
        label: DependencyTypes,
        include_build: bool,
    ) -> Result<Self> {
        let target = install_meta.version_metadata.get_target(&install_meta.target_bounds)?;
        let dependencies = install_meta.version_metadata.dependencies.iter().chain(target.dependencies.iter());

        // Also add build dependencies if include build is true
        let children = match include_build {
            true => install_meta
                .version_metadata
                .build_dependencies
                .iter()
                .chain(target.build_dependencies.iter())
                .map(|d| Self::new_from_dependency(manager, register, d, DependencyTypes::Build, include_build))
                .chain(dependencies.map(|d| Self::new_from_dependency(manager, register, d, label.clone(), include_build)))
                .collect::<Result<_>>()?,
            false => dependencies
                .into_iter()
                .map(|d| Self::new_from_dependency(manager, register, d, DependencyTypes::Normal, include_build))
                .collect::<Result<_>>()?,
        };

        Ok(Self {
            id: package_id.clone(),
            value: Some(install_meta),
            children,
            label,
        })
    }

    /// A wrapper method which creates the metadata dependency tree without build dependencies.
    pub fn new_from_meta(
        package_id: &PackageId,
        install_meta: InstallMeta,
        manager: &RepositoryManager,
        register: &PackageRegister,
    ) -> Result<Self> {
        Self::new_from_meta_impl(package_id, install_meta, manager, register, DependencyTypes::Normal, false)
    }

    /// A wrapper method which creates the metadata dependency tree with build dependencies.
    pub fn new_from_meta_build(
        package_id: &PackageId,
        install_meta: InstallMeta,
        manager: &RepositoryManager,
        register: &PackageRegister,
    ) -> Result<Self> {
        Self::new_from_meta_impl(package_id, install_meta, manager, register, DependencyTypes::Normal, true)
    }

    /// Expands a node with its build dependencies after initial creation of the tree.
    /// Note that this only applies for the build dependencies of the current node, the
    /// dependencies of the current node will be satisfied with pre-builds.
    pub fn expand_node_with_build(&mut self, manager: &RepositoryManager, register: &PackageRegister) -> Result<()> {
        let value = match &self.value {
            Some(value) => value,
            None => return Err(TreeError::ExpansionError),
        };

        let target = value.version_metadata.get_target(&value.target_bounds)?;
        for dependency in value.version_metadata.build_dependencies.iter().chain(target.build_dependencies.iter()) {
            self.children.push(Self::new_from_dependency(
                manager,
                register,
                dependency,
                DependencyTypes::Build,
                false,
            )?);
        }

        Ok(())
    }

    /// A helper method which does an often used recursion step. It creates the install meta from the dependency and then calls the new_from_meta_impl.
    fn new_from_dependency(
        manager: &RepositoryManager,
        register: &PackageRegister,
        dependency: &Dependency,
        label: DependencyTypes,
        include_build: bool,
    ) -> Result<Node<Option<InstallMeta>, DependencyTypes>> {
        // If the package is already satisfied don't expand the dependency tree further
        if let Some(package) = register.get_latest_satisfying_package(dependency) {
            return Ok(Node {
                id: package.package_id.clone(),
                value: None,
                children: Vec::new(),
                label,
            });
        }

        // Use the latest version if the dependency is not yet satisfied
        let (repository_id, package_metadata) = manager.read_package(dependency.get_name())?;
        let version = package_metadata.get_latest_dependency_version(&dependency, &Target::current())?;
        let dependency_id = PackageId::new(dependency.get_name().clone(), version.clone());
        let version_metadata = manager.read_repo_package_version(&repository_id, &dependency_id)?;
        let install_meta = InstallMeta::new(package_metadata, version_metadata, repository_id)?;
        Self::new_from_meta_impl(&dependency_id, install_meta, manager, register, label, include_build)
    }
}

/// Display trait for nice display of a tree.
impl<V, L: Eq> Display for Node<V, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_impl(f, self, "")?;
        Ok(())
    }
}
