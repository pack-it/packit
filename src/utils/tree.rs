// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::{HashSet, VecDeque},
    fmt::{Debug, Display},
};

use thiserror::Error;

use crate::{installer::types::PackageId, register::package_register::PackageRegister, repositories::error::RepositoryError};

// Static string prefixes for the tree display
const BRANCH: &str = "\u{251C}\u{2500}\u{2500}\u{2500} ";
const LAST_BRANCH: &str = "\u{2514}\u{2500}\u{2500}\u{2500} ";
const VERTICAL_LINE: &str = "\u{2502}    ";
const EMPTY_SPACE: &str = "     ";

/// The errors that occur while doing tree operations.
#[derive(Error, Debug)]
pub enum TreeError {
    #[error("Package id '{0}' cannot be found")]
    NotFound(PackageId),

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
    label: L,
}

impl<V, L: Eq> Tree<V, L> {
    pub fn new(root: Node<V, L>) -> Self {
        Self { nodes: vec![root] }
    }

    pub fn add_node(&mut self, parent_index: usize, node: Node<V, L>) -> usize {
        self.nodes.push(node);
        let index = self.nodes.len() - 1;
        let parent = self.nodes.get_mut(parent_index).expect("TODO");
        parent.children.push(index);
        index
    }

    pub fn get_root(&self) -> &Node<V, L> {
        self.nodes.get(0).expect("Expected root to exist")
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
            .map(|c| self.get_node_by_index(*c).expect("TODO"))
            .filter(|c| filter(&c.label))
            .map(|c| c.package_id.clone())
            .collect()
    }

    /// The implementation of the tree display.
    fn display_impl(&self, f: &mut std::fmt::Formatter<'_>, node_index: &usize, prefix: &str) -> std::fmt::Result {
        let node = self.get_node_by_index(*node_index).expect("Expected node to exist");
        writeln!(f, "{prefix}{}", node.package_id)?;

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

// Generic node implementation.
impl<V, L: Eq> Node<V, L> {
    pub fn new(package_id: PackageId, value: V, label: L) -> Self {
        Self {
            package_id,
            value,
            children: Vec::new(),
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
    pub fn new_emtpy(package_id: PackageId, register: &PackageRegister) -> Result<EmptyTree> {
        let root = Node::new(package_id, (), ());
        let mut tree = Tree::new(root);

        let mut package_queue = VecDeque::from([0 as usize]);
        while let Some(node_index) = package_queue.pop_front() {
            let node = tree.get_node_by_index_mut(node_index).expect("Expected node to exist");
            let dependencies = &register
                .get_package_version(node.get_package_id())
                .ok_or(TreeError::NotFound(node.get_package_id().clone()))?
                .dependencies;

            for dependency in dependencies {
                let new_node = Node::new(dependency.clone(), (), ());
                let new_index = tree.add_node(node_index, new_node);
                package_queue.push_back(new_index);
            }
        }

        Ok(tree)
    }
}

// Display trait for nice display of a tree.
impl<V, L: Eq> Display for Tree<V, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_impl(f, &0, "")?;
        Ok(())
    }
}
