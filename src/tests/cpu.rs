use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;
use serde_json::json;
use sysinfo::System;

use crate::core::test::{BurnInTest, TestResult, TestStatus, TestIssue, IssueSeverity};
use crate::core::config::TestConfig;
use crate::core::hardware::{HardwareInfo, CpuInfo};
use crate::core::error::Result;

/// CPU stress test
pub struct CpuStressTest;

impl BurnInTest for CpuStressTest {
    fn name(&self) -> &'static str {
        "cpu_stress"
    }
    
    fn detect_hardware(&self) -> Result<HardwareInfo> {
        // Use sysinfo to detect CPU information
        let mut system = System::new_all();
        system.refresh_cpu();
        
        let cpu_vendor = system.global_cpu_info().vendor_id().to_string();
        let cpu_name = system.global_cpu_info().brand().to_string();
        
        // Create CPU info
        let cpu_info = CpuInfo {
            model_name: cpu_name,
            vendor: cpu_vendor,
            physical_cores: num_cpus::get_physical() as u32,
            logical_cores: num_cpus::get() as u32,
            frequency_mhz: system.global_cpu_info().frequency() as f64,
            cache_sizes: std::collections::HashMap::new(), // Would need platform-specific code to detect
            features: Vec::new(), // Would need platform-specific code to detect
        };
        
        // Create hardware info (partial, just CPU for this test)
        let hardware_info = HardwareInfo {
            system_info: crate::core::hardware::SystemInfo {
                hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
                os_name: System::name().unwrap_or_else(|| "unknown".to_string()),
                os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
                kernel_version: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
            },
            cpu_info,
            memory_info: crate::core::hardware::MemoryInfo {
                total_bytes: system.total_memory(),
                available_bytes: system.available_memory(),
                speed_mhz: None,
                ecc_enabled: None,
            },
            storage_devices: Vec::new(),
            virtualization: None, // Would need platform-specific code to detect
            thermal_sensors: Vec::new(),
        };
        
        Ok(hardware_info)
    }
    
    fn estimate_duration(&self, config: &TestConfig) -> Duration {
        config.duration
    }
    
    fn execute(&self, config: &TestConfig) -> Result<TestResult> {
        let start_time = Instant::now();
        let thread_count = if config.threads == 0 {
            num_cpus::get() as u32
        } else {
            config.threads
        };
        
        println!("Starting CPU stress test with {} threads for {:?}", thread_count, config.duration);
        
        // Metrics collection
        let utilization = Arc::new(Mutex::new(0.0));
        let throttling_events = Arc::new(Mutex::new(0));
        let instructions_per_sec = Arc::new(Mutex::new(0u64));
        
        // Create a flag to signal threads to stop
        let running = Arc::new(Mutex::new(true));
        let running_clone = running.clone();
        
        // Set up a timer to stop the test after the configured duration
        let test_duration = config.duration;
        let timer_thread = thread::spawn(move || {
            thread::sleep(test_duration);
            let mut running = running_clone.lock().unwrap();
            *running = false;
        });
        
        // Start CPU stress test threads
        let handles: Vec<_> = (0..thread_count)
            .map(|id| {
                let running = running.clone();
                let utilization = utilization.clone();
                let throttling_events = throttling_events.clone();
                let instructions_per_sec = instructions_per_sec.clone();
                
                thread::spawn(move || {
                    // Different workload types based on thread ID
                    let workload_type = id % 6;
                    
                    let mut local_instructions = 0u64;
                    let start = Instant::now();
                    
                    while *running.lock().unwrap() {
                        match workload_type {
                            0 => {
                                // Prime number generation (CPU intensive)
                                for n in 2..10000 {
                                    if is_prime(n) {
                                        local_instructions += 1;
                                    }
                                }
                            },
                            1 => {
                                // Matrix multiplication (cache thrashing)
                                matrix_multiply();
                                local_instructions += 1000;
                            },
                            2 => {
                                // Floating point operations (FPU validation)
                                floating_point_ops();
                                local_instructions += 1000;
                            },
                            3 => {
                                // Integer arithmetic (ALU validation)
                                integer_arithmetic();
                                local_instructions += 1000;
                            },
                            4 => {
                                // Branch prediction stress
                                branch_prediction();
                                local_instructions += 1000;
                            },
                            _ => {
                                // Mixed workload
                                mixed_workload();
                                local_instructions += 1000;
                            }
                        }
                        
                        // Update metrics every second
                        if start.elapsed().as_secs() >= 1 {
                            let mut instr = instructions_per_sec.lock().unwrap();
                            *instr += local_instructions;
                            local_instructions = 0;
                            
                            // Check for thermal throttling (simplified)
                            let mut system = sysinfo::System::new();
                            system.refresh_cpu();
                            let current_freq = system.global_cpu_info().frequency() as f64;
                            let max_freq = system.global_cpu_info().frequency() as f64;
                            
                            if current_freq < max_freq * 0.9 {
                                let mut throttle = throttling_events.lock().unwrap();
                                *throttle += 1;
                            }
                            
                            // Update utilization
                            let mut util = utilization.lock().unwrap();
                            *util = system.global_cpu_info().cpu_usage();
                        }
                    }
                })
            })
            .collect();
        
        // Wait for all threads to complete
        for handle in handles {
            let _ = handle.join();
        }
        
        // Wait for timer thread
        let _ = timer_thread.join();
        
        // Calculate final metrics
        let final_utilization = *utilization.lock().unwrap();
        let final_throttling_events = *throttling_events.lock().unwrap();
        let final_instructions = *instructions_per_sec.lock().unwrap();
        
        // Calculate score (0-100)
        let mut score = 100;
        
        // Penalize for throttling
        if final_throttling_events > 0 {
            score -= (final_throttling_events as u8).min(20);
        }
        
        // Penalize for low utilization
        if final_utilization < 90.0 {
            score -= ((90.0 - final_utilization) / 2.0) as u8;
        }
        
        // Create issues if any
        let mut issues = Vec::new();
        
        if final_throttling_events > 5 {
            issues.push(TestIssue {
                component: "cpu".to_string(),
                severity: IssueSeverity::Medium,
                message: format!("CPU thermal throttling detected ({} events)", final_throttling_events),
                action: Some("Check cooling system and airflow".to_string()),
            });
        }
        
        if final_utilization < 80.0 {
            issues.push(TestIssue {
                component: "cpu".to_string(),
                severity: IssueSeverity::Low,
                message: format!("CPU utilization lower than expected ({}%)", final_utilization),
                action: Some("Check for CPU resource limits or contention".to_string()),
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
                "avg_cpu_utilization": final_utilization,
                "instructions_per_second": final_instructions,
                "thermal_throttling_events": final_throttling_events,
            }),
            issues,
        };
        
        Ok(result)
    }
    
    fn cleanup(&self) -> Result<()> {
        // No specific cleanup needed for CPU test
        Ok(())
    }
}

