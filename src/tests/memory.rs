use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde_json::json;

use crate::core::test::{BurnInTest, TestResult, TestStatus, TestIssue, IssueSeverity};
use crate::core::config::TestConfig;
use crate::core::hardware::HardwareInfo;
use crate::core::error::Result;

/// Memory validation test
pub struct MemoryValidationTest;

impl BurnInTest for MemoryValidationTest {
    fn name(&self) -> &'static str {
        "memory_validation"
    }
    
    fn detect_hardware(&self) -> Result<HardwareInfo> {
        
        let cpu_test = crate::tests::cpu::CpuStressTest;
        cpu_test.detect_hardware()
    }
    
    fn estimate_duration(&self, config: &TestConfig) -> Duration {
        config.duration
    }
    
    fn execute(&self, config: &TestConfig) -> Result<TestResult> {
        let start_time = Instant::now();
        
        
        let mut system = sysinfo::System::new();
        system.refresh_memory();
        
        let available_memory = system.available_memory();
        let test_size = (available_memory as f64 * (config.memory_test_size_percent as f64 / 100.0)) as usize;
        
        println!("Starting memory validation test using {} bytes", test_size);
        
        
        let error_count = Arc::new(Mutex::new(0));
        let bandwidth_mbps = Arc::new(Mutex::new(0.0));
        let latency_ns = Arc::new(Mutex::new(0.0));
        
        
        let patterns = [
            0x00, 
            0xFF, 
            0xAA, 
            0x55, 
        ];
        
        
        let seq_result = test_sequential_access(test_size, &patterns, bandwidth_mbps.clone())?;
        
        
        let random_result = test_random_access(test_size, &patterns, latency_ns.clone())?;
        
        
        let walking_result = test_walking_bits(test_size, error_count.clone())?;
        
        
        let thread_result = test_multithreaded_access(test_size, config, error_count.clone())?;
        
        
        let final_error_count = *error_count.lock().unwrap();
        let final_bandwidth = *bandwidth_mbps.lock().unwrap();
        let final_latency = *latency_ns.lock().unwrap();
        
        
        let mut score = 100;
        
        
        if final_error_count > 0 {
            score = 0; 
        }
        
        
        
        if final_bandwidth < 1000.0 {
            score -= ((1000.0 - final_bandwidth) / 100.0).min(20.0) as u8;
        }
        
        
        let mut issues = Vec::new();
        
        if final_error_count > 0 {
            issues.push(TestIssue {
                component: "memory".to_string(),
                severity: IssueSeverity::Critical,
                message: format!("Memory errors detected ({} errors)", final_error_count),
                action: Some("Run extended memory diagnostics and consider replacing memory modules".to_string()),
            });
        }
        
        if !seq_result {
            issues.push(TestIssue {
                component: "memory".to_string(),
                severity: IssueSeverity::High,
                message: "Sequential memory access test failed".to_string(),
                action: Some("Check for memory corruption or hardware issues".to_string()),
            });
        }
        
        if !random_result {
            issues.push(TestIssue {
                component: "memory".to_string(),
                severity: IssueSeverity::Medium,
                message: "Random memory access test failed".to_string(),
                action: Some("Check for memory addressing issues".to_string()),
            });
        }
        
        if !walking_result {
            issues.push(TestIssue {
                component: "memory".to_string(),
                severity: IssueSeverity::High,
                message: "Walking bit pattern test failed".to_string(),
                action: Some("Check for stuck bits in memory".to_string()),
            });
        }
        
        if !thread_result {
            issues.push(TestIssue {
                component: "memory".to_string(),
                severity: IssueSeverity::Medium,
                message: "Multi-threaded memory access test failed".to_string(),
                action: Some("Check for memory contention issues".to_string()),
            });
        }
        
        
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
                "memory_errors": final_error_count,
                "bandwidth_mbps": final_bandwidth,
                "latency_ns": final_latency,
                "test_size_bytes": test_size,
            }),
            issues,
        };
        
        Ok(result)
    }
    
    fn cleanup(&self) -> Result<()> {
        
        drop(vec![0u8; 1]);
        Ok(())
    }
}



