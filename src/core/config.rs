use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::path::PathBuf;

/// Test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    // Global settings
    pub duration: Duration,
    pub stress_level: u8,
    pub output_format: OutputFormat,
    pub output_file: Option<PathBuf>,
    pub thermal_monitoring: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub threads: u32,
    pub memory_test_size_percent: u8,
    
    // Component-specific settings
    pub cpu_enabled: bool,
    pub memory_enabled: bool,
    pub storage_enabled: bool,
    pub network_enabled: bool,
    pub thermal_enabled: bool,
    
    // Advanced settings
    pub storage_test_paths: Vec<PathBuf>,
    pub storage_file_size: u64,
    pub thermal_warning_threshold: f32,
    pub thermal_critical_threshold: f32,
    pub thermal_monitor_interval: Duration,
    pub alert_threshold: u8,
    pub alert_webhook_url: Option<String>,
    pub alert_email: Option<String>,
}

/// Output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            // Default to standard 2-hour burn-in
            duration: Duration::from_secs(2 * 60 * 60),
            stress_level: 8,
            output_format: OutputFormat::Text,
            output_file: None,
            thermal_monitoring: true,
            verbose: false,
            quiet: false,
            threads: 0, // Auto-detect
            memory_test_size_percent: 80,
            
            // Enable all components by default
            cpu_enabled: true,
            memory_enabled: true,
            storage_enabled: true,
            network_enabled: false, // Disabled by default
            thermal_enabled: true,
            
            // Advanced settings with reasonable defaults
            storage_test_paths: Vec::new(), // Auto-detect
            storage_file_size: 1_073_741_824, // 1GB
            thermal_warning_threshold: 80.0,
            thermal_critical_threshold: 90.0,
            thermal_monitor_interval: Duration::from_secs(5),
            alert_threshold: 95,
            alert_webhook_url: None,
            alert_email: None,
        }
    }
}

impl TestConfig {
    /// Create a quick validation configuration (15 minutes)
    pub fn quick() -> Self {
        let mut config = Self::default();
        config.duration = Duration::from_secs(15 * 60);
        config.stress_level = 6;
        config.storage_file_size = 536_870_912; // 512MB
        config
    }
    
    /// Create a standard burn-in configuration (2 hours)
    pub fn standard() -> Self {
        Self::default()
    }
    
    /// Create a full burn-in configuration (8 hours)
    pub fn full() -> Self {
        let mut config = Self::default();
        config.duration = Duration::from_secs(8 * 60 * 60);
        config.stress_level = 9;
        config.storage_file_size = 2_147_483_648; // 2GB
        config
    }
    
    /// Apply quick preset to existing config
    pub fn apply_preset_quick(&mut self) {
        self.duration = Duration::from_secs(15 * 60);
        self.stress_level = 6;
        self.storage_file_size = 536_870_912; // 512MB
    }
    
    /// Apply standard preset to existing config
    pub fn apply_preset_standard(&mut self) {
        self.duration = Duration::from_secs(2 * 60 * 60);
        self.stress_level = 8;
        self.storage_file_size = 1_073_741_824; // 1GB
    }
    
    /// Apply full preset to existing config
    pub fn apply_preset_full(&mut self) {
        self.duration = Duration::from_secs(8 * 60 * 60);
        self.stress_level = 9;
        self.storage_file_size = 2_147_483_648; // 2GB
    }
    
    /// Parse a duration string like "30m", "2h", "1d"
    pub fn parse_duration(duration_str: &str) -> Result<Duration, String> {
        let duration = humantime::parse_duration(duration_str)
            .map_err(|e| format!("Invalid duration format: {}", e))?;
        
        // Ensure duration is reasonable (between 1 minute and 7 days)
        if duration < Duration::from_secs(60) {
            return Err("Duration must be at least 1 minute".to_string());
        }
        if duration > Duration::from_secs(7 * 24 * 60 * 60) {
            return Err("Duration cannot exceed 7 days".to_string());
        }
        
        Ok(duration)
    }
    