// Helper functions for CPU stress testing

fn is_prime(n: u32) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }
    
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    
    true
}

fn matrix_multiply() {
    // Simple matrix multiplication to stress cache
    const SIZE: usize = 100;
    let a = vec![vec![1.0; SIZE]; SIZE];
    let b = vec![vec![2.0; SIZE]; SIZE];
    let mut c = vec![vec![0.0; SIZE]; SIZE];
    
    for i in 0..SIZE {
        for j in 0..SIZE {
            for k in 0..SIZE {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
}

fn floating_point_ops() {
    // Perform complex floating point operations
    let mut x: f64 = 1.0;
    for _ in 0..100000 {
        x = x.sin().cos().tan().exp().ln().sqrt();
    }
}

fn integer_arithmetic() {
    // Integer arithmetic to stress ALU
    let mut x = 1u64;
    for _ in 0..100000 {
        x = x.wrapping_mul(7).wrapping_add(3).wrapping_div(2).wrapping_sub(1);
    }
}

fn branch_prediction() {
    // Branch prediction stress
    let mut _sum: i64 = 0; // Use i64 to prevent overflow
    let mut v = vec![0; 10000];
    
    // Fill with random data
    for i in 0..10000 {
        v[i] = (i * 17) % 2;
    }
    
    // Create unpredictable branches
    for i in 0..10000 {
        if v[i] == 1 {
            _sum += i as i64; // Cast to i64 to prevent overflow
        } else {
            _sum -= i as i64; // Cast to i64 to prevent overflow
        }
    }
}

fn mixed_workload() {
    // Mix of different operations
    is_prime(9973);
    floating_point_ops();
    integer_arithmetic();
}
