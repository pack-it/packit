use crate::{installer::types::PackageId, storage::package_register::PackageRegister};

#[derive(Debug)]
pub struct Node<T> {
    id: PackageId,
    value: T,
    children: Vec<Node<T>>,
}

impl<T> Node<T> {
    /// Creates a tree with package id as root. It also takes a closure which will get the necessary value for each node.
    pub fn from<F>(package_id: &PackageId, register: &PackageRegister, value_closure: &F) -> Self
    where
        F: Fn() -> T,
    {
        let package = register.get_package_version(package_id).expect("TODO");
        let children: Vec<Node<T>> = package.dependencies.iter().map(|d| Node::from(&d, register, value_closure)).collect();

        Self {
            id: package_id.clone(),
            value: value_closure(),
            children,
        }
    }
}
