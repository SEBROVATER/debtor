use std::env;
use std::fs;
use std::path::Path;

use tempfile::TempDir;

/// T002: Test that .env file values are loaded into the process environment
#[test]
fn loads_env_file_into_process_environment() {
    let dir = TempDir::new().expect("temp dir");
    let env_path = dir.path().join(".env");
    fs::write(
        &env_path,
        "T002_DB_URL=sqlite://test.db\nT002_USER=testuser\n",
    )
    .expect("write .env");

    // Clear vars first
    unsafe {
        env::remove_var("T002_DB_URL");
        env::remove_var("T002_USER");
    };

    dotenvy::from_path(&env_path).expect("dotenvy should load .env");

    assert_eq!(env::var("T002_DB_URL").unwrap(), "sqlite://test.db");
    assert_eq!(env::var("T002_USER").unwrap(), "testuser");

    // Cleanup
    unsafe {
        env::remove_var("T002_DB_URL");
        env::remove_var("T002_USER");
    }
}

/// T003: Test that system environment variables take precedence over .env values
#[test]
fn system_env_takes_precedence_over_dotenv() {
    let dir = TempDir::new().expect("temp dir");
    let env_path = dir.path().join(".env");
    fs::write(&env_path, "T003_VAR=from_dotenv\n").expect("write .env");

    // Clear and set system env BEFORE loading .env
    unsafe {
        env::remove_var("T003_VAR");
        env::set_var("T003_VAR", "from_system");
    };

    // Use dotenv_iter to load only vars not already set (system env takes precedence)
    for item in dotenvy::from_path_iter(&env_path).expect("dotenvy iter") {
        let (key, value) = item.expect("valid entry");
        if env::var(&key).is_err() {
            unsafe { env::set_var(&key, &value) };
        }
    }

    // System env should win
    assert_eq!(
        env::var("T003_VAR").unwrap(),
        "from_system",
        "system env must take precedence over .env"
    );

    // Cleanup
    unsafe { env::remove_var("T003_VAR") };
}

/// T003b: Test that .env-only vars are loaded when not in system env
#[test]
fn dotenv_vars_loaded_when_not_in_system() {
    let dir = TempDir::new().expect("temp dir");
    let env_path = dir.path().join(".env");
    fs::write(&env_path, "T003B_NEW=from_dotenv\n").expect("write .env");

    // Clear var
    unsafe { env::remove_var("T003B_NEW") };

    for item in dotenvy::from_path_iter(&env_path).expect("dotenvy iter") {
        let (key, value) = item.expect("valid entry");
        if env::var(&key).is_err() {
            unsafe { env::set_var(&key, &value) };
        }
    }

    assert_eq!(
        env::var("T003B_NEW").unwrap(),
        "from_dotenv",
        "new vars from .env should be loaded"
    );

    // Cleanup
    unsafe { env::remove_var("T003B_NEW") };
}

/// T004: Test graceful fallback when no .env file exists
#[test]
fn graceful_fallback_when_no_env_file() {
    let dir = TempDir::new().expect("temp dir - empty, no .env");

    // dotenvy::from_path on a missing file should return Err
    let result = dotenvy::from_path(dir.path().join(".env"));
    assert!(result.is_err(), "should return Err when .env not found");
}

/// T009: Test that .env.example contains all 6 required variable keys
#[test]
fn env_example_contains_all_variables() {
    let example_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env.example");
    assert!(
        example_path.exists(),
        ".env.example must exist in project root"
    );

    let content = fs::read_to_string(&example_path).expect("read .env.example");

    let required_vars = [
        "APP_DATABASE_URL",
        "APP_SESSION_COOKIE_NAME",
        "APP_ADMIN_USERNAME",
        "APP_ADMIN_PASSWORD_HASH",
        "APP_SESSION_COOKIE_SECURE",
        "APP_EXCHANGE_BASE_URL",
    ];

    for var in required_vars {
        assert!(content.contains(var), ".env.example must contain {var}");
    }
}
