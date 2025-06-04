use std::process;
use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use anyhow::{Result, Context};
use log::{info, error};
use simple_logger::SimpleLogger;

mod core;
mod tests;
mod reporters;

use crate::core::config::TestConfig;
use crate::core::runner::BurnInRunner;
use crate::core::test::BurnInTest;
use crate::reporters::{Reporter, text::TextReporter, json::JsonReporter, csv::CsvReporter};


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
    
    
    #[arg(short, long)]
    output: Option<String>,
    
    
    #[arg(short, long)]
    verbose: bool,
    
    
    #[arg(short, long)]
    quiet: bool,
    
    
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    
    #[command(subcommand)]
    command: Commands,
}


#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    
    Text,
    
    Json,
    
    Csv,
}


#[derive(Subcommand)]
enum Commands {
    
    Quick {
        
        #[arg(short, long, value_enum)]
        components: Option<Vec<Component>>,
        
        
        #[arg(short, long)]
        threads: Option<usize>,
    },
    
    
    Standard {
        
        #[arg(short, long, value_enum)]
        components: Option<Vec<Component>>,
        
        
        #[arg(short, long)]
        threads: Option<usize>,
        
        
        #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=10))]
        stress: Option<u8>,
    },
    
    
    Full {
        
        #[arg(short, long, value_enum)]
        components: Option<Vec<Component>>,
        
        
        #[arg(short, long)]
        threads: Option<usize>,
        
        
        #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=10))]
        stress: Option<u8>,
    },
    
    
    Custom {
        
        #[arg(short, long)]
        duration: String,
        
        
        #[arg(short, long, value_enum)]
        components: Option<Vec<Component>>,
        
        
        #[arg(short, long)]
        threads: Option<usize>,
        
        
        #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=10))]
        stress: Option<u8>,
        
        
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=95))]
        memory_size: Option<u8>,
        
        
        #[arg(long)]
        storage_path: Option<PathBuf>,
        
        
        #[arg(long)]
        storage_size: Option<usize>,
    },
    
    
    Hardware,
}


#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Component {
    
    Cpu,
    
    Memory,
    
    Storage,
    
    Network,
    
    Thermal,
}

fn main() -> Result<()> {
    
    let cli = Cli::parse();
    
    
    let log_level = if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    
    SimpleLogger::new()
        .with_level(log_level)
        .init()
        .context("Failed to initialize logger")?;
    
    info!("Burnin v{}", env!("CARGO_PKG_VERSION"));
    
    
    let mut config = if let Some(_path) = &cli.config {
        
        
        TestConfig::default()
    } else {
        
        TestConfig::default()
    };
    
    
    match &cli.command {
        Commands::Quick { components, threads } => {
            config.apply_preset_quick();
            update_config_from_args(&mut config, components, *threads, None, None, None, None);
        }
        
        Commands::Standard { components, threads, stress } => {
            config.apply_preset_standard();
            update_config_from_args(&mut config, components, *threads, *stress, None, None, None);
        }
        
        Commands::Full { components, threads, stress } => {
            config.apply_preset_full();
            update_config_from_args(&mut config, components, *threads, *stress, None, None, None);
        }
        
        Commands::Custom { duration, components, threads, stress, memory_size, storage_path, storage_size } => {
            
            config.duration = humantime::parse_duration(duration)
                .context("Failed to parse duration")?;
            
            update_config_from_args(
                &mut config,
                components,
                *threads,
                *stress,
                *memory_size,
                storage_path.as_ref(),
                *storage_size,
            );
        }
        
        Commands::Hardware => {
            return print_hardware_info();
        }
    }
    
    
    let reporter: Box<dyn Reporter + Send + Sync> = match cli.format {
        OutputFormat::Text => Box::new(TextReporter::new(cli.verbose, cli.quiet)),
        OutputFormat::Json => Box::new(JsonReporter::new(cli.output.clone(), cli.verbose)),
        OutputFormat::Csv => Box::new(CsvReporter::new(cli.output.clone())),
    };
    
    
    let mut tests: Vec<Box<dyn core::test::BurnInTest + Send + Sync>> = Vec::new();
    
    if config.cpu_enabled {
        tests.push(Box::new(tests::cpu::CpuStressTest));
    }
    
    if config.memory_enabled {
        tests.push(Box::new(tests::memory::MemoryValidationTest));
    }
    
    if config.storage_enabled {
        tests.push(Box::new(tests::storage::StorageIoTest));
    }
    
    if config.network_enabled {
        tests.push(Box::new(tests::network::NetworkTest));
    }
    
    if config.thermal_enabled {
        tests.push(Box::new(tests::thermal::ThermalMonitorTest));
    }
    
    if tests.is_empty() {
        error!("No tests enabled. Please enable at least one component to test.");
        process::exit(1);
    }
    
    
    let mut runner = BurnInRunner::new(tests, config, reporter);
    
    
    match runner.execute_all() {
        Ok(suite) => {
            if suite.overall_status == core::test::TestStatus::Failed {
                process::exit(1);
            }
        }
        Err(e) => {
            error!("Test execution failed: {}", e);
            process::exit(2);
        }
    }
    
    Ok(())
}


