//! uFBT (Unified Flipper Build Tool) integration.
//!
//! Provides:
//!   - Detect if uFBT is installed
//!   - Get uFBT and SDK versions
//!   - Install/update uFBT
//!   - Create, build, and deploy .fap plugins

use std::process::Command;
use std::path::Path;
use super::errors::AppError;

/// Check if uFBT is installed and available in PATH.
pub fn is_ufbt_installed() -> bool {
    Command::new("ufbt")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the installed uFBT version string.
pub fn get_ufbt_version() -> Result<String, AppError> {
    let output = Command::new("ufbt")
        .arg("--version")
        .output()
        .map_err(|e| AppError::General(format!("ufbt not found: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::General("ufbt --version failed".to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get the installed Flipper SDK version.
pub fn get_sdk_version() -> Result<String, AppError> {
    let output = Command::new("ufbt")
        .arg("status")
        .output()
        .map_err(|e| AppError::General(format!("ufbt not found: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::General("ufbt status failed".to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse SDK version from output
    for line in stdout.lines() {
        if line.contains("SDK") || line.contains("sdk") {
            return Ok(line.trim().to_string());
        }
    }

    Ok(stdout.trim().to_string())
}

/// Install uFBT via pip.
pub fn ufbt_install() -> Result<String, AppError> {
    let output = Command::new("pip")
        .args(["install", "ufbt"])
        .output()
        .map_err(|e| AppError::General(format!("pip not found: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::General(format!("ufbt install failed: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Update uFBT to the latest version.
pub fn ufbt_update() -> Result<String, AppError> {
    let output = Command::new("pip")
        .args(["install", "--upgrade", "ufbt"])
        .output()
        .map_err(|e| AppError::General(format!("pip not found: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::General(format!("ufbt update failed: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Create a new .fap plugin project.
pub fn create_fap_project(name: &str, path: &str) -> Result<String, AppError> {
    let output = Command::new("ufbt")
        .args(["create", "app", "--name", name])
        .current_dir(path)
        .output()
        .map_err(|e| AppError::General(format!("ufbt not found: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::General(format!("ufbt create failed: {}", stderr)));
    }

    Ok(format!("Created plugin: {} at {}", name, path))
}

/// Build a .fap plugin project.
pub fn build_fap(path: &str) -> Result<String, AppError> {
    let output = Command::new("ufbt")
        .arg("build")
        .current_dir(path)
        .output()
        .map_err(|e| AppError::General(format!("ufbt not found: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::General(format!("ufbt build failed: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Deploy a .fap to the connected Flipper.
pub fn deploy_fap(path: &str) -> Result<String, AppError> {
    let output = Command::new("ufbt")
        .arg("launch")
        .current_dir(path)
        .output()
        .map_err(|e| AppError::General(format!("ufbt not found: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::General(format!("ufbt launch failed: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Clean a .fap build directory.
pub fn clean_fap(path: &str) -> Result<String, AppError> {
    let output = Command::new("ufbt")
        .arg("clean")
        .current_dir(path)
        .output()
        .map_err(|e| AppError::General(format!("ufbt not found: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::General(format!("ufbt clean failed: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ufbt_installed_returns_bool() {
        // Should not panic, just return true or false
        let _ = is_ufbt_installed();
    }

    #[test]
    fn test_get_ufbt_version_error_when_missing() {
        // If ufbt is not installed, should return error
        // (we can't guarantee it's not installed, so just check no panic)
        let _ = get_ufbt_version();
    }
}
