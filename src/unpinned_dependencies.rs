use std::{fmt::Display, path::PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub(crate) struct UnpinnedDependency {
    pub name: String,
    pub actual: String,
    pub expected: String,
}

impl Display for UnpinnedDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "dependency: {}\texpected: {}\tgot: {}",
            self.name, self.expected, self.actual
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct UnpinnedPackageDependencies(PathBuf, Vec<UnpinnedDependency>);

impl Display for UnpinnedPackageDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "File contains unpinned dependency versions: {:?}",
            self.0
        )?;
        for unpinned_dependency in self.1.iter() {
            writeln!(f, "\t{}", unpinned_dependency)?;
        }
        Ok(())
    }
}

impl From<(PathBuf, Vec<UnpinnedDependency>)> for UnpinnedPackageDependencies {
    fn from(value: (PathBuf, Vec<UnpinnedDependency>)) -> Self {
        Self(value.0, value.1)
    }
}

#[derive(Clone, Debug)]
pub struct UnpinnedMonorepoDependencies(Vec<UnpinnedPackageDependencies>);

impl Display for UnpinnedMonorepoDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for unpinned_package_dependencies in self.0.iter() {
            write!(f, "{}", unpinned_package_dependencies)?;
        }
        Ok(())
    }
}

impl From<Vec<UnpinnedPackageDependencies>> for UnpinnedMonorepoDependencies {
    fn from(value: Vec<UnpinnedPackageDependencies>) -> Self {
        Self(value)
    }
}

impl FromIterator<(PathBuf, Vec<UnpinnedDependency>)> for UnpinnedMonorepoDependencies {
    fn from_iter<T: IntoIterator<Item = (PathBuf, Vec<UnpinnedDependency>)>>(iter: T) -> Self {
        let collection = iter
            .into_iter()
            .filter(|(_package_name, unpinned_dependencies)| !unpinned_dependencies.is_empty())
            .map(Into::into)
            .collect();
        Self(collection)
    }
}

impl UnpinnedMonorepoDependencies {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{UnpinnedDependency, UnpinnedMonorepoDependencies, UnpinnedPackageDependencies};

    #[test]
    fn should_display_all_data() {
        let unpinned: UnpinnedMonorepoDependencies = vec![UnpinnedPackageDependencies(
            PathBuf::from("packages/a/package.json"),
            vec![
                UnpinnedDependency {
                    name: "one".into(),
                    actual: "0.0.0".into(),
                    expected: "2.0.0".into(),
                },
                UnpinnedDependency {
                    name: "two".into(),
                    actual: "0.0.0".into(),
                    expected: "2.0.0".into(),
                },
            ],
        )]
        .into();

        let expected = r#"
File contains unpinned dependency versions: "packages/a/package.json"
	dependency: one	expected: 2.0.0	got: 0.0.0
	dependency: two	expected: 2.0.0	got: 0.0.0
"#
        .trim_start();
        let actual = format!("{}", unpinned);
        assert_eq!(expected, actual);
    }
}
