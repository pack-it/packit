// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::{HashSet, VecDeque},
    fmt::{Debug, Display},
};

use thiserror::Error;

use crate::{
    cli::display::styled::Styled, installer::types::PackageId, register::package_register::PackageRegister,
    repositories::error::RepositoryError,
};

// Static string prefixes for the tree display
const BRANCH: &str = "\u{251C}\u{2500}\u{2500}\u{2500} ";
const LAST_BRANCH: &str = "\u{2514}\u{2500}\u{2500}\u{2500} ";
const VERTICAL_LINE: &str = "\u{2502}    ";
const EMPTY_SPACE: &str = "     ";

/// The errors that occur while doing tree operations.
#[derive(Error, Debug)]
pub enum TreeError {
    #[error("Package id {} cannot be found", .0.style())]
    NotFound(PackageId),

    #[error("Parent with node id '{0}' cannot be found")]
    NonExistentParent(usize),

    #[error("{} has itself as a dependency, creating a cycle in the tree", .0.style())]
    CycleError(PackageId),

    #[error("Cannot create tree, because of an error reading the repository.")]
    RepositoryError(#[from] RepositoryError),
}

pub type Result<T> = std::result::Result<T, TreeError>;

pub struct Tree<V, L: Eq> {
    // A list of all nodes in the tree. Note that this list will always have parents before children by
    // way of construction (you can only add a node as child of another node).
    nodes: Vec<Node<V, L>>,
}

/// Represents a node in a dependency tree with generic values and labels.
#[derive(Debug)]
pub struct Node<V, L: Eq> {
    package_id: PackageId,
    value: V,
    children: Vec<usize>,
    parent_index: usize,
    label: L,
}

impl<V, L: Eq> Tree<V, L> {
    pub fn new(root: Node<V, L>) -> Self {
        Self { nodes: vec![root] }
    }

    /// Adds a node to the tree under the given parent index.
    /// Returns the index of the node added to the nodes list.
    /// Returns a `TreeError::CycleError` if the new node creates a cycle in the tree.
    /// Returns a `TreeError::NonExistentParent` if the given parent index doesn't exist.
    pub fn add_node(&mut self, parent_index: usize, mut node: Node<V, L>) -> Result<usize> {
        // Check for cycles before adding the node to the tree
        if self.is_cyclic(parent_index, node.get_package_id())? {
            return Err(TreeError::CycleError(node.get_package_id().clone()));
        }

        // Check if parent node exists
        if self.nodes.get(parent_index).is_none() {
            return Err(TreeError::NonExistentParent(parent_index));
        }

        node.parent_index = parent_index;
        self.nodes.push(node);
        let index = self.nodes.len() - 1;

        let parent = self.nodes.get_mut(parent_index).expect("Expected parent to exist");
        parent.children.push(index);
        Ok(index)
    }

    /// Gets the root of the tree.
    #[expect(unused)]
    pub fn get_root(&self) -> &Node<V, L> {
        self.nodes.first().expect("Expected root to exist")
    }

    /// Returns a reference to a list which contains the nodes of the tree.
    pub fn get_nodes(&self) -> &Vec<Node<V, L>> {
        &self.nodes
    }

    /// Gets a node based on its index.
    pub fn get_node_by_index(&self, index: usize) -> Option<&Node<V, L>> {
        self.nodes.get(index)
    }

    /// Gets a node mutably based on its index.
    pub fn get_node_by_index_mut(&mut self, index: usize) -> Option<&mut Node<V, L>> {
        self.nodes.get_mut(index)
    }

    /// Gets the ids of the children of the node in a hashset.
    /// Checks for a filter on the label and will only return the children that satisfy the filter.
    pub fn get_children_ids_filtered<F>(&self, node: &Node<V, L>, filter: F) -> HashSet<PackageId>
    where
        F: Fn(&L) -> bool,
    {
        node.get_children()
            .iter()
            .map(|c| self.get_node_by_index(*c).expect("Expected node to exist"))
            .filter(|c| filter(&c.label))
            .map(|c| c.package_id.clone())
            .collect()
    }

    /// Checks if a given package id will create a cycle in the tree.
    /// Returns true if a cycle will be formed, false if not.
    /// Returns `TreeError::NonExistentParent` if the given parent_index doesn't exist.
    fn is_cyclic(&self, parent_index: usize, package_id: &PackageId) -> Result<bool> {
        let mut current_parent = parent_index;
        while current_parent != 0 {
            let Some(parent_node) = self.get_node_by_index(current_parent) else {
                return Err(TreeError::NonExistentParent(current_parent));
            };

            if parent_node.get_package_id() == package_id {
                return Ok(true);
            }

            current_parent = parent_node.get_parent();
        }

        Ok(false)
    }

    /// The implementation of the tree display.
    fn display_impl(&self, f: &mut std::fmt::Formatter<'_>, node_index: usize, prefix: &str) -> std::fmt::Result {
        let node = self.get_node_by_index(node_index).expect("Expected node to exist");
        writeln!(f, "{prefix}{}", node.package_id.style())?;

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

            self.display_impl(f, *child, format!("{prefix}{branch_section}").as_str())?;
        }

        Ok(())
    }
}

// Generic node implementation.
impl<V, L: Eq> Node<V, L> {
    /// Creates a new `Node`. Note that the parent_index is 0 by default. The root will have itself as parent
    /// and when adding a node to a `Tree` the parent index will need to be adjusted.
    pub fn new(package_id: PackageId, value: V, label: L) -> Self {
        Self {
            package_id,
            value,
            children: Vec::new(),
            parent_index: 0,
            label,
        }
    }

    /// Gets the package id.
    pub fn get_package_id(&self) -> &PackageId {
        &self.package_id
    }

    /// Gets the value of the node.
    pub fn get_value(&self) -> &V {
        &self.value
    }

    /// Gets the parent index of the node.
    pub fn get_parent(&self) -> usize {
        self.parent_index
    }

    /// Gets the child nodes as reference.
    pub fn get_children(&self) -> &Vec<usize> {
        &self.children
    }

    /// Gets the label.
    pub fn get_label(&self) -> &L {
        &self.label
    }
}

/// Represents a tree, without any value or label at its nodes.
pub type EmptyTree = Tree<(), ()>;

// An empty node implementation, without any values or labels.
impl EmptyTree {
    /// Builds a simple tree based on the installed packages.
    pub fn new_empty(package_id: PackageId, register: &PackageRegister) -> Result<EmptyTree> {
        let root = Node::new(package_id, (), ());
        let mut tree = Tree::new(root);

        let mut package_queue = VecDeque::from([0]);
        while let Some(node_index) = package_queue.pop_front() {
            let node = tree.get_node_by_index_mut(node_index).expect("Expected node to exist");
            let dependencies = &register
                .get_package_version(node.get_package_id())
                .ok_or(TreeError::NotFound(node.get_package_id().clone()))?
                .dependencies;

            for dependency in dependencies {
                let new_node = Node::new(dependency.clone(), (), ());
                let new_index = tree.add_node(node_index, new_node)?;
                package_queue.push_back(new_index);
            }
        }

        Ok(tree)
    }
}

// Display trait for nice display of a tree.
impl<V, L: Eq> Display for Tree<V, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_impl(f, 0, "")?;
        Ok(())
    }
}