fn update_config_from_args(
    config: &mut TestConfig,
    components: &Option<Vec<Component>>,
    threads: Option<usize>,
    stress: Option<u8>,
    memory_size: Option<u8>,
    storage_path: Option<&PathBuf>,
    storage_size: Option<usize>,
) {
    
    if let Some(components) = components {
        
        config.cpu_enabled = false;
        config.memory_enabled = false;
        config.storage_enabled = false;
        config.network_enabled = false;
        config.thermal_enabled = false;
        
        
        for component in components {
            match component {
                Component::Cpu => config.cpu_enabled = true,
                Component::Memory => config.memory_enabled = true,
                Component::Storage => config.storage_enabled = true,
                Component::Network => config.network_enabled = true,
                Component::Thermal => config.thermal_enabled = true,
            }
        }
    }
    
    
    if let Some(threads) = threads {
        config.threads = threads as u32;
    }
    
    
    if let Some(stress) = stress {
        config.stress_level = stress;
    }
    
    
    if let Some(memory_size) = memory_size {
        config.memory_test_size_percent = memory_size;
    }
    
    
    if let Some(path) = storage_path {
        if config.storage_test_paths.is_empty() {
            config.storage_test_paths.push(path.clone());
        } else {
            config.storage_test_paths[0] = path.clone();
        }
    }
    
    
    if let Some(size) = storage_size {
        config.storage_file_size = size as u64 * 1024 * 1024; 
    }
}


fn print_hardware_info() -> Result<()> {
    println!("System Hardware Information:");
    println!("============================");
    
    
    let cpu_test = tests::cpu::CpuStressTest;
    
    
    match cpu_test.detect_hardware() {
        Ok(hardware) => {
            println!("CPU Information:");
            println!("  Model: {}", hardware.cpu_info.model_name);
            println!("  Cores: {} physical, {} logical", hardware.cpu_info.physical_cores, hardware.cpu_info.logical_cores);
            println!("  Frequency: {:.2} GHz", hardware.cpu_info.frequency_mhz as f64 / 1000.0);
            
            println!("\nMemory Information:");
            println!("  Total: {:.2} GB", hardware.memory_info.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("  Available: {:.2} GB", hardware.memory_info.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            
            println!("\nStorage Devices:");
            for (i, device) in hardware.storage_devices.iter().enumerate() {
                println!("  Device #{}:", i + 1);
                println!("    Name: {}", device.name);
                println!("    Type: {:?}", device.device_type);
                println!("    Size: {:.2} GB", device.size_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
                if let Some(mount) = &device.mount_point {
                    println!("    Mount: {}", mount);
                }
            }
            
            println!("\nVirtualization:");
            println!("  Type: {:?}", hardware.virtualization);
            
            println!("\nThermal Sensors:");
            if hardware.thermal_sensors.is_empty() {
                println!("  No thermal sensors detected");
            } else {
                for (i, sensor) in hardware.thermal_sensors.iter().enumerate() {
                    println!("  Sensor #{}:", i + 1);
                    println!("    Name: {}", sensor.name);
                    println!("    Temperature: {:.1}°C", sensor.current_temp_celsius);
                    if let Some(critical) = sensor.critical_temp_celsius {
                        println!("    Critical: {:.1}°C", critical);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to detect hardware: {}", e);
        }
    }
    
    Ok(())
}
