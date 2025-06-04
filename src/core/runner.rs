use std::sync::{Arc, Mutex};
use std::time::Instant;
use rayon::prelude::*;
use crate::core::error::{Result, BurnInError};
use crate::core::test::{BurnInTest, TestResult, TestStatus};
use crate::core::hardware::SystemInfo;
use crate::core::config::TestConfig;
use crate::reporters::Reporter;

/// Collection of test results
#[derive(Debug)]
pub struct TestSuite {
    pub results: Vec<TestResult>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub overall_score: u8,
    pub overall_status: TestStatus,
    pub system_info: Option<SystemInfo>,
    pub duration: std::time::Duration,
}

impl TestSuite {
    /// Create a new test suite
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            start_time: chrono::Utc::now(),
            end_time: None,
            overall_score: 0,
            overall_status: TestStatus::Pending,
            system_info: None,
            duration: std::time::Duration::from_secs(0),
        }
    }
    
    /// Calculate the overall score and status
    pub fn finalize(&mut self) {
        // Set end time and calculate duration
        let end = chrono::Utc::now();
        self.end_time = Some(end);
        self.duration = (end - self.start_time).to_std().unwrap_or_default();
        
        if self.results.is_empty() {
            self.overall_score = 0;
            self.overall_status = TestStatus::Failed;
            return;
        }
        
        // Calculate weighted average score
        let total_duration_secs: u64 = self.results.iter()
            .map(|r| r.duration.as_secs())
            .sum();
        
        if total_duration_secs == 0 {
            // Equal weighting if all durations are 0
            self.overall_score = (self.results.iter()
                .map(|r| r.score as u32)
                .sum::<u32>() / self.results.len() as u32) as u8;
        } else {
            // Weight by duration
            self.overall_score = (self.results.iter()
                .map(|r| (r.score as u64 * r.duration.as_secs()) as u32)
                .sum::<u32>() / total_duration_secs as u32) as u8;
        }
        
        // Determine overall status
        if self.results.iter().any(|r| r.status == TestStatus::Failed) {
            self.overall_status = TestStatus::Failed;
        } else if self.results.iter().any(|r| r.status == TestStatus::Partial) {
            self.overall_status = TestStatus::Partial;
        } else {
            self.overall_status = TestStatus::Completed;
        }
    }
}

/// Test execution engine
pub struct BurnInRunner {
    tests: Vec<Box<dyn BurnInTest + Send + Sync>>,
    config: TestConfig,
    reporter: Box<dyn Reporter + Send + Sync>,
    interrupted: Arc<Mutex<bool>>,
}

