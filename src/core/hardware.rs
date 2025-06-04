use serde::{Serialize, Deserialize};
use std::collections::HashMap;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub system_info: SystemInfo,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub storage_devices: Vec<StorageDevice>,
    pub virtualization: Option<VirtualizationType>,
    pub thermal_sensors: Vec<ThermalSensor>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model_name: String,
    pub vendor: String,
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub frequency_mhz: f64,
    pub cache_sizes: HashMap<String, u64>, 
    pub features: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub speed_mhz: Option<u32>,
    pub ecc_enabled: Option<bool>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    pub name: String,
    pub model: String,
    pub device_type: DiskType,
    pub size_bytes: u64,
    pub mount_point: Option<String>,
    pub filesystem: Option<String>,
    pub smart_supported: bool,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiskType {
    Hdd,
    Ssd,
    Nvme,
    Unknown,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualizationType {
    None,
    Kvm,
    Vmware,
    Virtualbox,
    Xen,
    Hyperv,
    Lxc,
    Docker,
    Unknown,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalSensor {
    pub name: String,
    pub location: String,
    pub current_temp_celsius: f32,
    pub critical_temp_celsius: Option<f32>,
}


#[derive(Debug, Clone)]
pub struct SystemProfile {
    pub hardware_info: HardwareInfo,
}

impl SystemProfile {
    
    pub fn new(hardware_info: HardwareInfo) -> Self {
        Self { hardware_info }
    }
    
    
    pub fn optimize_test_config(&self, base_config: &crate::core::config::TestConfig) 
        -> crate::core::config::TestConfig {
        
        let mut optimized = base_config.clone();
        
        
        if let Some(virt_type) = &self.hardware_info.virtualization {
            match virt_type {
                VirtualizationType::Docker | VirtualizationType::Lxc => {
                    
                    optimized.stress_level = (f64::from(optimized.stress_level) * 0.7).round() as u8;
                    optimized.thermal_monitoring = false;
                }
                _ => {
                    
                    optimized.stress_level = (f64::from(optimized.stress_level) * 0.8).round() as u8;
                }
            }
        }
        
        
        let mem_info = &self.hardware_info.memory_info;
        let available_gb = mem_info.available_bytes as f64 / 1_073_741_824.0;
        if available_gb < 2.0 {
            
            optimized.memory_test_size_percent = 50;
        } else if available_gb < 8.0 {
            
            optimized.memory_test_size_percent = 70;
        }
        
        
        let cpu_info = &self.hardware_info.cpu_info;
        if optimized.threads == 0 {  
            
            optimized.threads = (cpu_info.logical_cores as f32 * 0.75).round() as u32;
            
            if optimized.threads == 0 {
                optimized.threads = 1;
            }
        }
        
        optimized
    }
}
