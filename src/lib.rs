
pub mod core {
    pub mod config;
    pub mod error;
    pub mod hardware;
    pub mod runner;
    pub mod test;
}


pub mod tests {
    pub mod cpu;
    pub mod memory;
    pub mod network;
    pub mod storage;
    pub mod thermal;
}


pub mod reporters;