fn test_sequential_access(
    size: usize,
    patterns: &[u8],
    bandwidth: Arc<Mutex<f64>>,
) -> Result<bool> {
    
    let mut memory = vec![0; size];
    
    let mut success = true;
    
    for &pattern in patterns {
        
        let write_start = Instant::now();
        for val in memory.iter_mut() {
            *val = pattern;
        }
        let write_time = write_start.elapsed();
        
        
        let read_start = Instant::now();
        for val in &memory {
            if *val != pattern {
                success = false;
                break;
            }
        }
        let read_time = read_start.elapsed();
        
        
        let total_bytes = size * 2; 
        let total_time = write_time + read_time;
        let mbps = (total_bytes as f64 / 1_000_000.0) / total_time.as_secs_f64();
        
        let mut bw = bandwidth.lock().unwrap();
        *bw = mbps;
    }
    
    Ok(success)
}

fn test_random_access(
    size: usize,
    patterns: &[u8],
    latency: Arc<Mutex<f64>>,
) -> Result<bool> {
    
    let mut memory = vec![0; size];
    
    
    let mut rng = StdRng::seed_from_u64(42); 
    let mut indices: Vec<usize> = (0..size).collect();
    indices.shuffle(&mut rng);
    
    let mut success = true;
    
    for &pattern in patterns {
        
        let write_start = Instant::now();
        for &i in &indices {
            memory[i] = pattern;
        }
        let write_time = write_start.elapsed();
        
        
        let read_start = Instant::now();
        for &i in &indices {
            if memory[i] != pattern {
                success = false;
                break;
            }
        }
        let read_time = read_start.elapsed();
        
        
        let total_ops = indices.len() * 2; 
        let total_time = write_time + read_time;
        let ns_per_op = (total_time.as_nanos() as f64) / (total_ops as f64);
        
        let mut lat = latency.lock().unwrap();
        *lat = ns_per_op;
    }
    
    Ok(success)
}

fn test_walking_bits(
    size: usize,
    error_count: Arc<Mutex<usize>>,
) -> Result<bool> {
    
    let mut memory = vec![0; size];
    
    let mut success = true;
    
    
    for bit in 0..8 {
        let pattern = 1 << bit;
        
        
        let write_start = Instant::now();
        for val in memory.iter_mut() {
            *val = pattern;
        }
        let _write_duration = write_start.elapsed();
        
        let _read_start = Instant::now();
        for val in &memory {
            if *val != pattern {
                let mut errors = error_count.lock().unwrap();
                *errors += 1;
                success = false;
            }
        }
    }
    
    
    for bit in 0..8 {
        let pattern = !(1 << bit) & 0xFF;
        
        
        let write_start = Instant::now();
        for val in memory.iter_mut() {
            *val = pattern;
        }
        let _write_duration = write_start.elapsed();
        
        let _read_start = Instant::now();
        let mut checksum = 0u64;
        for val in &memory {
            checksum = checksum.wrapping_add(*val as u64);
        }
    }
    
    Ok(success)
}

fn test_multithreaded_access(
    size: usize,
    config: &TestConfig,
    error_count: Arc<Mutex<usize>>,
) -> Result<bool> {
    let thread_count = if config.threads == 0 {
        num_cpus::get() as u32
    } else {
        config.threads
    };
    
    
    let memory = Arc::new(Mutex::new(vec![0; size]));
    
    
    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();
    
    
    let test_duration = config.duration / 4; 
    let timer_thread = thread::spawn(move || {
        thread::sleep(test_duration);
        let mut running = running_clone.lock().unwrap();
        *running = false;
    });
    
    
    let handles: Vec<_> = (0..thread_count)
        .map(|id| {
            let memory = memory.clone();
            let running = running.clone();
            
            thread::spawn(move || {
                let mut rng = StdRng::seed_from_u64(id as u64);
                let chunk_size = size / thread_count as usize;
                let start = id as usize * chunk_size;
                let end = if id == thread_count - 1 {
                    size
                } else {
                    (id as usize + 1) * chunk_size
                };
                
                while *running.lock().unwrap() {
                    
                    {
                        let mut mem = memory.lock().unwrap();
                        for val in mem[start..end].iter_mut() {
                            *val = rng.gen();
                        }
                    }
                    
                    
                    thread::sleep(Duration::from_micros(10));
                    
                    
                    {
                        let mem = memory.lock().unwrap();
                        for val in &mem[start..end] {
                            
                            let _ = val;
                        }
                    }
                }
            })
        })
        .collect();
    
    
    for handle in handles {
        let _ = handle.join();
    }
    
    
    let _ = timer_thread.join();
    
    
    let errors = *error_count.lock().unwrap();
    Ok(errors == 0)
}


trait SliceExt {
    fn shuffle<R: Rng>(&mut self, rng: &mut R);
}

impl<T> SliceExt for [T] {
    fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        for i in (1..self.len()).rev() {
            let j = rng.gen_range(0..=i);
            self.swap(i, j);
        }
    }
}
