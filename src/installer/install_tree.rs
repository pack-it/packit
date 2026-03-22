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
pub enum InstallTypes {
    Prebuild {
        is_dependency: bool,
    },
    Build {
        is_dependency: bool,
    },
    BuildAll {
        is_dependency: bool,
    },
}

impl InstallTypes {
    pub fn is_dependency(&self) -> bool {
        match self {
            InstallTypes::Prebuild { is_dependency } => *is_dependency,
            InstallTypes::Build { is_dependency } => *is_dependency,
            InstallTypes::BuildAll { is_dependency } => *is_dependency,
        }
    }
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
    fn new(package_metadata: PackageMeta, version_metadata: PackageVersionMeta, repository_id: String) -> Result<Self, RepositoryError> {
        let target_bounds = version_metadata.get_best_target(&Target::current())?;

        Ok(Self {
            package_metadata,
            version_metadata,
            repository_id,
            target_bounds,
        })
    }
}

pub type InstallNode = Node<Option<InstallMeta>, InstallTypes>;

impl InstallNode {
    pub fn expander(parent: &InstallNode) -> tree::Result<Vec<(Dependency, InstallTypes)>> {
        let install_meta = match parent.get_value() {
            Some(install_meta) => install_meta,
            None => return Ok(Vec::new()),
        };

        // Determine the (build) dependency types of the children based on the parent
        // TODO: Refactor to not depent on Option
        let (install_type, build_install_type) = match *parent.get_label() {
            InstallTypes::Prebuild { .. } => (InstallTypes::Prebuild { is_dependency: true }, None),
            InstallTypes::Build { .. } => (
                InstallTypes::Prebuild { is_dependency: true },
                Some(InstallTypes::Prebuild { is_dependency: false }),
            ),
            InstallTypes::BuildAll { .. } => (
                InstallTypes::BuildAll { is_dependency: true },
                Some(InstallTypes::BuildAll { is_dependency: false }),
            ),
        };

        let target = install_meta.version_metadata.get_target(&install_meta.target_bounds)?;

        if let Some(build_install_type) = build_install_type {
            let build_dependencies = install_meta
                .version_metadata
                .build_dependencies
                .iter()
                .chain(target.build_dependencies.iter())
                .cloned()
                .map(|d| (d, build_install_type.clone()));

            let dependencies = install_meta
                .version_metadata
                .dependencies
                .iter()
                .chain(target.dependencies.iter())
                .cloned()
                .map(|d| (d, install_type.clone()));

            return Ok(build_dependencies.chain(dependencies).collect());
        }

        Ok(install_meta
            .version_metadata
            .dependencies
            .iter()
            .chain(target.dependencies.iter())
            .cloned()
            .map(|d| (d, install_type.clone()))
            .collect())
    }

    pub fn populator(
        register: &PackageRegister,
        manager: &RepositoryManager,
        dependency: &Dependency,
        label: InstallTypes,
    ) -> tree::Result<(PackageId, Option<InstallMeta>, InstallTypes)> {
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
        self.set_label(InstallTypes::Build {
            is_dependency: self.get_label().is_dependency(),
        });

        // TODO: Check if this doesn't give double children (because some children already exist)
        self.expand(&Self::expander, &|(d, l)| Self::populator(register, manager, &d, l))
    }
}
