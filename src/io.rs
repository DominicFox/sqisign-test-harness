use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::Path;
use std::mem;

use crate::models::{BenchmarkRow, TelemetryRow, KeygenRow, KeyMetadataRow, SignatureTelemetryRow};

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
// WRITE METADATA (The "Heavy" Table)
// ============================================================================
pub fn write_metadata_csv(rows: Vec<KeyMetadataRow>, filepath: &str) {
    let file_exists = Path::new(filepath).exists();

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(filepath)
        .unwrap_or_else(|_| panic!("[-] Failed to open or create metadata file: {}", filepath));

    let mut wtr = csv::WriterBuilder::new()
        .has_headers(!file_exists) // Only write headers if the file is brand new
        .from_writer(file);

    for row in rows {
        if let Err(e) = wtr.serialize(row) {
            eprintln!("[-] Error serializing metadata row: {}", e);
        }
    }

    if let Err(e) = wtr.flush() {
        eprintln!("[-] Error flushing metadata CSV: {}", e);
    }
}

// ============================================================================
// WRITE TELEMETRY (The "Light" Table)
// ============================================================================
pub fn write_telemetry_csv(rows: Vec<SignatureTelemetryRow>, filepath: &str) {
    let file_exists = Path::new(filepath).exists();

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(filepath)
        .unwrap_or_else(|_| panic!("[-] Failed to open or create telemetry file: {}", filepath));

    let mut wtr = csv::WriterBuilder::new()
        .has_headers(!file_exists) // Only write headers if the file is brand new
        .from_writer(file);

    for row in rows {
        if let Err(e) = wtr.serialize(row) {
            eprintln!("[-] Error serializing telemetry row: {}", e);
        }
    }

    if let Err(e) = wtr.flush() {
        eprintln!("[-] Error flushing telemetry CSV: {}", e);
    }
}