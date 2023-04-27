use tempdir::TempDir;
use typescript_tools::{
    opts::Action,
    pin::{pin_version_numbers_in_internal_packages, PinError},
};
use utilities::recursive_copy;

#[test]
fn pin_happy_path_should_not_error() -> Result<(), PinError> {
    let root = "test_data/happy_path";
    pin_version_numbers_in_internal_packages(root, Action::Lint)?;
    Ok(())
}

#[test]
fn pin_should_detect_unpinned_internal_dependency() {
    let root = "test_data/unpinned_internal_dependency";
    assert!(pin_version_numbers_in_internal_packages(root, Action::Lint).is_err());
}

#[test]
fn pin_should_correct_unpinned_internal_dependency() -> Result<(), anyhow::Error> {
    // Arrange
    let template = "test_data/unpinned_internal_dependency_playground";
    let root = TempDir::new("typescript-tools-test-pin")?;
    let root = root.path().join("unpinned_internal_dependency_playground");
    let root = root.as_path();
    recursive_copy(template, root)?;
    assert!(pin_version_numbers_in_internal_packages(root, Action::Lint).is_err());

    // Act
    pin_version_numbers_in_internal_packages(root, Action::Modify)?;

    // Assert
    pin_version_numbers_in_internal_packages(root, Action::Lint)?;

    Ok(())
}
