use tempdir::TempDir;
use typescript_tools::{
    link::{link_typescript_project_references, LinkError},
    opts::Action,
};
use utilities::recursive_copy;

#[test]
fn link_happy_path_should_not_error() -> Result<(), LinkError> {
    let root = "test_data/happy_path";
    link_typescript_project_references(root, Action::Lint)?;
    Ok(())
}

#[test]
fn link_should_detect_missing_project_references() {
    let root = "test_data/project_references_missing";
    assert!(link_typescript_project_references(root, Action::Lint).is_err());
}

#[test]
fn link_should_correct_incorrect_project_references() -> Result<(), anyhow::Error> {
    // Arrange
    let template = "test_data/incorrect_project_references_playground";
    let root = TempDir::new("typescript-tools-test-link")?;
    let root = root.path().join("incorrect_project_references_playground");
    let root = root.as_path();
    recursive_copy(template, root)?;
    assert!(link_typescript_project_references(root, Action::Lint).is_err());

    // Act
    link_typescript_project_references(root, Action::Modify)?;

    // Assert
    link_typescript_project_references(root, Action::Lint)?;

    Ok(())
}
