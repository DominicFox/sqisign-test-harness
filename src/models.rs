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

// All data saved for a single benchmark iteration
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

// One row per keygen iteration 
#[derive(Serialize)]
pub struct KeygenRow {
    pub batch_id: usize,
    pub key_index: u64,
    pub thermal_state: u64,
    pub level: u32,
    pub keygen_cycles: f64,
}

// Stack telemetry as written by the C stack_probe binaries.
// Must stay byte-identical to sp_row_t in stack_probe_common.h.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StackTelemetryRow {
    pub iter: u64,
    pub keygen_stack: u64,
    pub sign_stack: u64,
    pub verify_stack: u64,
    pub keygen_resident: u64,
    pub sign_resident: u64,
    pub verify_resident: u64,
    pub ok: u64,
}

// One row per primitive per iteration
// stack_bytes  = stack pointer excursion, i.e. what a thread must reserve.
// resident_bytes = pages actually faulted in, i.e. the RSS cost.
// These differ whenever a large frame is only sparsely written.
#[derive(Serialize)]
pub struct StackRow {
    pub timestamp: String,
    pub variant: String,
    pub batch_id: usize,
    pub level: u32,
    pub iteration: u64,
    pub primitive: String,
    pub stack_bytes: u64,
    pub resident_bytes: u64,
    pub ok: u64,
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
    // Stack high-water probe; empty string disables it (no probe for Julia).
    // The level suffix is appended exactly as for `path`.
    pub stack_path: &'static str,
    pub output_csv_stack: &'static str,
    pub is_enabled: bool,
}


