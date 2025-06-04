use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde_json::json;
use sysinfo::{System, DiskKind, Disks};

use crate::core::hardware::{HardwareInfo, StorageDevice, StorageType};
use crate::core::test::{BurnInTest, TestResult, TestStatus, TestIssue, IssueSeverity};
use crate::core::config::TestConfig;
use crate::core::error::{Result, BurnInError};

/// Storage I/O test
pub struct StorageIoTest;

impl BurnInTest for StorageIoTest {
    fn name(&self) -> &'static str {
        "storage_io"
    }
    
    fn detect_hardware(&self) -> Result<HardwareInfo> {
        // Reuse hardware detection from CPU test for basic info
        let mut hardware_info = crate::tests::cpu::CpuStressTest.detect_hardware()?;
        
        // Add storage device detection
        let _system = System::new_all();
        // system.refresh_disks() is called by new_all()
        
        let mut storage_devices = Vec::new();
        
        // In sysinfo 0.30, we need to iterate over disks this way
        // In sysinfo 0.30, disks are accessed through a separate struct
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            let device_type = match disk.kind() {
                DiskKind::SSD => StorageType::SSD,
                DiskKind::HDD => StorageType::HDD,
                _ => StorageType::Unknown,
            };
            
            storage_devices.push(StorageDevice {
                name: disk.name().to_string_lossy().to_string(),
                model: "Unknown".to_string(), // Would need platform-specific code to detect
                device_type,
                size_bytes: disk.total_space(),
                mount_point: Some(disk.mount_point().to_string_lossy().to_string()),
                filesystem: Some(disk.file_system().to_string_lossy().to_string()),
                smart_supported: false, // Would need platform-specific code to detect
            });
        }
        
        hardware_info.storage_devices = storage_devices;
        
        Ok(hardware_info)
    }
    
    fn estimate_duration(&self, config: &TestConfig) -> Duration {
        config.duration
    }
    
    fn execute(&self, config: &TestConfig) -> Result<TestResult> {
        let start_time = Instant::now();
        
        // Determine test paths
        let test_paths = if config.storage_test_paths.is_empty() {
            // Auto-detect a suitable test path
            detect_test_paths()?
        } else {
            config.storage_test_paths.clone()
        };
        
        if test_paths.is_empty() {
            return Err(BurnInError::InsufficientResources(
                "No suitable storage paths found for testing".to_string(),
            ));
        }
        
        println!("Starting storage I/O test on paths: {:?}", test_paths);
        
        // Metrics collection
        let seq_read_mbps = Arc::new(Mutex::new(0.0));
        let seq_write_mbps = Arc::new(Mutex::new(0.0));
        let random_read_iops = Arc::new(Mutex::new(0.0));
        let random_write_iops = Arc::new(Mutex::new(0.0));
        let error_count = Arc::new(Mutex::new(0));
        
        // Determine file size for testing
        let file_size = config.storage_file_size;
        
        // Test each path
        let mut _all_successful = true;
        
        for path in &test_paths {
            // Create test file path
            let test_file = path.join("burnin_storage_test.tmp");
            
            // Sequential write test
            let seq_write_result = test_sequential_write(&test_file, file_size, seq_write_mbps.clone())?;
            _all_successful &= seq_write_result;
            
            // Sequential read test
            let seq_read_result = test_sequential_read(&test_file, file_size, seq_read_mbps.clone())?;
            _all_successful &= seq_read_result;
            
            // Random read test
            let rand_read_result = test_random_read(&test_file, file_size, random_read_iops.clone())?;
            _all_successful &= rand_read_result;
            
            // Random write test
            let rand_write_result = test_random_write(&test_file, file_size, random_write_iops.clone())?;
            _all_successful &= rand_write_result;
            
            // Metadata operations test
            let meta_result = test_metadata_operations(&test_file.parent().unwrap())?;
            _all_successful &= meta_result;
            
            // Clean up test file
            if test_file.exists() {
                if let Err(e) = fs::remove_file(&test_file) {
                    *error_count.lock().unwrap() += 1;
                    eprintln!("Failed to remove test file: {}", e);
                }
            }
        }
        
        // Calculate final metrics
        let final_seq_read = *seq_read_mbps.lock().unwrap();
        let final_seq_write = *seq_write_mbps.lock().unwrap();
        let final_rand_read = *random_read_iops.lock().unwrap();
        let final_rand_write = *random_write_iops.lock().unwrap();
        let final_error_count = *error_count.lock().unwrap();
        
        // Calculate score (0-100)
        let mut score = 100;
        
        // Penalize for errors
        score -= (final_error_count as u8 * 5).min(50);
        
        // Penalize for poor performance (simplified - in a real implementation you'd compare to expected values)
        if final_seq_read < 50.0 {
            score -= ((50.0 - final_seq_read) / 5.0).min(10.0) as u8;
        }
        
        if final_seq_write < 20.0 {
            score -= ((20.0 - final_seq_write) / 2.0).min(10.0) as u8;
        }
        
        if final_rand_read < 1000.0 {
            score -= ((1000.0 - final_rand_read) / 100.0).min(10.0) as u8;
        }
        
        if final_rand_write < 500.0 {
            score -= ((500.0 - final_rand_write) / 50.0).min(10.0) as u8;
        }
        
        // Create issues if any
        let mut issues = Vec::new();
        
        if final_error_count > 0 {
            issues.push(TestIssue {
                component: "storage".to_string(),
                severity: if final_error_count > 5 {
                    IssueSeverity::Critical
                } else {
                    IssueSeverity::High
                },
                message: format!("Storage I/O errors detected ({} errors)", final_error_count),
                action: Some("Check disk health and file system integrity".to_string()),
            });
        }
        
        if final_seq_read < 10.0 {
            issues.push(TestIssue {
                component: "storage".to_string(),
                severity: IssueSeverity::Medium,
                message: format!("Sequential read performance is very low ({:.2} MB/s)", final_seq_read),
                action: Some("Check for disk issues or resource contention".to_string()),
            });
        }
        
        if final_seq_write < 5.0 {
            issues.push(TestIssue {
                component: "storage".to_string(),
                severity: IssueSeverity::Medium,
                message: format!("Sequential write performance is very low ({:.2} MB/s)", final_seq_write),
                action: Some("Check for disk issues or resource contention".to_string()),
            });
        }
        
        // Create test result
        let result = TestResult {
            name: self.name().to_string(),
            status: if issues.iter().any(|i| i.severity == IssueSeverity::Critical) {
                TestStatus::Failed
            } else {
                TestStatus::Completed
            },
            score,
            duration: start_time.elapsed(),
            metrics: json!({
                "sequential_read_mbps": final_seq_read,
                "sequential_write_mbps": final_seq_write,
                "random_read_iops": final_rand_read,
                "random_write_iops": final_rand_write,
                "error_count": final_error_count,
                "test_file_size_bytes": file_size,
            }),
            issues,
        };
        
        Ok(result)
    }
    
    fn cleanup(&self) -> Result<()> {
        // Clean up any remaining test files
        let test_paths = detect_test_paths()?;
        
        for path in &test_paths {
            let test_file = path.join("burnin_storage_test.tmp");
            if test_file.exists() {
                if let Err(e) = fs::remove_file(&test_file) {
                    eprintln!("Failed to remove test file during cleanup: {}", e);
                }
            }
        }
        
        Ok(())
    }
}

