use tempdir::TempDir;
use typescript_tools::pin::PinError;
use utilities::recursive_copy;

#[test]
fn pin_happy_path_should_not_error() -> Result<(), PinError> {
    let root = "test_data/happy_path";
    typescript_tools::pin::modify(root)
}

#[test]
fn pin_should_detect_unpinned_internal_dependency() {
    let root = "test_data/unpinned_internal_dependency";
    assert!(typescript_tools::pin::lint(root).is_err());
}

#[test]
fn pin_should_correct_unpinned_internal_dependency() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let template = "test_data/unpinned_internal_dependency_playground";
    let root = TempDir::new("typescript-tools-test-pin")?;
    let root = root.path().join("unpinned_internal_dependency_playground");
    let root = root.as_path();
    recursive_copy(template, root)?;
    assert!(typescript_tools::pin::lint(root).is_err());

    // Act
    typescript_tools::pin::modify(root)?;

    // Assert
    typescript_tools::pin::lint(root)?;

    // TODO: snapshot test, to ensure the trailing newline, for example

    Ok(())
}