    /// Load configuration from a file
    pub fn from_file(path: &str) -> Result<Self, String> {
        use std::fs;
        use std::io::Read;
        use std::path::Path;
        
        let path = Path::new(path);
        if !path.exists() {
            return Err(format!("Config file not found: {}", path.display()));
        }
        
        let mut file = fs::File::open(path)
            .map_err(|e| format!("Failed to open config file: {}", e))?;
            
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
            
        // Parse TOML or JSON based on file extension
        let config = if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            toml::from_str::<Self>(&contents)
                .map_err(|e| format!("Failed to parse TOML config: {}", e))?
        } else {
            serde_json::from_str::<Self>(&contents)
                .map_err(|e| format!("Failed to parse JSON config: {}", e))?
        };
        
        Ok(config)
    }
    
    /// Parse a memory size string like "80%", "4G", "512M"
    pub fn parse_memory_size(size_str: &str) -> Result<(bool, u64), String> {
        if size_str.ends_with('%') {
            let percent = size_str[..size_str.len() - 1]
                .parse::<u8>()
                .map_err(|_| format!("Invalid percentage: {}", size_str))?;
            
            if percent == 0 || percent > 95 {
                return Err("Percentage must be between 1% and 95%".to_string());
            }
            
            return Ok((true, percent as u64));
        }
        
        // Parse size string using bytesize crate
        let bytes = match size_str.parse::<bytesize::ByteSize>() {
            Ok(size) => size.as_u64(),
            Err(_) => return Err(format!("Invalid size format: {}", size_str)),
        };
        
        Ok((false, bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TestConfig::default();
        assert_eq!(config.duration, Duration::from_secs(2 * 60 * 60));
        assert_eq!(config.stress_level, 8);
        assert!(config.cpu_enabled);
        assert!(config.memory_enabled);
        assert!(config.storage_enabled);
        assert!(!config.network_enabled);
        assert!(config.thermal_enabled);
    }

    #[test]
    fn test_quick_preset() {
        let config = TestConfig::quick();
        assert_eq!(config.duration, Duration::from_secs(15 * 60));
        assert_eq!(config.stress_level, 6);
        assert_eq!(config.storage_file_size, 536_870_912);
    }

    #[test]
    fn test_full_preset() {
        let config = TestConfig::full();
        assert_eq!(config.duration, Duration::from_secs(8 * 60 * 60));
        assert_eq!(config.stress_level, 9);
        assert_eq!(config.storage_file_size, 2_147_483_648);
    }

    #[test]
    fn test_parse_duration_valid() {
        assert_eq!(TestConfig::parse_duration("30m").unwrap(), Duration::from_secs(30 * 60));
        assert_eq!(TestConfig::parse_duration("2h").unwrap(), Duration::from_secs(2 * 60 * 60));
        assert_eq!(TestConfig::parse_duration("1d").unwrap(), Duration::from_secs(24 * 60 * 60));
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(TestConfig::parse_duration("30s").is_err()); // Too short
        assert!(TestConfig::parse_duration("8d").is_err()); // Too long
        assert!(TestConfig::parse_duration("invalid").is_err()); // Invalid format
    }

    #[test]
    fn test_parse_memory_size_percentage() {
        let (is_percent, value) = TestConfig::parse_memory_size("80%").unwrap();
        assert!(is_percent);
        assert_eq!(value, 80);
    }

    #[test]
    fn test_parse_memory_size_bytes() {
        let (is_percent, value) = TestConfig::parse_memory_size("1GB").unwrap();
        assert!(!is_percent);
        assert_eq!(value, 1_000_000_000);

        let (is_percent, value) = TestConfig::parse_memory_size("512MB").unwrap();
        assert!(!is_percent);
        assert_eq!(value, 512_000_000);
    }

    #[test]
    fn test_parse_memory_size_invalid() {
        assert!(TestConfig::parse_memory_size("0%").is_err());
        assert!(TestConfig::parse_memory_size("100%").is_err());
        assert!(TestConfig::parse_memory_size("invalid").is_err());
    }

    #[test]
    fn test_output_format_equality() {
        assert_eq!(OutputFormat::Text, OutputFormat::Text);
        assert_ne!(OutputFormat::Text, OutputFormat::Json);
    }

    #[test]
    fn test_apply_presets() {
        let mut config = TestConfig::default();
        
        config.apply_preset_quick();
        assert_eq!(config.duration, Duration::from_secs(15 * 60));
        
        config.apply_preset_full();
        assert_eq!(config.duration, Duration::from_secs(8 * 60 * 60));
        
        config.apply_preset_standard();
        assert_eq!(config.duration, Duration::from_secs(2 * 60 * 60));
    }
}
