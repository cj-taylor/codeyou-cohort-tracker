use cohort_tracker::config::Config;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_config_serialization() {
    let config = Config {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        api_base: "https://api.openclass.ai".to_string(),
        check_for_updates: true,
    };

    let toml_str = toml::to_string(&config).unwrap();
    let parsed: Config = toml::from_str(&toml_str).unwrap();

    assert_eq!(config.email, parsed.email);
    assert_eq!(config.password, parsed.password);
    assert_eq!(config.api_base, parsed.api_base);
}

#[test]
fn test_config_save_and_load() {
    let config = Config {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        api_base: "https://api.openclass.ai".to_string(),
        check_for_updates: true,
    };

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Save config
    config.save(path).unwrap();

    // Load config
    let loaded = Config::from_file(path).unwrap();

    assert_eq!(config.email, loaded.email);
    assert_eq!(config.password, loaded.password);
    assert_eq!(config.api_base, loaded.api_base);
}

#[test]
fn test_config_load_nonexistent_file() {
    let result = Config::from_file("/nonexistent/path.toml");
    assert!(result.is_err());
}

#[test]
fn test_config_load_invalid_toml() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    fs::write(path, "invalid toml content [[[").unwrap();

    let result = Config::from_file(path);
    assert!(result.is_err());
}
