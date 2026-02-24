use std::fmt::Display;

use thiserror::Error;

use crate::{installer::types::PackageId, storage::package_register::PackageRegister};

#[derive(Debug)]
pub struct Node<T> {
    id: PackageId,
    value: T,
    children: Vec<Node<T>>,
}

#[derive(Error, Debug)]
pub enum TreeError {
    #[error("Package id '{0}' cannot be found")]
    NotFound(PackageId),
}

const BRANCH: &str = "\u{251C}\u{2500}\u{2500}\u{2500} ";
const LAST_BRANCH: &str = "\u{2514}\u{2500}\u{2500}\u{2500} ";
const VERTICAL_LINE: &str = "\u{2502}    ";
const EMPTY_SPACE: &str = "     ";

impl<T> Node<T> {
    /// Creates a tree with package id as root. It also takes a closure which will get the necessary value for each node.
    fn new_impl<F>(package_id: &PackageId, register: &PackageRegister, value_closure: &F) -> Result<Self, TreeError>
    where
        F: Fn(&PackageId) -> T,
    {
        let package = register.get_package_version(package_id).ok_or(TreeError::NotFound(package_id.clone()))?;
        let children: Vec<Node<T>> =
            package.dependencies.iter().map(|d| Node::new_impl(&d, register, value_closure)).collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            id: package_id.clone(),
            value: value_closure(&package_id),
            children,
        })
    }

    pub fn new_with_value<F>(package_id: &PackageId, register: &PackageRegister, value_closure: F) -> Result<Self, TreeError>
    where
        F: Fn(&PackageId) -> T,
    {
        Self::new_impl(package_id, register, &value_closure)
    }

    fn display_impl(&self, f: &mut std::fmt::Formatter<'_>, node: &Node<T>, prefix: &str) -> std::fmt::Result {
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
}

impl Node<()> {
    pub fn new(package_id: &PackageId, register: &PackageRegister) -> Result<Self, TreeError> {
        Node::new_impl(package_id, register, &|_| {})
    }
}

impl<T> Display for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_impl(f, self, "")?;
        Ok(())
    }
}
