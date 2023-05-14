use tempdir::TempDir;
use typescript_tools::link::LinkLintError;
use utilities::recursive_copy;

#[test]
fn link_happy_path_should_not_error() -> Result<(), LinkLintError> {
    let root = "test_data/happy_path";
    typescript_tools::link::lint(root)?;
    Ok(())
}

#[test]
fn link_should_detect_missing_project_references() {
    let root = "test_data/project_references_missing";
    assert!(typescript_tools::link::lint(root).is_err());
}

#[test]
fn link_should_correct_incorrect_project_references() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let template = "test_data/incorrect_project_references_playground";
    let root = TempDir::new("typescript-tools-test-link")?;
    let root = root.path().join("incorrect_project_references_playground");
    let root = root.as_path();
    recursive_copy(template, root)?;
    assert!(typescript_tools::link::lint(root).is_err());

    // Act
    typescript_tools::link::modify(root)?;

    // Assert
    typescript_tools::link::lint(root)?;

    // TODO: snapshot test, to ensure the trailing newline, for example

    Ok(())
}
