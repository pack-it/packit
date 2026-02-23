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
}

impl Node<()> {
    pub fn new(package_id: &PackageId, register: &PackageRegister) -> Result<Self, TreeError> {
        Node::new_impl(package_id, register, &|package_id| {})
    }
}
