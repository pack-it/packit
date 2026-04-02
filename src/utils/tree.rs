// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::HashSet,
    fmt::{Debug, Display},
};

use thiserror::Error;

use crate::{installer::types::PackageId, repositories::error::RepositoryError, storage::package_register::PackageRegister};

/// The errors that occur while doing tree operations.
#[derive(Error, Debug)]
pub enum TreeError {
    #[error("Package id '{0}' cannot be found")]
    NotFound(PackageId),

    #[error("Cannot create tree, because of an error reading the repository.")]
    RepositoryError(#[from] RepositoryError),

    #[error("Cannot create tree, because necessary build attribute '{0}' was missing from the tree builder.")]
    MissingBuildAttributes(String),
}

pub type Result<T> = std::result::Result<T, TreeError>;

/// Represents a node in a dependency tree with generic values and labels.
#[derive(Debug)]
pub struct Node<V, L: Eq> {
    id: PackageId,
    value: V,
    children: Vec<Node<V, L>>,
    label: L,
}

/// A tree builder to build trees from a root, an expander and a populator.
#[derive(Debug)]
pub struct TreeBuilder<E, P, T, V, L: Eq>
where
    E: Fn(&Node<V, L>) -> Result<Vec<T>>,
    P: Fn(T) -> Result<(PackageId, V, L)>,
{
    /// The root of the tree.
    root: Option<Node<V, L>>,

    /// Closure specifying how to get children of a certain package id.
    expander: Option<E>,

    /// Closure specifying how to populate the tree with the expander return values.
    populator: Option<P>,
}

// Static string prefixes for the tree display
const BRANCH: &str = "\u{251C}\u{2500}\u{2500}\u{2500} ";
const LAST_BRANCH: &str = "\u{2514}\u{2500}\u{2500}\u{2500} ";
const VERTICAL_LINE: &str = "\u{2502}    ";
const EMPTY_SPACE: &str = "     ";

impl<E, P, T, V, L: Eq> TreeBuilder<E, P, T, V, L>
where
    E: Fn(&Node<V, L>) -> Result<Vec<T>>,
    P: Fn(T) -> Result<(PackageId, V, L)>,
{
    pub fn new() -> Self {
        Self {
            root: None,
            expander: None,
            populator: None,
        }
    }

    pub fn expander(mut self, expander: E) -> Self {
        self.expander = Some(expander);
        self
    }

    pub fn populator(mut self, populator: P) -> Self {
        self.populator = Some(populator);
        self
    }

    /// Sets the root of the tree. Its children are initialized to an empty vec.
    pub fn root(mut self, package_id: PackageId, value: V, label: L) -> Self {
        self.root = Some(Node {
            id: package_id,
            value,
            children: Vec::new(),
            label,
        });

        self
    }

    /// Builds the tree. Returns a result containing the root if successful. If any of
    /// the tree builder attributes are None a MissingBuildAttributes error is returned instead.
    pub fn build(self) -> Result<Node<V, L>> {
        let mut root = match self.root {
            Some(root) => root,
            None => return Err(TreeError::MissingBuildAttributes("root".to_string())),
        };

        let expander = match &self.expander {
            Some(expander) => expander,
            None => return Err(TreeError::MissingBuildAttributes("expander".to_string())),
        };

        let populator = match &self.populator {
            Some(populator) => populator,
            None => return Err(TreeError::MissingBuildAttributes("populator".to_string())),
        };

        root.expand(&expander, &populator)?;
        Ok(root)
    }
}

// Generic node implementation.
impl<V, L: Eq> Node<V, L> {
    /// Expands a node. The expander is used in combination with the populator to get its child nodes.
    /// If a child already exists it's not added again.
    pub fn expand<E, P, T>(&mut self, expander: &E, populator: &P) -> Result<()>
    where
        E: Fn(&Node<V, L>) -> Result<Vec<T>>,
        P: Fn(T) -> Result<(PackageId, V, L)>,
    {
        let existing_childs = self.get_children_ids();

        for child in expander(&self)? {
            let (id, value, label) = populator(child)?;

            // Node already exists, skip adding
            if existing_childs.contains(&id) {
                continue;
            }

            let mut child_node = Node {
                id,
                value,
                children: Vec::new(),
                label,
            };

            child_node.expand(expander, populator)?;
            self.children.push(child_node);
        }

        Ok(())
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

    /// Gets the ids of the children of the node in a hashset.
    /// Checks for a filter on the label and will only return the children that satisfy the filter.
    pub fn get_children_ids_filtered<F>(&self, filter: F) -> HashSet<PackageId>
    where
        F: Fn(&L) -> bool,
    {
        self.children.iter().filter(|c| filter(&c.label)).map(|c| c.id.clone()).collect()
    }

    /// Gets the ids of the children of the node in a hashset.
    pub fn get_children_ids(&self) -> HashSet<PackageId> {
        self.children.iter().map(|c| c.id.clone()).collect()
    }

    /// Gets the child nodes as reference.
    pub fn get_children(&self) -> &Vec<Node<V, L>> {
        &self.children
    }

    /// Gets the child nodes as mutable reference.
    pub fn get_children_mut(&mut self) -> &mut Vec<Node<V, L>> {
        &mut self.children
    }

    /// Gets the label.
    pub fn get_label(&self) -> &L {
        &self.label
    }

    /// Sets a new label.
    pub fn set_label(&mut self, new_label: L) {
        self.label = new_label;
    }
}

/// Represents an empty node, without any value or label.
pub type EmptyNode = Node<(), ()>;

// An empty node implementation, without any values or labels.
impl EmptyNode {
    /// Builds a simple tree based on the installed packages.
    pub fn build_simple_tree(package_id: PackageId, register: &PackageRegister) -> Result<EmptyNode> {
        TreeBuilder::new()
            .root(package_id, (), ())
            .expander(|p| EmptyNode::expander(p, register))
            .populator(EmptyNode::populator)
            .build()
    }

    /// Gets the children based on the installed packages.
    fn expander(parent: &EmptyNode, register: &PackageRegister) -> Result<Vec<PackageId>> {
        let package = register.get_package_version(parent.get_id()).ok_or(TreeError::NotFound(parent.get_id().clone()))?;
        Ok(package.dependencies.iter().cloned().collect())
    }

    /// Populates nodes with empty values and labels.
    fn populator(package_id: PackageId) -> Result<(PackageId, (), ())> {
        Ok((package_id, (), ()))
    }
}

// Display trait for nice display of a tree.
impl<V, L: Eq> Display for Node<V, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_impl(f, self, "")?;
        Ok(())
    }
}
