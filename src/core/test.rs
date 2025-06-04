use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::core::error::Result;
use crate::core::hardware::HardwareInfo;
use crate::core::config::TestConfig;

/// The status of a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Partial,
}

impl TestStatus {
    /// Returns `true` if the test has failed.
    pub fn is_failure(&self) -> bool {
        matches!(self, TestStatus::Failed)
    }
}

/// The result of a test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub score: u8,
    pub duration: Duration,
    pub metrics: serde_json::Value,
    pub issues: Vec<TestIssue>,
}

/// An issue detected during a test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestIssue {
    pub component: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub action: Option<String>,
}

/// The severity of an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// A trait for burn-in tests.
pub trait BurnInTest {
    /// Returns the name of the test.
    fn name(&self) -> &'static str;
    
    /// Detects the hardware required for the test.
    fn detect_hardware(&self) -> Result<HardwareInfo>;
    
    /// Estimates the duration of the test.
    fn estimate_duration(&self, config: &TestConfig) -> Duration;
    
    /// Executes the test.
    fn execute(&self, config: &TestConfig) -> Result<TestResult>;
    
    /// Cleans up after the test.
    fn cleanup(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_status_is_failure() {
        assert!(TestStatus::Failed.is_failure());
        assert!(!TestStatus::Completed.is_failure());
        assert!(!TestStatus::Skipped.is_failure());
        assert!(!TestStatus::Pending.is_failure());
    }

    #[test]
    fn test_issue_severity_ordering() {
        assert!(IssueSeverity::Critical > IssueSeverity::High);
        assert!(IssueSeverity::High > IssueSeverity::Medium);
        assert!(IssueSeverity::Medium > IssueSeverity::Low);
    }

    #[test]
    fn test_test_result_creation() {
        let result = TestResult {
            name: "test_cpu".to_string(),
            status: TestStatus::Completed,
            score: 85,
            duration: Duration::from_secs(60),
            metrics: serde_json::json!({}),
            issues: vec![],
        };
        
        assert_eq!(result.name, "test_cpu");
        assert_eq!(result.status, TestStatus::Completed);
        assert_eq!(result.score, 85);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_test_issue_creation() {
        let issue = TestIssue {
            component: "CPU".to_string(),
            severity: IssueSeverity::High,
            message: "High temperature detected".to_string(),
            action: Some("Check cooling system".to_string()),
        };
        
        assert_eq!(issue.component, "CPU");
        assert_eq!(issue.severity, IssueSeverity::High);
        assert!(issue.action.is_some());
    }
}
