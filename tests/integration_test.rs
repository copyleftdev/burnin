use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A lightweight, single-binary CLI tool for system burn-in testing"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("quick"));
    assert!(stdout.contains("standard"));
    assert!(stdout.contains("full"));
}

#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("burnin"));
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_hardware_info() {
    let output = Command::new("cargo")
        .args(&["run", "--", "hardware"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("System Hardware Information"));
    assert!(stdout.contains("CPU Information"));
    assert!(stdout.contains("Memory Information"));
}

#[test]
fn test_custom_short_duration() {
    let output = Command::new("cargo")
        .args(&["run", "--release", "--", "custom", "--duration", "1s", "--components", "cpu"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BURN-IN TEST STARTING"));
    assert!(stdout.contains("cpu_stress"));
    assert!(stdout.contains("PASS") || stdout.contains("FAIL"));
}

#[test]
fn test_invalid_duration() {
    let output = Command::new("cargo")
        .args(&["run", "--", "custom", "--duration", "invalid", "--components", "cpu"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid duration format") || stderr.contains("error"));
}

#[test]
fn test_config_parsing() {
    use burnin::core::config::{TestConfig, OutputFormat};
    
    let config = TestConfig::default();
    assert_eq!(config.stress_level, 8);
    assert_eq!(config.output_format, OutputFormat::Text);
    assert!(config.cpu_enabled);
    assert!(config.memory_enabled);
    assert!(config.storage_enabled);
    assert!(!config.network_enabled);
}

#[test]
fn test_duration_parsing() {
    use burnin::core::config::TestConfig;
    
    
    assert!(TestConfig::parse_duration("5m").is_ok());
    assert!(TestConfig::parse_duration("2h").is_ok());
    assert!(TestConfig::parse_duration("1d").is_ok());
    
    
    assert!(TestConfig::parse_duration("30s").is_err()); 
    assert!(TestConfig::parse_duration("8d").is_err()); 
    assert!(TestConfig::parse_duration("xyz").is_err()); 
}

#[test]
fn test_memory_size_parsing() {
    use burnin::core::config::TestConfig;
    
    // Test percentage parsing
    let value = TestConfig::parse_size_str("80%", 100).unwrap();
    assert_eq!(value, 80);
    
    // Test absolute value parsing  
    let value = TestConfig::parse_size_str("1GB", 1_000_000_000).unwrap();
    assert_eq!(value, 1_000_000_000);
    
    // Test invalid parsing
    assert!(TestConfig::parse_size_str("0%", 100).is_err());
    assert!(TestConfig::parse_size_str("101%", 100).is_err());
}
