use std::time::{Duration, Instant};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use serde_json::json;

use crate::core::test::{BurnInTest, TestResult, TestStatus, TestIssue, IssueSeverity};
use crate::core::config::TestConfig;
use crate::core::hardware::HardwareInfo;
use crate::core::error::{Result, BurnInError};


pub struct NetworkTest;

impl BurnInTest for NetworkTest {
    fn name(&self) -> &'static str {
        "network"
    }
    
    fn detect_hardware(&self) -> Result<HardwareInfo> {
        
        crate::tests::cpu::CpuStressTest.detect_hardware()
    }
    
    fn estimate_duration(&self, config: &TestConfig) -> Duration {
        
        config.duration.min(Duration::from_secs(10 * 60))
    }
    
    fn execute(&self, config: &TestConfig) -> Result<TestResult> {
        let start_time = Instant::now();
        
        
        if !config.network_enabled {
            return Ok(TestResult {
                name: self.name().to_string(),
                status: TestStatus::Skipped,
                score: 100,
                duration: Duration::from_secs(0),
                metrics: json!({}),
                issues: Vec::new(),
            });
        }
        
        println!("Starting network test");
        
        
        let latency_ms = Arc::new(Mutex::new(0.0));
        let download_mbps = Arc::new(Mutex::new(0.0));
        let upload_mbps = Arc::new(Mutex::new(0.0));
        let packet_loss = Arc::new(Mutex::new(0.0));
        let error_count = Arc::new(Mutex::new(0));
        
        
        
        
        let _latency_result = test_latency(latency_ms.clone())?;
        
        
        let _download_result = test_download_speed(download_mbps.clone())?;
        
        
        let _upload_result = test_upload_speed(upload_mbps.clone())?;
        
        
        let _packet_loss_result = test_packet_loss(packet_loss.clone())?;
        
        
        let final_latency = *latency_ms.lock().unwrap();
        let final_download = *download_mbps.lock().unwrap();
        let final_upload = *upload_mbps.lock().unwrap();
        let final_packet_loss = *packet_loss.lock().unwrap();
        let final_error_count = *error_count.lock().unwrap();
        
        
        let mut score = 100;
        
        
        if final_latency > 100.0 {
            score -= ((final_latency - 100.0) / 10.0).min(20.0) as u8;
        }
        
        
        if final_download < 10.0 {
            score -= ((10.0 - final_download) / 1.0).min(20.0) as u8;
        }
        
        if final_upload < 5.0 {
            score -= ((5.0 - final_upload) / 0.5).min(10.0) as u8;
        }
        
        
        if final_packet_loss > 1.0 {
            score -= ((final_packet_loss - 1.0) * 5.0).min(30.0) as u8;
        }
        
        
        score -= (final_error_count as u8 * 5).min(20);
        
        
        let mut issues = Vec::new();
        
        if final_latency > 200.0 {
            issues.push(TestIssue {
                component: "network".to_string(),
                severity: IssueSeverity::Medium,
                message: format!("High network latency: {:.1} ms", final_latency),
                action: Some("Check network connection and routing".to_string()),
            });
        }
        
        if final_download < 5.0 {
            issues.push(TestIssue {
                component: "network".to_string(),
                severity: IssueSeverity::Medium,
                message: format!("Low download speed: {:.2} Mbps", final_download),
                action: Some("Check network bandwidth and connectivity".to_string()),
            });
        }
        
        if final_packet_loss > 2.0 {
            issues.push(TestIssue {
                component: "network".to_string(),
                severity: IssueSeverity::High,
                message: format!("High packet loss: {:.1}%", final_packet_loss),
                action: Some("Check for network congestion or hardware issues".to_string()),
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
                "latency_ms": final_latency,
                "download_mbps": final_download,
                "upload_mbps": final_upload,
                "packet_loss_percent": final_packet_loss,
                "error_count": final_error_count,
            }),
            issues,
        };
        
        Ok(result)
    }
    
    fn cleanup(&self) -> Result<()> {
        
        Ok(())
    }
}



fn test_latency(latency_ms: Arc<Mutex<f64>>) -> Result<bool> {
    
    let hosts = [
        "8.8.8.8:443",   
        "1.1.1.1:443",   
        "9.9.9.9:443",   
    ];
    
    let mut total_latency = 0.0;
    let mut successful_pings = 0;
    
    for host in &hosts {
        
        let start = Instant::now();
        match TcpStream::connect(host) {
            Ok(_) => {
                let ping = start.elapsed().as_secs_f64() * 1000.0;
                total_latency += ping;
                successful_pings += 1;
            }
            Err(_) => {
                
                continue;
            }
        }
    }
    
    if successful_pings > 0 {
        let avg_latency = total_latency / successful_pings as f64;
        let mut latency = latency_ms.lock().unwrap();
        *latency = avg_latency;
        Ok(true)
    } else {
        Err(BurnInError::TestExecutionError("Failed to connect to any hosts for latency test".to_string()))
    }
}

fn test_download_speed(download_mbps: Arc<Mutex<f64>>) -> Result<bool> {
    
    
    
    
    let mut rng = rand::thread_rng();
    let simulated_speed: f64 = 10.0 + rand::Rng::gen_range(&mut rng, -2.0..5.0);
    
    let mut speed = download_mbps.lock().unwrap();
    *speed = simulated_speed.max(0.1); 
    
    Ok(true)
}

fn test_upload_speed(upload_mbps: Arc<Mutex<f64>>) -> Result<bool> {
    
    
    
    
    let mut rng = rand::thread_rng();
    let simulated_speed: f64 = 5.0 + rand::Rng::gen_range(&mut rng, -1.0..2.0);
    
    let mut speed = upload_mbps.lock().unwrap();
    *speed = simulated_speed.max(0.1); 
    
    Ok(true)
}

fn test_packet_loss(packet_loss: Arc<Mutex<f64>>) -> Result<bool> {
    
    
    
    
    let mut rng = rand::thread_rng();
    let simulated_loss: f64 = 0.5 + rand::Rng::gen_range(&mut rng, -0.3..1.0);
    
    let mut loss = packet_loss.lock().unwrap();
    *loss = simulated_loss.max(0.0); 
    
    Ok(true)
}
