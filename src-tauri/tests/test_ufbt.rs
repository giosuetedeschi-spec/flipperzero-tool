//! Tests for uFBT module

use flipperzero_tool_lib::ufbt::*;

#[test]
fn test_is_ufbt_installed_no_panic() {
    let _ = is_ufbt_installed();
}

#[test]
fn test_get_ufbt_version_error_when_missing() {
    if !is_ufbt_installed() {
        assert!(get_ufbt_version().is_err());
    }
}

#[test]
fn test_get_sdk_version_no_panic() {
    let _ = get_sdk_version();
}

#[test]
fn test_create_fap_project_empty_name() {
    assert!(create_fap_project("", "/tmp").is_err());
}

#[test]
fn test_create_fap_project_empty_path() {
    assert!(create_fap_project("test", "").is_err());
}

#[test]
fn test_build_fap_invalid_path() {
    assert!(build_fap("/nonexistent/path").is_err());
}

#[test]
fn test_deploy_fap_invalid_path() {
    assert!(deploy_fap("/nonexistent/path").is_err());
}

#[test]
fn test_clean_fap_invalid_path() {
    assert!(clean_fap("/nonexistent/path").is_err());
}
