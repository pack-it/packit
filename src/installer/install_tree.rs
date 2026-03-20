use crate::{
    installer::types::{Dependency, PackageId},
    platforms::Target,
    repositories::{
        error::RepositoryError,
        manager::RepositoryManager,
        types::{PackageMeta, PackageVersionMeta, TargetBounds},
    },
    storage::package_register::PackageRegister,
    utils::tree::{self, Node},
};

/// A label enum for the install/dependency tree
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DependencyTypes {
    Normal,
    Build,
}

/// A helper struct for the installer to move around nodes from the dependency trees
#[derive(Debug)]
pub struct InstallMeta {
    pub package_metadata: PackageMeta,
    pub version_metadata: PackageVersionMeta,
    pub repository_id: String,
    pub target_bounds: TargetBounds,
}

impl InstallMeta {
    fn new(
        package_metadata: PackageMeta,
        version_metadata: PackageVersionMeta,
        repository_id: String,
    ) -> std::result::Result<Self, RepositoryError> {
        let target_bounds = version_metadata.get_best_target(&Target::current())?;

        Ok(Self {
            package_metadata,
            version_metadata,
            repository_id,
            target_bounds,
        })
    }
}

pub type InstallNode = Node<Option<InstallMeta>, DependencyTypes>;

impl InstallNode {
    pub fn expander(parent: &InstallNode) -> tree::Result<Vec<(Dependency, DependencyTypes)>> {
        let install_meta = match parent.get_value() {
            Some(install_meta) => install_meta,
            None => return Ok(Vec::new()),
        };

        let target = install_meta.version_metadata.get_target(&install_meta.target_bounds)?;
        Ok(install_meta
            .version_metadata
            .dependencies
            .iter()
            .chain(target.dependencies.iter())
            .cloned()
            .map(|d| (d, DependencyTypes::Normal))
            .collect())
    }

    pub fn expander_with_build(parent: &InstallNode) -> tree::Result<Vec<(Dependency, DependencyTypes)>> {
        // Return early if the value was not set, it means that this node is already installed
        let install_meta = match parent.get_value() {
            Some(install_meta) => install_meta,
            None => return Ok(Vec::new()),
        };

        let target = install_meta.version_metadata.get_target(&install_meta.target_bounds)?;
        let build_dependencies = install_meta
            .version_metadata
            .build_dependencies
            .iter()
            .chain(target.build_dependencies.iter())
            .cloned()
            .map(|d| (d, DependencyTypes::Build));

        let dependencies = install_meta
            .version_metadata
            .dependencies
            .iter()
            .chain(target.dependencies.iter())
            .cloned()
            .map(|d| (d, parent.get_label().clone()));

        Ok(build_dependencies.chain(dependencies).collect())
    }

    pub fn populator(
        register: &PackageRegister,
        manager: &RepositoryManager,
        dependency: &Dependency,
        label: DependencyTypes,
    ) -> tree::Result<(PackageId, Option<InstallMeta>, DependencyTypes)> {
        // If the package is already satisfied don't expand the dependency tree further
        if let Some(package) = register.get_latest_satisfying_package(dependency) {
            return Ok((package.package_id.clone(), None, label));
        }

        // Use the latest version if the dependency is not yet satisfied
        let (repository_id, package_metadata) = manager.read_package(dependency.get_name())?;
        let version = package_metadata.get_latest_dependency_version(&dependency, &Target::current())?;
        let dependency_id = PackageId::new(dependency.get_name().clone(), version.clone());
        let version_metadata = manager.read_repo_package_version(&repository_id, &dependency_id)?;
        let install_meta = InstallMeta::new(package_metadata, version_metadata, repository_id)?;
        Ok((dependency_id, Some(install_meta), label))
    }

    pub fn expand_with_build(&mut self, register: &PackageRegister, manager: &RepositoryManager) -> tree::Result<()> {
        self.expand(&Self::expander_with_build, &|(d, l)| Self::populator(register, manager, &d, l))
    }
}
