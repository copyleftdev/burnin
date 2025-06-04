use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Hardware information detected by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub system_info: SystemInfo,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub storage_devices: Vec<StorageDevice>,
    pub virtualization: Option<VirtualizationType>,
    pub thermal_sensors: Vec<ThermalSensor>,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
}

/// CPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model_name: String,
    pub vendor: String,
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub frequency_mhz: f64,
    pub cache_sizes: HashMap<String, u64>, // L1, L2, L3 in bytes
    pub features: Vec<String>,
}

/// Memory information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub speed_mhz: Option<u32>,
    pub ecc_enabled: Option<bool>,
}

/// Storage device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    pub name: String,
    pub model: String,
    pub device_type: StorageType,
    pub size_bytes: u64,
    pub mount_point: Option<String>,
    pub filesystem: Option<String>,
    pub smart_supported: bool,
}

/// Storage device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageType {
    HDD,
    SSD,
    NVMe,
    Virtual,
    Unknown,
}

/// Virtualization type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualizationType {
    KVM,
    VMware,
    VirtualBox,
    Xen,
    HyperV,
    Docker,
    LXC,
    Unknown,
}

/// Thermal sensor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalSensor {
    pub name: String,
    pub location: String,
    pub current_temp_celsius: f32,
    pub critical_temp_celsius: Option<f32>,
}

/// System profile for adaptive test configuration
#[derive(Debug, Clone)]
pub struct SystemProfile {
    pub hardware_info: HardwareInfo,
}

impl SystemProfile {
    /// Create a new system profile from hardware information
    pub fn new(hardware_info: HardwareInfo) -> Self {
        Self { hardware_info }
    }
    
    /// Optimize test configuration based on detected hardware
    pub fn optimize_test_config(&self, base_config: &crate::core::config::TestConfig) 
        -> crate::core::config::TestConfig {
        // Clone the base configuration
        let mut optimized = base_config.clone();
        
        // Adjust based on virtualization
        if let Some(virt_type) = &self.hardware_info.virtualization {
            match virt_type {
                VirtualizationType::Docker | VirtualizationType::LXC => {
                    // Container-specific optimizations
                    optimized.stress_level = (f64::from(optimized.stress_level) * 0.7).round() as u8;
                    optimized.thermal_monitoring = false;
                }
                _ => {
                    // VM-specific optimizations
                    optimized.stress_level = (f64::from(optimized.stress_level) * 0.8).round() as u8;
                }
            }
        }
        
        // Adjust based on available memory
        let mem_info = &self.hardware_info.memory_info;
        let available_gb = mem_info.available_bytes as f64 / 1_073_741_824.0;
        if available_gb < 2.0 {
            // Very limited memory
            optimized.memory_test_size_percent = 50;
        } else if available_gb < 8.0 {
            // Moderate memory
            optimized.memory_test_size_percent = 70;
        }
        
        // Adjust thread count based on CPU cores
        let cpu_info = &self.hardware_info.cpu_info;
        if optimized.threads == 0 {  // Auto mode
            // Use 75% of logical cores by default
            optimized.threads = (cpu_info.logical_cores as f32 * 0.75).round() as u32;
            // Ensure at least 1 thread
            if optimized.threads == 0 {
                optimized.threads = 1;
            }
        }
        
        optimized
    }
}
