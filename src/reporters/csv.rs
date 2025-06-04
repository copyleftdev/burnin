use std::fs::File;
use std::io::{self, Write};
use csv::Writer;

use crate::core::test::{TestResult, TestStatus, IssueSeverity};
use crate::core::config::TestConfig;
use crate::core::runner::TestSuite;
use crate::reporters::Reporter;

/// CSV reporter for spreadsheet-compatible output
pub struct CsvReporter {
    output_file: Option<String>,
}

impl CsvReporter {
    /// Create a new CSV reporter
    pub fn new(output_file: Option<String>) -> Self {
        Self { output_file }
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
    
    /// Convert severity to string
    fn severity_to_string(severity: IssueSeverity) -> &'static str {
        match severity {
            IssueSeverity::Critical => "CRITICAL",
            IssueSeverity::High => "HIGH",
            IssueSeverity::Medium => "MEDIUM",
            IssueSeverity::Low => "LOW",
        }
    }
    
    /// Create a CSV writer
    fn create_writer(&self) -> io::Result<Writer<Box<dyn Write>>> {
        match &self.output_file {
            Some(path) => {
                let file = File::create(path)?;
                Ok(csv::Writer::from_writer(Box::new(file) as Box<dyn Write>))
            }
            None => {
                Ok(csv::Writer::from_writer(Box::new(io::stdout()) as Box<dyn Write>))
            }
        }
    }
}

impl Reporter for CsvReporter {
    fn report_start(&self, _config: &TestConfig) {
        // CSV reporter doesn't output anything at start
    }
    
    fn report_test_start(&self, _test_name: &str) {
        // CSV reporter doesn't output anything at test start
    }
    
    fn report_test_result(&self, _result: &TestResult) {
        // Individual test results are only reported in the final output
    }
    
    fn report_suite_result(&self, suite: &TestSuite) {
        // Create CSV writer
        let mut writer = match self.create_writer() {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Error creating CSV writer: {}", e);
                return;
            }
        };
        
        // Write test results section
        if let Err(e) = writer.write_record(&[
            "Test Name", "Status", "Score", "Duration (s)", "Issues"
        ]) {
            eprintln!("Error writing CSV header: {}", e);
            return;
        }
        
        // Write each test result
        for result in &suite.results {
            // Collect issues as a single string
            let issues_str = result.issues.iter()
                .map(|issue| format!("[{}] {}", Self::severity_to_string(issue.severity), issue.message))
                .collect::<Vec<_>>()
                .join("; ");
            
            if let Err(e) = writer.write_record(&[
                &result.name,
                Self::status_to_string(result.status),
                &result.score.to_string(),
                &result.duration.as_secs().to_string(),
                &issues_str,
            ]) {
                eprintln!("Error writing CSV record: {}", e);
                return;
            }
        }
        
        // Write a blank line
        if let Err(e) = writer.write_record(&[""; 5]) {
            eprintln!("Error writing CSV blank line: {}", e);
            return;
        }
        
        // Write summary section
        if let Err(e) = writer.write_record(&[
            "Summary", "", "", "", ""
        ]) {
            eprintln!("Error writing CSV summary header: {}", e);
            return;
        }
        
        // System information
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
        
        // Write summary data
        let summary_records = [
            ["System", &hostname, "", "", ""],
            ["Test Date", &suite.start_time.format("%Y-%m-%d").to_string(), "", "", ""],
            ["Test Time", &suite.start_time.format("%H:%M:%S").to_string(), "", "", ""],
            ["Overall Result", Self::status_to_string(suite.overall_status), "", "", ""],
            ["Overall Score", &suite.overall_score.to_string(), "", "", ""],
            ["Duration (s)", &suite.end_time.map_or(0, |end| {
                end.signed_duration_since(suite.start_time).num_seconds() as u64
            }).to_string(), "", "", ""],
        ];
        
        for record in &summary_records {
            if let Err(e) = writer.write_record(record) {
                eprintln!("Error writing CSV summary record: {}", e);
                return;
            }
        }
        
        // Write a blank line
        if let Err(e) = writer.write_record(&[""; 5]) {
            eprintln!("Error writing CSV blank line: {}", e);
            return;
        }
        
        // Write metrics section
        if let Err(e) = writer.write_record(&[
            "Metrics", "", "", "", ""
        ]) {
            eprintln!("Error writing CSV metrics header: {}", e);
            return;
        }
        
        if let Err(e) = writer.write_record(&[
            "Test Name", "Metric", "Value", "", ""
        ]) {
            eprintln!("Error writing CSV metrics column headers: {}", e);
            return;
        }
        
        // Write metrics for each test
        for result in &suite.results {
            if let serde_json::Value::Object(metrics) = &result.metrics {
                for (key, value) in metrics {
                    if let Err(e) = writer.write_record(&[
                        &result.name,
                        key,
                        &value.to_string(),
                        "",
                        "",
                    ]) {
                        eprintln!("Error writing CSV metrics record: {}", e);
                        return;
                    }
                }
            }
        }
        
        // Flush the writer to ensure all data is written
        if let Err(e) = writer.flush() {
            eprintln!("Error flushing CSV writer: {}", e);
        }
    }
    
    fn report_warning(&self, _message: &str) {
        // CSV reporter doesn't output warnings
    }
    
    fn report_info(&self, _message: &str) {
        // CSV reporter doesn't output info messages
    }
}
