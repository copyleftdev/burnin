// Core modules
pub mod core {
    pub mod config;
    pub mod error;
    pub mod hardware;
    pub mod runner;
    pub mod test;
}

// Test modules
pub mod tests {
    pub mod cpu;
    pub mod memory;
    pub mod network;
    pub mod storage;
    pub mod thermal;
}

// Reporter modules
pub mod reporters;