// Helper functions for storage testing

fn detect_test_paths() -> Result<Vec<PathBuf>> {
    // List of potential temporary directories
    let temp_dir = std::env::temp_dir();
    let temp_dir_str = temp_dir.to_str().unwrap_or("/tmp");
    
    let tmp_dirs = [
        "/tmp",
        "/var/tmp",
        temp_dir_str,
    ];
    
    let mut paths = Vec::new();
    
    for dir in &tmp_dirs {
        let path = PathBuf::from(dir);
        if path.exists() && path.is_dir() && is_writable(&path) {
            paths.push(path);
            break; // One path is enough for testing
        }
    }
    
    // If no suitable path found, try current directory
    if paths.is_empty() {
        let current_dir = std::env::current_dir()
            .map_err(|e| BurnInError::IoError(e))?;
        
        if is_writable(&current_dir) {
            paths.push(current_dir);
        }
    }
    
    Ok(paths)
}

fn is_writable(path: &Path) -> bool {
    // Try to create and remove a test file
    let test_file = path.join(".burnin_write_test");
    match File::create(&test_file) {
        Ok(_) => {
            let _ = fs::remove_file(&test_file);
            true
        }
        Err(_) => false,
    }
}

fn test_sequential_write(
    path: &Path,
    size: u64,
    mbps: Arc<Mutex<f64>>,
) -> Result<bool> {
    // Create file
    let file = File::create(path).map_err(|e| BurnInError::IoError(e))?;
    
    // Prepare buffer (1MB)
    let buffer_size = 1024 * 1024;
    let buffer = vec![0u8; buffer_size];
    
    // Write data
    let start_time = Instant::now();
    let mut writer = io::BufWriter::new(file);
    let mut remaining = size;
    
    while remaining > 0 {
        let to_write = buffer_size.min(remaining as usize);
        writer.write_all(&buffer[..to_write])
            .map_err(|e| BurnInError::IoError(e))?;
        remaining -= to_write as u64;
    }
    
    // Flush to ensure data is written
    writer.flush().map_err(|e| BurnInError::IoError(e))?;
    
    // Calculate throughput
    let elapsed = start_time.elapsed();
    let throughput = (size as f64 / 1_000_000.0) / elapsed.as_secs_f64();
    
    let mut m = mbps.lock().unwrap();
    *m = throughput;
    
    Ok(true)
}

