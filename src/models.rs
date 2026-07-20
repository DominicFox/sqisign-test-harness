use serde::Serialize;

// Telemetry as gathered by the C binaries
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TelemetryRow {
    pub key_index: u64,            
    pub keygen_cycles: u64,
    pub sign_cycles: u64,
    pub verify_cycles: u64,
}

// 
#[derive(Serialize)]
pub struct BenchmarkRow {
    pub timestamp: String,
    pub variant: String,
    pub batch_id: usize,
    pub key_index: u64,
    pub message_hex: String,
    pub public_key_hex: String,
    pub secret_key_hex: String,
    pub thermal_state: u64,
    pub level: u32,
    pub keygen_cycles: f64,             
    pub sign_cycles_array: String,      // Needs renaming as now a single value
    pub verify_cycles_array: String,    // Needs renaming as now a single value
}

#[derive(Serialize)]
pub struct KeygenRow {
    pub batch_id: usize,
    pub key_index: u64,
    pub thermal_state: u64,
    pub level: u32,
    pub keygen_cycles: f64,
}

#[derive(Debug)]
pub struct SchemeConfig {
    pub name: &'static str,
    pub path: &'static str,
    pub language: &'static str, // "C" or "Julia"
    pub levels: Vec<u32>, // NIST levels to benchmark ([1, 3, 5])
    pub output_csv_fixed_key: &'static str, // Output CSV filename for fixed key tests
    pub output_csv_fixed_msg: &'static str, // Output CSV filename for fixed msg tests
    pub output_csv_keygen: &'static str, // Output CSV filename for keygen only
    pub is_enabled: bool,
}


// OUTDATED STRUCTS: EXPERIMENTED WITH SEPARATING CSVs TO REDUCE REDUNDUNT DATA STORAGE
#[derive(Serialize)]
pub struct KeyMetadataRow {
    pub timestamp: String,
    pub variant: String,
    pub level: u32,
    pub batch_id: usize,
    pub key_index: u64,           
    pub public_key_hex: String,
    pub secret_key_hex: String,
    pub keygen_cycles: f64,
}

#[derive(Serialize)]
pub struct SignatureTelemetryRow {
    pub batch_id: usize,
    pub key_index: u64,           
    pub iteration_index: u64,     
    pub message_hex: String,
    pub thermal_state: u64,
    pub sign_cycles: f64,
    pub verify_cycles: f64,
}