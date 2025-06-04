use std::io::{self, Write};
use colored::*;
use chrono::Local;

use crate::core::test::{TestResult, TestStatus, IssueSeverity};
use crate::core::config::TestConfig;
use crate::core::runner::TestSuite;
use crate::reporters::Reporter;

/// Text reporter for console output
pub struct TextReporter {
    verbose: bool,
    quiet: bool,
}

impl TextReporter {
    /// Create a new text reporter
    pub fn new(verbose: bool, quiet: bool) -> Self {
        Self { verbose, quiet }
    }
    
    /// Format a duration in a human-readable format
    fn format_duration(&self, duration: std::time::Duration) -> String {
        let total_secs = duration.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
    
    /// Format a test status with color
    fn format_status(&self, status: TestStatus) -> ColoredString {
        match status {
            TestStatus::Completed => "✓ PASS".green().bold(),
            TestStatus::Failed => "✗ FAIL".red().bold(),
            TestStatus::Partial => "⚠ PARTIAL".yellow().bold(),
            TestStatus::Skipped => "⏸ SKIPPED".blue().bold(),
            TestStatus::Pending => "⋯ PENDING".normal(),
            TestStatus::Running => "⟳ RUNNING".cyan().bold(),
        }
    }
}

impl Reporter for TextReporter {
    fn report_start(&self, config: &TestConfig) {
        if self.quiet {
            return;
        }
        
        println!("{}", "BURN-IN TEST STARTING".bold());
        println!("====================");
        
        let now = Local::now();
        println!("Started: {}", now.format("%Y-%m-%d %H:%M:%S %Z"));
        
        if self.verbose {
            println!("\nTest Configuration:");
            println!("  Duration: {:?}", config.duration);
            println!("  Stress Level: {}/10", config.stress_level);
            println!("  Threads: {}", if config.threads == 0 { "auto".to_string() } else { config.threads.to_string() });
            println!("  Memory Test Size: {}%", config.memory_test_size_percent);
            println!("\nEnabled Components:");
            println!("  CPU: {}", if config.cpu_enabled { "yes".green() } else { "no".red() });
            println!("  Memory: {}", if config.memory_enabled { "yes".green() } else { "no".red() });
            println!("  Storage: {}", if config.storage_enabled { "yes".green() } else { "no".red() });
            println!("  Network: {}", if config.network_enabled { "yes".green() } else { "no".red() });
            println!("  Thermal: {}", if config.thermal_enabled { "yes".green() } else { "no".red() });
        }
        
        println!("\nRunning tests...\n");
        io::stdout().flush().unwrap();
    }
    
    fn report_test_start(&self, test_name: &str) {
        if self.quiet {
            return;
        }
        
        if self.verbose {
            println!("Starting test: {}", test_name.cyan());
            io::stdout().flush().unwrap();
        } else {
            print!("Testing {}... ", test_name.cyan());
            io::stdout().flush().unwrap();
        }
    }
    
    fn report_test_result(&self, result: &TestResult) {
        if self.quiet {
            return;
        }
        
        if self.verbose {
            println!("Test {} completed with status: {}", 
                result.name.cyan(),
                self.format_status(result.status));
            println!("  Score: {}/100", result.score);
            println!("  Duration: {}", self.format_duration(result.duration));
            
            // Print metrics
            println!("  Metrics:");
            if let serde_json::Value::Object(metrics) = &result.metrics {
                for (key, value) in metrics {
                    println!("    {}: {}", key, value);
                }
            }
            
            // Print issues
            if !result.issues.is_empty() {
                println!("  Issues:");
                for issue in &result.issues {
                    let severity = match issue.severity {
                        IssueSeverity::Critical => "CRITICAL".red().bold(),
                        IssueSeverity::High => "HIGH".red(),
                        IssueSeverity::Medium => "MEDIUM".yellow(),
                        IssueSeverity::Low => "LOW".blue(),
                    };
                    
                    println!("    [{}] {}", severity, issue.message);
                    if let Some(action) = &issue.action {
                        println!("      Action: {}", action);
                    }
                }
            }
            
            println!();
        } else {
            println!("{} (Score: {}/100)", 
                self.format_status(result.status),
                result.score);
        }
        
        io::stdout().flush().unwrap();
    }
    
    fn report_suite_result(&self, suite: &TestSuite) {
        if self.quiet {
            // In quiet mode, just print the overall result and score
            println!("{} {}/100", 
                self.format_status(suite.overall_status),
                suite.overall_score);
            return;
        }
        
        println!("\n{}", "BURN-IN TEST RESULTS".bold());
        println!("====================");
        println!("System: {}", suite.system_info.as_ref().map(|s| s.hostname.as_str()).unwrap_or("Unknown"));
        println!("Started: {}", suite.start_time.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("Duration: {:?}", suite.duration);
        println!();
        
        // Print individual test results
        let max_name_len = suite.results.iter()
            .map(|r| r.name.len())
            .max()
            .unwrap_or(10);
        
        for result in &suite.results {
            println!("{}: {}{}(Score: {}/100)",
                result.name.cyan().bold(),
                " ".repeat(max_name_len - result.name.len() + 2),
                self.format_status(result.status),
                result.score);
        }
        
        println!("\n{}: {} (Score: {}/100)",
            "OVERALL RESULT".bold(),
            self.format_status(suite.overall_status),
            suite.overall_score);
        
        // Print recommendations based on issues
        let all_issues: Vec<_> = suite.results.iter()
            .flat_map(|r| r.issues.iter())
            .collect();
        
        if !all_issues.is_empty() {
            println!("\nRecommendations:");
            
            // Sort issues by severity
            let mut critical_issues: Vec<_> = all_issues.iter()
                .filter(|i| i.severity == IssueSeverity::Critical)
                .collect();
            
            let mut high_issues: Vec<_> = all_issues.iter()
                .filter(|i| i.severity == IssueSeverity::High)
                .collect();
            
            let mut other_issues: Vec<_> = all_issues.iter()
                .filter(|i| i.severity != IssueSeverity::Critical && i.severity != IssueSeverity::High)
                .collect();
            
            // Limit to top issues if there are many
            if critical_issues.len() + high_issues.len() + other_issues.len() > 5 {
                critical_issues.truncate(2);
                high_issues.truncate(2);
                other_issues.truncate(1);
            }
            
            // Print critical issues first
            for issue in critical_issues {
                println!("- {} - {}", "CRITICAL".red().bold(), issue.message);
                if let Some(action) = &issue.action {
                    println!("  → {}", action);
                }
            }
            
            // Print high severity issues
            for issue in high_issues {
                println!("- {} - {}", "HIGH".red(), issue.message);
                if let Some(action) = &issue.action {
                    println!("  → {}", action);
                }
            }
            
            // Print other issues
            for issue in other_issues {
                println!("- {}", issue.message);
                if let Some(action) = &issue.action {
                    println!("  → {}", action);
                }
            }
        }
    }
    
    fn report_warning(&self, message: &str) {
        if self.quiet {
            return;
        }
        
        eprintln!("{}: {}", "WARNING".yellow().bold(), message);
    }
    
    fn report_info(&self, message: &str) {
        if self.quiet {
            return;
        }
        
        if self.verbose {
            println!("{}: {}", "INFO".blue().bold(), message);
        }
    }
}