impl BurnInRunner {
    /// Create a new test runner
    pub fn new(
        tests: Vec<Box<dyn BurnInTest + Send + Sync>>,
        config: TestConfig,
        reporter: Box<dyn Reporter + Send + Sync>,
    ) -> Self {
        Self {
            tests,
            config,
            reporter,
            interrupted: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Set up interrupt handler
    pub fn setup_interrupt_handler(&self) -> Result<()> {
        let interrupted = self.interrupted.clone();
        
        ctrlc::set_handler(move || {
            let mut flag = interrupted.lock().unwrap();
            *flag = true;
            println!("\nReceived interrupt signal...");
            println!("Stopping current tests gracefully...");
            println!("This may take a moment to clean up resources safely.");
        })
        .map_err(|e| BurnInError::UnexpectedError(format!("Failed to set Ctrl-C handler: {}", e)))?;
        
        Ok(())
    }
    
    /// Check if execution was interrupted
    fn is_interrupted(&self) -> bool {
        *self.interrupted.lock().unwrap()
    }
    
    /// Execute tests sequentially
    pub fn execute_sequential(&mut self) -> Result<TestSuite> {
        let mut suite = TestSuite::new();
        
        self.reporter.report_start(&self.config);
        
        for test in &self.tests {
            if self.is_interrupted() {
                break;
            }
            
            let name = test.name();
            self.reporter.report_test_start(name);
            
            let start_time = Instant::now();
            let result = match test.execute(&self.config) {
                Ok(result) => result,
                Err(e) => {
                    let mut result = TestResult {
                        name: name.to_string(),
                        status: TestStatus::Failed,
                        score: 0,
                        duration: start_time.elapsed(),
                        metrics: serde_json::json!({}),
                        issues: Vec::new(),
                    };
                    
                    // Add the error as an issue
                    use crate::core::test::{TestIssue, IssueSeverity};
                    result.issues.push(TestIssue {
                        component: name.to_string(),
                        severity: IssueSeverity::Critical,
                        message: format!("Test failed: {}", e),
                        action: Some("Check system logs for details".to_string()),
                    });
                    
                    result
                }
            };
            
            self.reporter.report_test_result(&result);
            suite.results.push(result);
            
            // Clean up after the test
            if let Err(e) = test.cleanup() {
                self.reporter.report_warning(&format!("Failed to clean up after test {}: {}", name, e));
            }
        }
        
        suite.finalize();
        self.reporter.report_suite_result(&suite);
        
        Ok(suite)
    }
    
    /// Execute compatible tests in parallel
    pub fn execute_parallel(&mut self) -> Result<TestSuite> {
        let mut suite = TestSuite::new();
        
        self.reporter.report_start(&self.config);
        
        // Group tests by compatibility
        // For now, we'll just run CPU and memory tests together, and storage tests separately
        let mut cpu_memory_tests = Vec::new();
        let mut other_tests = Vec::new();
        
        for test in self.tests.drain(..) {
            let name = test.name();
            if name.contains("cpu") || name.contains("memory") {
                cpu_memory_tests.push(test);
            } else {
                other_tests.push(test);
            }
        }
        
        // Execute CPU and memory tests in parallel
        if !cpu_memory_tests.is_empty() {
            let config = self.config.clone();
            let interrupted = self.interrupted.clone();
            let reporter = &self.reporter;
            
            reporter.report_info("Running CPU and memory tests in parallel...");
            
            let results: Vec<TestResult> = cpu_memory_tests.par_iter()
                .map(|test| {
                    if *interrupted.lock().unwrap() {
                        return None;
                    }
                    
                    let name = test.name();
                    reporter.report_test_start(name);
                    
                    let start_time = Instant::now();
                    let result = match test.execute(&config) {
                        Ok(result) => result,
                        Err(e) => {
                            let mut result = TestResult {
                                name: name.to_string(),
                                status: TestStatus::Failed,
                                score: 0,
                                duration: start_time.elapsed(),
                                metrics: serde_json::json!({}),
                                issues: Vec::new(),
                            };
                            
                            // Add the error as an issue
                            use crate::core::test::{TestIssue, IssueSeverity};
                            result.issues.push(TestIssue {
                                component: name.to_string(),
                                severity: IssueSeverity::Critical,
                                message: format!("Test failed: {}", e),
                                action: Some("Check system logs for details".to_string()),
                            });
                            
                            result
                        }
                    };
                    
                    reporter.report_test_result(&result);
                    
                    // Clean up after the test
                    if let Err(e) = test.cleanup() {
                        reporter.report_warning(&format!("Failed to clean up after test {}: {}", name, e));
                    }
                    
                    Some(result)
                })
                .filter_map(|r| r)
                .collect();
            
            suite.results.extend(results);
        }
        
        // Execute other tests sequentially
        for test in other_tests {
            if self.is_interrupted() {
                break;
            }
            
            let name = test.name();
            self.reporter.report_test_start(name);
            
            let start_time = Instant::now();
            let result = match test.execute(&self.config) {
                Ok(result) => result,
                Err(e) => {
                    let mut result = TestResult {
                        name: name.to_string(),
                        status: TestStatus::Failed,
                        score: 0,
                        duration: start_time.elapsed(),
                        metrics: serde_json::json!({}),
                        issues: Vec::new(),
                    };
                    
                    // Add the error as an issue
                    use crate::core::test::{TestIssue, IssueSeverity};
                    result.issues.push(TestIssue {
                        component: name.to_string(),
                        severity: IssueSeverity::Critical,
                        message: format!("Test failed: {}", e),
                        action: Some("Check system logs for details".to_string()),
                    });
                    
                    result
                }
            };
            
            self.reporter.report_test_result(&result);
            suite.results.push(result);
            
            // Clean up after the test
            if let Err(e) = test.cleanup() {
                self.reporter.report_warning(&format!("Failed to clean up after test {}: {}", name, e));
            }
        }
        
        suite.finalize();
        self.reporter.report_suite_result(&suite);
        
        Ok(suite)
    }
    
    /// Execute tests with recovery capabilities
    pub fn execute_with_recovery(&mut self) -> Result<TestSuite> {
        // Set up interrupt handler
        self.setup_interrupt_handler()?;
        
        // Execute tests based on configuration
        if self.config.cpu_enabled && self.config.memory_enabled {
            // Run compatible tests in parallel
            self.execute_parallel()
        } else {
            // Run tests sequentially
            self.execute_sequential()
        }
    }
    
    /// Execute all tests based on configuration
    pub fn execute_all(&mut self) -> Result<TestSuite> {
        // Set up interrupt handler
        self.setup_interrupt_handler()?;
        
        // Report start of testing
        self.reporter.report_info("Starting burn-in tests");
        
        // Choose execution strategy based on configuration
        let result = if self.config.cpu_enabled && self.config.memory_enabled {
            // Run compatible tests in parallel
            self.execute_parallel()
        } else {
            // Run tests sequentially
            self.execute_sequential()
        };
        
        // Report completion
        match &result {
            Ok(suite) => {
                let status_str = format!("{:?}", suite.overall_status);
                self.reporter.report_info(&format!("All tests completed with status: {}", status_str));
            },
            Err(e) => {
                self.reporter.report_warning(&format!("Tests failed with error: {}", e));
            }
        }
        
        result
    }
}
