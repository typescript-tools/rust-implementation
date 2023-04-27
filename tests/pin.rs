use typescript_tools::{
    opts::Action,
    pin::{pin_version_numbers_in_internal_packages, PinError},
};

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
