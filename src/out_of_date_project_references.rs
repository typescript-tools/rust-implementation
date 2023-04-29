use std::fmt::Display;

use crate::typescript_config::{
    TypescriptConfig, TypescriptParentProjectReference, TypescriptProjectReference,
};

#[derive(Debug)]
pub(crate) struct OutOfDateParentProjectReferences {
    pub tsconfig: TypescriptParentProjectReference,
    pub desired_references: Vec<TypescriptProjectReference>,
}

#[derive(Debug)]
pub(crate) struct OutOfDatePackageProjectReferences {
    pub tsconfig: TypescriptConfig,
    pub desired_references: Vec<TypescriptProjectReference>,
}

#[derive(Debug)]
pub enum MonorepoTypescriptConfig {
    #[non_exhaustive]
    Parent(TypescriptParentProjectReference),
    #[non_exhaustive]
    Package(TypescriptConfig),
}

#[derive(Debug)]
pub(crate) struct OutOfDateTypescriptConfig {
    config_file: MonorepoTypescriptConfig,
    #[allow(dead_code)]
    desired_references: Vec<TypescriptProjectReference>,
}

impl Display for OutOfDateTypescriptConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "File contains out-of-date project references: {:?}",
            self.config_file
        )
    }
}

impl From<OutOfDateParentProjectReferences> for OutOfDateTypescriptConfig {
    fn from(
        OutOfDateParentProjectReferences {
            tsconfig,
            desired_references,
        }: OutOfDateParentProjectReferences,
    ) -> Self {
        Self {
            config_file: MonorepoTypescriptConfig::Parent(tsconfig),
            desired_references,
        }
    }
}

impl From<OutOfDatePackageProjectReferences> for OutOfDateTypescriptConfig {
    fn from(
        OutOfDatePackageProjectReferences {
            tsconfig,
            desired_references,
        }: OutOfDatePackageProjectReferences,
    ) -> Self {
        Self {
            config_file: MonorepoTypescriptConfig::Package(tsconfig),
            desired_references,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct AllOutOfDateTypescriptConfig(Vec<OutOfDateTypescriptConfig>);

impl FromIterator<OutOfDateTypescriptConfig> for AllOutOfDateTypescriptConfig {
    fn from_iter<T: IntoIterator<Item = OutOfDateTypescriptConfig>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl AllOutOfDateTypescriptConfig {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for AllOutOfDateTypescriptConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for tsconfig in self.0.iter() {
            writeln!(f, "{}", tsconfig)?;
        }
        Ok(())
    }
}
