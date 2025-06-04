pub mod text;
pub mod json;
pub mod csv;

use crate::core::test::TestResult;
use crate::core::config::TestConfig;
use crate::core::runner::TestSuite;


pub trait Reporter {
    
    fn report_start(&self, config: &TestConfig);
    
    
    fn report_test_start(&self, test_name: &str);
    
    
    fn report_test_result(&self, result: &TestResult);
    
    
    fn report_suite_result(&self, suite: &TestSuite);
    
    
    fn report_warning(&self, message: &str);
    
    
    fn report_info(&self, message: &str);
}
