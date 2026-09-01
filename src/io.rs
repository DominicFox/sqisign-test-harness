use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::Path;
use std::mem;

use crate::models::{BenchmarkRow, KeygenRow, StackRow, StackTelemetryRow};

pub fn write_keygen_csv(rows: Vec<KeygenRow>, filename: &str) {
    let file_exists = Path::new(filename).exists();
    let file = OpenOptions::new().create(true).write(true).append(true).open(filename).unwrap();
    let mut writer = csv::WriterBuilder::new().has_headers(!file_exists).from_writer(file);

    for row in rows {
        writer.serialize(row).unwrap();
    }
    writer.flush().unwrap();
}

pub fn write_full_csv(rows: Vec<BenchmarkRow>, filename: &str) {
    let file_exists = Path::new(filename).exists();
    
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filename)
        .expect("[-] Failed to open or establish targets on benchmark database CSV file");

    let mut writer = csv::WriterBuilder::new()
        .has_headers(!file_exists)
        .from_writer(file);

    for row in rows {
        // Ensure we don't write entirely empty arrays
        if !row.sign_cycles_array.is_empty() {
            writer.serialize(row).expect("[-] Failed to write dataset row to disk");
        }
    }
    writer.flush().expect("[-] Database pipe flushing routine failed");
}

pub fn read_telemetry_file(filename: &str) -> Vec<TelemetryRow> {
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("[-] Error: Could not find telemetry file '{}'", filename);
            return Vec::new();
        }
    };

    let mut raw_bytes = Vec::new();
    file.read_to_end(&mut raw_bytes).expect("[-] Failed to read telemetry bytes");

    let row_size = mem::size_of::<TelemetryRow>();
    let total_rows = raw_bytes.len() / row_size;
    let mut parsed_results = Vec::with_capacity(total_rows);
    
    unsafe {
        let struct_ptr = raw_bytes.as_ptr() as *const TelemetryRow;
        for i in 0..total_rows {
            parsed_results.push(*struct_ptr.add(i));
        }
    }
    
    let _ = std::fs::remove_file(filename);
    parsed_results
}

// ============================================================================
// STACK HIGH-WATER TELEMETRY
// ============================================================================
pub fn read_stack_telemetry_file(filename: &str) -> Vec<StackTelemetryRow> {
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("[-] Error: Could not find stack telemetry file '{}'", filename);
            return Vec::new();
        }
    };

    let mut raw_bytes = Vec::new();
    file.read_to_end(&mut raw_bytes).expect("[-] Failed to read stack telemetry bytes");

    let row_size = mem::size_of::<StackTelemetryRow>();
    let total_rows = raw_bytes.len() / row_size;
    let mut parsed_results = Vec::with_capacity(total_rows);

    unsafe {
        let struct_ptr = raw_bytes.as_ptr() as *const StackTelemetryRow;
        for i in 0..total_rows {
            parsed_results.push(*struct_ptr.add(i));
        }
    }

    let _ = std::fs::remove_file(filename);
    parsed_results
}

pub fn write_stack_csv(rows: Vec<StackRow>, filename: &str) {
    let file_exists = Path::new(filename).exists();

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filename)
        .expect("[-] Failed to open or establish targets on stack benchmark CSV file");

    let mut writer = csv::WriterBuilder::new()
        .has_headers(!file_exists)
        .from_writer(file);

    for row in rows {
        writer.serialize(row).expect("[-] Failed to write stack row to disk");
    }
    writer.flush().expect("[-] Stack CSV flushing routine failed");
}


