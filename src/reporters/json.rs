use std::io::{self, Write};
use std::fs::File;
use serde_json::{json, Value};
use sysinfo::System;

use crate::core::test::{TestResult, TestStatus};
use crate::core::config::TestConfig;
use crate::core::runner::TestSuite;
use crate::reporters::Reporter;

/// JSON reporter for machine-readable output
pub struct JsonReporter {
    output_file: Option<String>,
    verbose: bool,
}

impl JsonReporter {
    /// Create a new JSON reporter
    pub fn new(output_file: Option<String>, verbose: bool) -> Self {
        Self { output_file, verbose }
    }
    
    /// Convert test status to string
    fn status_to_string(status: TestStatus) -> &'static str {
        match status {
            TestStatus::Completed => "PASS",
            TestStatus::Failed => "FAIL",
            TestStatus::Partial => "PARTIAL",
            TestStatus::Skipped => "SKIPPED",
            TestStatus::Pending => "PENDING",
            TestStatus::Running => "RUNNING",
        }
    }
    
    /// Write JSON to file or stdout
    fn write_json(&self, json_value: Value) -> io::Result<()> {
        let json_string = serde_json::to_string_pretty(&json_value)?;
        
        match &self.output_file {
            Some(path) => {
                let mut file = File::create(path)?;
                file.write_all(json_string.as_bytes())?;
            }
            None => {
                println!("{}", json_string);
            }
        }
        
        Ok(())
    }
}

impl Reporter for JsonReporter {
    fn report_start(&self, config: &TestConfig) {
        if self.verbose {
            let start_info = json!({
                "event": "test_start",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "config": {
                    "duration_seconds": config.duration.as_secs(),
                    "stress_level": config.stress_level,
                    "threads": config.threads,
                    "memory_test_size_percent": config.memory_test_size_percent,
                    "components": {
                        "cpu": config.cpu_enabled,
                        "memory": config.memory_enabled,
                        "storage": config.storage_enabled,
                        "network": config.network_enabled,
                        "thermal": config.thermal_enabled,
                    }
                }
            });
            
            if self.output_file.is_none() {
                // Only print to stdout if not writing to file
                let _ = self.write_json(start_info);
            }
        }
    }
    
    fn report_test_start(&self, test_name: &str) {
        if self.verbose {
            let test_start = json!({
                "event": "test_module_start",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "test_name": test_name,
            });
            
            if self.output_file.is_none() {
                // Only print to stdout if not writing to file
                let _ = self.write_json(test_start);
            }
        }
    }
    
    fn report_test_result(&self, result: &TestResult) {
        if self.verbose {
            let test_result = json!({
                "event": "test_module_result",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "test_name": result.name,
                "status": Self::status_to_string(result.status),
                "score": result.score,
                "duration_seconds": result.duration.as_secs(),
                "metrics": result.metrics,
                "issues": result.issues,
            });
            
            if self.output_file.is_none() {
                // Only print to stdout if not writing to file
                let _ = self.write_json(test_result);
            }
        }
    }
    
    fn report_suite_result(&self, suite: &TestSuite) {
        // Convert test results to JSON
        let test_results: Vec<Value> = suite.results.iter()
            .map(|result| {
                json!({
                    "name": result.name,
                    "result": Self::status_to_string(result.status),
                    "score": result.score,
                    "duration_seconds": result.duration.as_secs(),
                    "metrics": result.metrics,
                    "issues": result.issues.iter().map(|issue| {
                        json!({
                            "component": issue.component,
                            "severity": format!("{:?}", issue.severity).to_uppercase(),
                            "message": issue.message,
                            "action": issue.action,
                        })
                    }).collect::<Vec<Value>>(),
                })
            })
            .collect();
        
        // Get system information
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
        
        // Get system info using sysinfo
        let mut system = System::new_all();
        system.refresh_all();
        
        // Build recommendations from issues
        let recommendations: Vec<Value> = suite.results.iter()
            .flat_map(|r| r.issues.iter())
            .map(|issue| {
                json!({
                    "component": issue.component,
                    "severity": format!("{:?}", issue.severity).to_lowercase(),
                    "message": issue.message,
                    "action": issue.action,
                })
            })
            .collect();
        
        // Build final JSON output
        let final_result = json!({
            "summary": {
                "result": Self::status_to_string(suite.overall_status),
                "overall_score": suite.overall_score,
                "duration_seconds": suite.end_time.map_or(0, |end| {
                    end.signed_duration_since(suite.start_time).num_seconds() as u64
                }),
                "timestamp": suite.start_time.to_rfc3339(),
                "system_info": {
                    "hostname": hostname,
                    "os": format!("{} {}", System::name().unwrap_or_else(|| "Unknown".to_string()), 
                                   System::os_version().unwrap_or_else(|| "Unknown".to_string())),
                    "cpu": system.global_cpu_info().brand().to_string(),
                    "memory_gb": system.total_memory() / 1024 / 1024,
                    "virtualization": "Unknown", // Would need platform-specific detection
                }
            },
            "tests": test_results,
            "recommendations": recommendations,
        });
        
        // Write to file or stdout
        if let Err(e) = self.write_json(final_result) {
            eprintln!("Error writing JSON output: {}", e);
        }
    }
    
    fn report_warning(&self, message: &str) {
        if self.verbose {
            let warning = json!({
                "event": "warning",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "message": message,
            });
            
            if self.output_file.is_none() {
                // Only print to stdout if not writing to file
                let _ = self.write_json(warning);
            }
        }
    }
    
    fn report_info(&self, message: &str) {
        if self.verbose {
            let info = json!({
                "event": "info",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "message": message,
            });
            
            if self.output_file.is_none() {
                // Only print to stdout if not writing to file
                let _ = self.write_json(info);
            }
        }
    }
}
