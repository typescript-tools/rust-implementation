use typescript_tools::lint::LintError;

#[test]
fn lint_happy_path_should_not_error() -> Result<(), LintError> {
    let root = "test_data/happy_path";
    typescript_tools::lint::lint_dependency_version(root, &["external"])?;
    Ok(())
}

#[test]
fn lint_should_error_when_multiple_version_of_an_external_dependency_are_used() {
    let root = "test_data/external_dependency_multiple_versions";
    assert!(
        typescript_tools::lint::lint_dependency_version(root, &["@typescript-tools/external"])
            .is_err()
    );
}
