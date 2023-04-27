use typescript_tools::{
    link::{link_typescript_project_references, LinkError},
    opts::Action,
};

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
