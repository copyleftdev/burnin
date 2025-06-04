use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};
use serde_json::json;
use sysinfo::{System, Components};

use crate::core::test::{BurnInTest, TestResult, TestStatus, TestIssue, IssueSeverity};
use crate::core::config::TestConfig;
use crate::core::hardware::{HardwareInfo, ThermalSensor};
use crate::core::error::Result;


pub struct ThermalMonitorTest;

impl BurnInTest for ThermalMonitorTest {
    fn name(&self) -> &'static str {
        "thermal_monitor"
    }
    
    fn detect_hardware(&self) -> Result<HardwareInfo> {
        
        let mut hardware_info = crate::tests::cpu::CpuStressTest.detect_hardware()?;
        
        
        let _system = System::new_all();
        
        let mut thermal_sensors = Vec::new();
        
        
        let components = Components::new_with_refreshed_list();
        for component in &components {
            if component.label().contains("temp") || component.label().contains("cpu") {
                thermal_sensors.push(ThermalSensor {
                    name: component.label().to_string(),
                    location: "Unknown".to_string(), 
                    current_temp_celsius: component.temperature(),
                    critical_temp_celsius: component.critical(),
                });
            }
        }
        
        hardware_info.thermal_sensors = thermal_sensors;
        
        Ok(hardware_info)
    }
    
    fn estimate_duration(&self, config: &TestConfig) -> Duration {
        config.duration
    }
    
    fn execute(&self, config: &TestConfig) -> Result<TestResult> {
        let start_time = Instant::now();
        
        
        if !config.thermal_monitoring {
            return Ok(TestResult {
                name: self.name().to_string(),
                status: TestStatus::Skipped,
                score: 100,
                duration: Duration::from_secs(0),
                metrics: json!({}),
                issues: Vec::new(),
            });
        }
        
        
        let hardware_info = self.detect_hardware()?;
        let sensors = &hardware_info.thermal_sensors;
        
        if sensors.is_empty() {
            return Ok(TestResult {
                name: self.name().to_string(),
                status: TestStatus::Skipped,
                score: 100,
                duration: start_time.elapsed(),
                metrics: json!({
                    "sensors_detected": 0,
                }),
                issues: vec![TestIssue {
                    component: "thermal".to_string(),
                    severity: IssueSeverity::Low,
                    message: "No thermal sensors detected".to_string(),
                    action: Some("Check if your system supports thermal monitoring".to_string()),
                }],
            });
        }
        
        println!("Starting thermal monitoring with {} sensors", sensors.len());
        
        
        let max_temp = Arc::new(Mutex::new(0.0f32));
        let min_temp = Arc::new(Mutex::new(100.0f32));
        let avg_temp = Arc::new(Mutex::new(0.0f32));
        let temp_readings = Arc::new(Mutex::new(0usize));
        let _throttling_events = Arc::new(Mutex::new(0usize));
        let warning_events = Arc::new(Mutex::new(0usize));
        let critical_events = Arc::new(Mutex::new(0usize));
        
        
        let running = Arc::new(Mutex::new(true));
        let running_clone = running.clone();
        
        
        let test_duration = config.duration; 
        let timer_thread = thread::spawn(move || {
            thread::sleep(test_duration);
            let mut running = running_clone.lock().unwrap();
            *running = false;
        });
        
        
        let monitor_thread = {
            let max_temp = max_temp.clone();
            let min_temp = min_temp.clone();
            let avg_temp = avg_temp.clone();
            let temp_readings = temp_readings.clone();
            let warning_events = warning_events.clone();
            let critical_events = critical_events.clone();
            let running = running.clone();
            
            
            let thermal_warning_threshold = config.thermal_warning_threshold;
            let thermal_critical_threshold = config.thermal_critical_threshold;
            let thermal_monitor_interval = config.thermal_monitor_interval;
            
            thread::spawn(move || {
                let mut _system = sysinfo::System::new();
                let mut total_temp = 0.0f32;
                let mut readings = 0usize;
                
                while *running.lock().unwrap() {
                    
                    _system.refresh_all();
                    
                    
                    
                    let components = Components::new_with_refreshed_list();
                    for component in &components {
                        let temp = component.temperature();
                        
                        
                        {
                            let mut max = max_temp.lock().unwrap();
                            if temp > *max {
                                *max = temp;
                            }
                        }
                        
                        {
                            let mut min = min_temp.lock().unwrap();
                            if temp < *min {
                                *min = temp;
                            }
                        }
                        
                        total_temp += temp;
                        readings += 1;
                        
                        
                        if temp >= thermal_warning_threshold {
                            let mut warnings = warning_events.lock().unwrap();
                            *warnings += 1;
                            
                            if temp >= thermal_critical_threshold {
                                let mut criticals = critical_events.lock().unwrap();
                                *criticals += 1;
                            }
                        }
                    }
                    
                    
                    if readings > 0 {
                        let mut avg = avg_temp.lock().unwrap();
                        *avg = total_temp / readings as f32;
                        
                        let mut count = temp_readings.lock().unwrap();
                        *count = readings;
                    }
                    
                    
                    thread::sleep(thermal_monitor_interval);
                }
            })
        };
        
        
        let _ = timer_thread.join();
        
        
        {
            let mut running_flag = running.lock().unwrap();
            *running_flag = false;
        }
        let _ = monitor_thread.join();
        
        
        let final_max_temp = *max_temp.lock().unwrap();
        let final_min_temp = *min_temp.lock().unwrap();
        let final_avg_temp = *avg_temp.lock().unwrap();
        let final_readings = *temp_readings.lock().unwrap();
        let final_warnings = *warning_events.lock().unwrap();
        let final_criticals = *critical_events.lock().unwrap();
        
        
        let mut score = 100;
        
        
        if final_max_temp > config.thermal_warning_threshold {
            let over_warning = final_max_temp - config.thermal_warning_threshold;
            let warning_range = config.thermal_critical_threshold - config.thermal_warning_threshold;
            let penalty = ((over_warning / warning_range) * 30.0) as u8;
            score -= penalty;
        }
        
        
        score -= (final_criticals as u8 * 10).min(50);
        
        
        let mut issues = Vec::new();
        
        if final_criticals > 0 {
            issues.push(TestIssue {
                component: "thermal".to_string(),
                severity: IssueSeverity::Critical,
                message: format!("Critical temperature threshold exceeded {} times", final_criticals),
                action: Some("Check cooling system immediately".to_string()),
            });
        } else if final_warnings > 0 {
            issues.push(TestIssue {
                component: "thermal".to_string(),
                severity: IssueSeverity::High,
                message: format!("Warning temperature threshold exceeded {} times", final_warnings),
                action: Some("Improve cooling or reduce system load".to_string()),
            });
        }
        
        if final_max_temp > 85.0 {
            issues.push(TestIssue {
                component: "thermal".to_string(),
                severity: IssueSeverity::Medium,
                message: format!("Maximum temperature is very high: {:.1}°C", final_max_temp),
                action: Some("Check cooling system efficiency".to_string()),
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
                "max_temperature_celsius": final_max_temp,
                "min_temperature_celsius": final_min_temp,
                "avg_temperature_celsius": final_avg_temp,
                "temperature_readings": final_readings,
                "warning_events": final_warnings,
                "critical_events": final_criticals,
                "sensors_detected": sensors.len(),
            }),
            issues,
        };
        
        Ok(result)
    }
    
    fn cleanup(&self) -> Result<()> {
        
        Ok(())
    }
}
