use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::path::PathBuf;

/// Stress test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Duration of the test
    pub duration: Duration,
    /// Stress level (1-10)
    pub stress_level: u8,
    /// Output format
    pub output_format: OutputFormat,
    /// Output file
    pub output_file: Option<PathBuf>,
    /// Thermal monitoring
    pub thermal_monitoring: bool,
    /// Verbose output
    pub verbose: bool,
    /// Quiet output
    pub quiet: bool,
    /// Number of threads
    pub threads: u32,
    /// Memory test size percentage
    pub memory_test_size_percent: u8,
    
    /// CPU test enabled
    pub cpu_enabled: bool,
    /// Memory test enabled
    pub memory_enabled: bool,
    /// Storage test enabled
    pub storage_enabled: bool,
    /// Network test enabled
    pub network_enabled: bool,
    /// Thermal test enabled
    pub thermal_enabled: bool,
    
    /// Storage test paths
    pub storage_test_paths: Vec<PathBuf>,
    /// Storage file size
    pub storage_file_size: u64,
    /// Thermal warning threshold
    pub thermal_warning_threshold: f32,
    /// Thermal critical threshold
    pub thermal_critical_threshold: f32,
    /// Thermal monitor interval
    pub thermal_monitor_interval: Duration,
    /// Alert threshold
    pub alert_threshold: u8,
    /// Alert webhook URL
    pub alert_webhook_url: Option<String>,
    /// Alert email
    pub alert_email: Option<String>,
}

