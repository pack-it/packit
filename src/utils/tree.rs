use std::{collections::HashSet, fmt::Display};

use clap::error::Result;
use thiserror::Error;

use crate::{
    installer::{types::PackageId, DependencyTypes, InstallMeta},
    platforms::TARGET_ARCHITECTURE,
    repositories::{error::RepositoryError, manager::RepositoryManager},
    storage::package_register::PackageRegister,
};

#[derive(Debug)]
pub struct Node<V, L: Eq> {
    id: PackageId,
    value: V,
    children: Vec<Node<V, L>>,
    label: L,
}

#[derive(Error, Debug)]
pub enum TreeError {
    #[error("Package id '{0}' cannot be found")]
    NotFound(PackageId),

    #[error("Cannot create tree, because of an error reading the repository")]
    RepositoryError(#[from] RepositoryError),
}

const BRANCH: &str = "\u{251C}\u{2500}\u{2500}\u{2500} ";
const LAST_BRANCH: &str = "\u{2514}\u{2500}\u{2500}\u{2500} ";
const VERTICAL_LINE: &str = "\u{2502}    ";
const EMPTY_SPACE: &str = "     ";

impl<V, L: Eq> Node<V, L> {
    /// Creates a tree with package id as root. It also takes a closure which will get the necessary value for each node.
    fn new_impl<F>(package_id: &PackageId, register: &PackageRegister, value_closure: &F) -> Result<Self, TreeError>
    where
        F: Fn(&PackageId) -> (V, L),
    {
        let package = register.get_package_version(package_id).ok_or(TreeError::NotFound(package_id.clone()))?;
        let children: Vec<Node<V, L>> =
            package.dependencies.iter().map(|d| Node::new_impl(&d, register, value_closure)).collect::<Result<Vec<_>, _>>()?;

        let (value, label) = value_closure(package_id);

        Ok(Self {
            id: package_id.clone(),
            value,
            children,
            label,
        })
    }

    pub fn new_with_value<F>(package_id: &PackageId, register: &PackageRegister, value_closure: F) -> Result<Self, TreeError>
    where
        F: Fn(&PackageId) -> (V, L),
    {
        Self::new_impl(package_id, register, &value_closure)
    }

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

    pub fn get_id(&self) -> &PackageId {
        &self.id
    }

    pub fn get_value(&self) -> &V {
        &self.value
    }

    pub fn get_children_ids(&self, label: Option<L>) -> HashSet<PackageId> {
        if let Some(label) = label {
            return self.children.iter().filter(|c| c.label == label).map(|c| c.id.clone()).collect();
        }

        self.children.iter().map(|c| c.id.clone()).collect()
    }

    pub fn get_children(&self) -> &Vec<Node<V, L>> {
        &self.children
    }

    pub fn get_children_mut(&mut self) -> Vec<&mut Node<V, L>> {
        self.children.iter_mut().collect()
    }

    pub fn get_label(&self) -> &L {
        &self.label
    }
}

impl Node<(), ()> {
    pub fn new(package_id: &PackageId, register: &PackageRegister) -> Result<Self, TreeError> {
        Node::new_impl(package_id, register, &|_| ((), ()))
    }
}

impl Node<InstallMeta, DependencyTypes> {
    fn new_from_meta_impl(
        package_id: &PackageId,
        install_meta: InstallMeta,
        manager: &RepositoryManager,
        label: DependencyTypes,
        include_build: bool,
    ) -> Result<Self, TreeError> {
        let target = install_meta.version_metadata.get_target(TARGET_ARCHITECTURE)?;
        let dependencies = install_meta.version_metadata.dependencies.iter().chain(target.dependencies.iter());

        // Also add build dependencies if include build is true
        let children = if include_build {
            let build_dependencies = install_meta.version_metadata.build_dependencies.iter().chain(target.build_dependencies.iter());
            build_dependencies
                .into_iter()
                .map(|d| {
                    let install_meta = InstallMeta::new(manager, d)?;
                    let latest_version = install_meta.package_metadata.get_latest_version(TARGET_ARCHITECTURE)?;
                    let dependency_id = PackageId::new(d.get_name(), latest_version).expect("Expected valid dependency");
                    Self::new_from_meta_impl(&dependency_id, install_meta, manager, DependencyTypes::Build, include_build)
                })
                .chain(dependencies.map(|d| {
                    let install_meta = InstallMeta::new(manager, d)?;
                    let latest_version = install_meta.package_metadata.get_latest_version(TARGET_ARCHITECTURE)?;
                    let dependency_id = PackageId::new(d.get_name(), latest_version).expect("Expected valid dependency");
                    Self::new_from_meta_impl(&dependency_id, install_meta, manager, label.clone(), include_build)
                }))
                .collect::<Result<_, _>>()?
        } else {
            dependencies
                .into_iter()
                .map(|d| {
                    let install_meta = InstallMeta::new(manager, d)?;
                    let latest_version = install_meta.package_metadata.get_latest_version(TARGET_ARCHITECTURE)?;
                    let dependency_id = PackageId::new(d.get_name(), latest_version).expect("Expected valid dependency");
                    Self::new_from_meta_impl(&dependency_id, install_meta, manager, DependencyTypes::Normal, include_build)
                })
                .collect::<Result<_, _>>()?
        };

        Ok(Self {
            id: package_id.clone(),
            value: install_meta,
            children,
            label,
        })
    }

    pub fn new_from_meta(package_id: &PackageId, install_meta: InstallMeta, manager: &RepositoryManager) -> Result<Self, TreeError> {
        Self::new_from_meta_impl(package_id, install_meta, manager, DependencyTypes::Normal, false)
    }

    pub fn new_from_meta_build(package_id: &PackageId, install_meta: InstallMeta, manager: &RepositoryManager) -> Result<Self, TreeError> {
        Self::new_from_meta_impl(package_id, install_meta, manager, DependencyTypes::Normal, true)
    }

    pub fn expand_node_with_build(&mut self, manager: &RepositoryManager) -> Result<(), TreeError> {
        let target = self.value.version_metadata.get_target(TARGET_ARCHITECTURE)?;

        for dependency in self.value.version_metadata.build_dependencies.iter().chain(target.build_dependencies.iter()) {
            let install_meta = InstallMeta::new(manager, dependency)?;
            let latest_version = install_meta.package_metadata.get_latest_version(TARGET_ARCHITECTURE)?;
            let dependency_id = PackageId::new(dependency.get_name(), latest_version).expect("Expected valid dependency");
            let new_child = Self::new_from_meta_impl(&dependency_id, install_meta, manager, DependencyTypes::Normal, true)?;
            self.children.push(new_child);
        }

        Ok(())
    }
}

impl<V, L: Eq> Display for Node<V, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_impl(f, self, "")?;
        Ok(())
    }
}
