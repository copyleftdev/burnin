pub mod text;
pub mod json;
pub mod csv;

use crate::core::test::TestResult;
use crate::core::config::TestConfig;
use crate::core::runner::TestSuite;

/// Reporter trait for outputting test results
pub trait Reporter {
    /// Report the start of testing
    fn report_start(&self, config: &TestConfig);
    
    /// Report the start of a specific test
    fn report_test_start(&self, test_name: &str);
    
    /// Report the result of a specific test
    fn report_test_result(&self, result: &TestResult);
    
    /// Report the final results of the test suite
    fn report_suite_result(&self, suite: &TestSuite);
    
    /// Report a warning message
    fn report_warning(&self, message: &str);
    
    /// Report an informational message
    fn report_info(&self, message: &str);
}