/// Output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Text output
    Text,
    /// JSON output
    Json,
    /// CSV output
    Csv,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(2 * 60 * 60),
            stress_level: 8,
            output_format: OutputFormat::Text,
            output_file: None,
            thermal_monitoring: true,
            verbose: false,
            quiet: false,
            threads: 0, 
            memory_test_size_percent: 80,
            
            cpu_enabled: true,
            memory_enabled: true,
            storage_enabled: true,
            network_enabled: false, 
            thermal_enabled: true,
            
            storage_test_paths: Vec::new(), 
            storage_file_size: 1_073_741_824, 
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
    /// Create stress test configuration
    pub fn stress() -> Self {
        Self { 
            duration: Duration::from_secs(15 * 60),
            stress_level: 6,
            storage_file_size: 536_870_912,
            ..Default::default()
        }
    }
    
    /// Create burn-in test configuration
    pub fn burn_in() -> Self {
        Self {
            duration: Duration::from_secs(8 * 60 * 60),
            stress_level: 9,
            storage_file_size: 2_147_483_648,
            ..Default::default()
        }
    }
    
    /// Create quick test configuration
    pub fn quick() -> Self {
        Self {
            duration: Duration::from_secs(15 * 60),
            stress_level: 6,
            storage_file_size: 536_870_912,
            ..Default::default()
        }
    }
    
    /// Create standard test configuration
    pub fn standard() -> Self {
        Self::default()
    }
    
    /// Create full test configuration
    pub fn full() -> Self {
        Self {
            duration: Duration::from_secs(8 * 60 * 60),
            stress_level: 9,
            storage_file_size: 2_147_483_648,
            ..Default::default()
        }
    }
    
    /// Apply stress test preset
    pub fn apply_preset_stress(&mut self) {
        self.duration = Duration::from_secs(15 * 60);
        self.stress_level = 6;
        self.storage_file_size = 536_870_912; 
    }
    
    /// Apply burn-in test preset
    pub fn apply_preset_burn_in(&mut self) {
        self.duration = Duration::from_secs(8 * 60 * 60);
        self.stress_level = 9;
        self.storage_file_size = 2_147_483_648; 
    }
    
    /// Apply quick test preset
    pub fn apply_preset_quick(&mut self) {
        self.duration = Duration::from_secs(15 * 60);
        self.stress_level = 6;
        self.storage_file_size = 536_870_912; 
    }
    
    /// Apply standard test preset
    pub fn apply_preset_standard(&mut self) {
        self.duration = Duration::from_secs(2 * 60 * 60);
        self.stress_level = 8;
        self.storage_file_size = 1_073_741_824; 
    }
    
    /// Apply full test preset
    pub fn apply_preset_full(&mut self) {
        self.duration = Duration::from_secs(8 * 60 * 60);
        self.stress_level = 9;
        self.storage_file_size = 2_147_483_648; 
    }
    
    /// Parse duration string
    pub fn parse_duration(duration_str: &str) -> Result<Duration, String> {
        let duration = humantime::parse_duration(duration_str)
            .map_err(|e| format!("Invalid duration format: {}", e))?;
        
        if duration < Duration::from_secs(60) {
            return Err("Duration must be at least 1 minute".to_string());
        }
        if duration > Duration::from_secs(7 * 24 * 60 * 60) {
            return Err("Duration cannot exceed 7 days".to_string());
        }
        
        Ok(duration)
    }
    
    /// Parse size string
    pub fn parse_size_str(size_str: &str, total_size: u64) -> Result<u64, String> {
        if let Some(stripped) = size_str.strip_suffix('%') {
            let percent = stripped
                .parse::<f64>()
                .map_err(|_| format!("Invalid percentage: {}", size_str))?;
            if percent <= 0.0 || percent > 100.0 {
                return Err(format!("Percentage must be between 0 and 100: {}", percent));
            }
            Ok((total_size as f64 * percent / 100.0) as u64)
        } else {
            match size_str.parse::<bytesize::ByteSize>() {
                Ok(size) => Ok(size.as_u64()),
                Err(_) => Err(format!("Invalid size format: {}", size_str)),
            }
        }
    }
    
    /// Load configuration from file
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
            
        let config = if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            toml::from_str::<Self>(&contents)
                .map_err(|e| format!("Failed to parse TOML config: {}", e))?
        } else {
            serde_json::from_str::<Self>(&contents)
                .map_err(|e| format!("Failed to parse JSON config: {}", e))?
        };
        
        Ok(config)
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
    fn test_stress_preset() {
        let config = TestConfig::stress();
        assert_eq!(config.duration, Duration::from_secs(15 * 60));
        assert_eq!(config.stress_level, 6);
        assert_eq!(config.storage_file_size, 536_870_912);
    }

    #[test]
    fn test_burn_in_preset() {
        let config = TestConfig::burn_in();
        assert_eq!(config.duration, Duration::from_secs(8 * 60 * 60));
        assert_eq!(config.stress_level, 9);
        assert_eq!(config.storage_file_size, 2_147_483_648);
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
        assert!(TestConfig::parse_duration("30s").is_err()); 
        assert!(TestConfig::parse_duration("8d").is_err()); 
        assert!(TestConfig::parse_duration("invalid").is_err()); 
    }

    #[test]
    fn test_parse_size_str_percentage() {
        let size = TestConfig::parse_size_str("80%", 100).unwrap();
        assert_eq!(size, 80);
    }

    #[test]
    fn test_parse_size_str_bytes() {
        let size = TestConfig::parse_size_str("1GB", 100).unwrap();
        assert_eq!(size, 1_000_000_000);

        let size = TestConfig::parse_size_str("512MB", 100).unwrap();
        assert_eq!(size, 512_000_000);
    }

    #[test]
    fn test_parse_size_str_invalid() {
        assert!(TestConfig::parse_size_str("0%", 100).is_err());
        assert!(TestConfig::parse_size_str("101%", 100).is_err());
        assert!(TestConfig::parse_size_str("invalid", 100).is_err());
    }

    #[test]
    fn test_output_format_equality() {
        assert_eq!(OutputFormat::Text, OutputFormat::Text);
        assert_ne!(OutputFormat::Text, OutputFormat::Json);
    }

    #[test]
    fn test_apply_presets() {
        let mut config = TestConfig::default();
        
        config.apply_preset_stress();
        assert_eq!(config.duration, Duration::from_secs(15 * 60));
        
        config.apply_preset_burn_in();
        assert_eq!(config.duration, Duration::from_secs(8 * 60 * 60));
        
        config.apply_preset_quick();
        assert_eq!(config.duration, Duration::from_secs(15 * 60));
        
        config.apply_preset_standard();
        assert_eq!(config.duration, Duration::from_secs(2 * 60 * 60));
        
        config.apply_preset_full();
        assert_eq!(config.duration, Duration::from_secs(8 * 60 * 60));
    }
}