fn test_sequential_read(
    path: &Path,
    size: u64,
    mbps: Arc<Mutex<f64>>,
) -> Result<bool> {
    // Open file
    let file = File::open(path).map_err(|e| BurnInError::IoError(e))?;
    
    // Prepare buffer (1MB)
    let buffer_size = 1024 * 1024;
    let mut buffer = vec![0u8; buffer_size];
    
    // Read data
    let start_time = Instant::now();
    let mut reader = io::BufReader::new(file);
    let mut remaining = size;
    
    while remaining > 0 {
        let to_read = buffer_size.min(remaining as usize);
        match reader.read_exact(&mut buffer[..to_read]) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(BurnInError::IoError(e)),
        }
        remaining -= to_read as u64;
    }
    
    // Calculate throughput
    let elapsed = start_time.elapsed();
    let throughput = ((size - remaining) as f64 / 1_000_000.0) / elapsed.as_secs_f64();
    
    let mut m = mbps.lock().unwrap();
    *m = throughput;
    
    Ok(true)
}

fn test_random_read(
    path: &Path,
    size: u64,
    iops: Arc<Mutex<f64>>,
) -> Result<bool> {
    // Open file
    let mut file = File::open(path).map_err(|e| BurnInError::IoError(e))?;
    
    // Prepare buffer (4KB)
    let buffer_size = 4 * 1024;
    let mut buffer = vec![0u8; buffer_size];
    
    // Generate random positions
    let mut rng = StdRng::seed_from_u64(42);
    let max_pos = size.saturating_sub(buffer_size as u64);
    let num_ops = 10000.min(size / buffer_size as u64);
    
    // Read data from random positions
    let start_time = Instant::now();
    let mut ops_completed = 0;
    
    for _ in 0..num_ops {
        let pos = rng.gen_range(0..=max_pos);
        file.seek(SeekFrom::Start(pos)).map_err(|e| BurnInError::IoError(e))?;
        
        match file.read_exact(&mut buffer) {
            Ok(_) => ops_completed += 1,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(BurnInError::IoError(e)),
        }
    }
    
    // Calculate IOPS
    let elapsed = start_time.elapsed();
    let ops_per_sec = ops_completed as f64 / elapsed.as_secs_f64();
    
    let mut i = iops.lock().unwrap();
    *i = ops_per_sec;
    
    Ok(true)
}

fn test_random_write(
    path: &Path,
    size: u64,
    iops: Arc<Mutex<f64>>,
) -> Result<bool> {
    // Open file
    let mut file = OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| BurnInError::IoError(e))?;
    
    // Prepare buffer (4KB)
    let buffer_size = 4 * 1024;
    let buffer = vec![0u8; buffer_size];
    
    // Generate random positions
    let mut rng = StdRng::seed_from_u64(43);
    let max_pos = size.saturating_sub(buffer_size as u64);
    let num_ops = 5000.min(size / buffer_size as u64);
    
    // Write data to random positions
    let start_time = Instant::now();
    let mut ops_completed = 0;
    
    for _ in 0..num_ops {
        let pos = rng.gen_range(0..=max_pos);
        file.seek(SeekFrom::Start(pos)).map_err(|e| BurnInError::IoError(e))?;
        
        if let Ok(_) = file.write_all(&buffer) {
            ops_completed += 1;
        }
    }
    
    // Flush to ensure data is written
    file.flush().map_err(|e| BurnInError::IoError(e))?;
    
    // Calculate IOPS
    let elapsed = start_time.elapsed();
    let ops_per_sec = ops_completed as f64 / elapsed.as_secs_f64();
    
    let mut i = iops.lock().unwrap();
    *i = ops_per_sec;
    
    Ok(true)
}

fn test_metadata_operations(path: &Path) -> Result<bool> {
    // Create a directory for metadata testing
    let test_dir = path.with_file_name("burnin_metadata_test");
    fs::create_dir_all(&test_dir).map_err(|e| BurnInError::IoError(e))?;
    
    // Create some files
    for i in 0..100 {
        let file_path = test_dir.join(format!("file_{}.txt", i));
        let mut file = File::create(&file_path).map_err(|e| BurnInError::IoError(e))?;
        file.write_all(b"test").map_err(|e| BurnInError::IoError(e))?;
    }
    
    // List files
    let entries = fs::read_dir(&test_dir).map_err(|e| BurnInError::IoError(e))?;
    let mut count = 0;
    
    for entry in entries {
        let entry = entry.map_err(|e| BurnInError::IoError(e))?;
        let metadata = entry.metadata().map_err(|e| BurnInError::IoError(e))?;
        
        if metadata.is_file() {
            count += 1;
        }
    }
    
    // Verify count
    let success = count == 100;
    
    // Clean up
    fs::remove_dir_all(&test_dir).map_err(|e| BurnInError::IoError(e))?;
    
    Ok(success)
}
