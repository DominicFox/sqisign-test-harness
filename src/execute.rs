use std::process::Command;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Read;
use chrono::Utc;

use crate::models::{BenchmarkRow, SchemeConfig, TelemetryRow, KeygenRow, KeyMetadataRow, SignatureTelemetryRow};
use crate::io;

use std::ffi::CString;
use std::os::raw::{c_char, c_int};

unsafe extern "C" {
    fn notify_register_check(name: *const c_char, out_token: *mut c_int) -> u32;
    fn notify_get_state(token: c_int, state64: *mut u64) -> u32;
    fn notify_cancel(token: c_int) -> u32;
}

// Queries the macOS Darwin kernel for the exact Thermal Pressure State.
// Returns 0 (Nominal), 1 (Moderate), 2 (Heavy/Throttling), 3 (Trapping), 4 (Sleeping), or 99 (Error)
pub fn get_thermal_pressure_state() -> u64 {
    unsafe {
        let key = CString::new("com.apple.system.thermalpressurelevel")
            .expect("Failed to create CString");
            
        let mut token: c_int = 0;
        
        let reg_status = notify_register_check(key.as_ptr(), &mut token);
        if reg_status != 0 {
            eprintln!("[-] Warning: Failed to register Darwin thermal notification.");
            return 99; 
        }

        let mut state: u64 = 0;
        let state_status = notify_get_state(token, &mut state);
        notify_cancel(token);

        if state_status == 0 {
            state
        } else {
            eprintln!("[-] Warning: Failed to read Darwin thermal state.");
            99
        }
    }
}

pub fn apply_macos_thread_policies() {
    unsafe {
        libc::setpriority(libc::PRIO_PROCESS, 0, -20);
    }
}

// Centralised Command Builder to call the binaries
fn build_cmd(scheme: &SchemeConfig, mode: usize, keys: usize, iterations: Option<usize>, level: u32, warmup: usize) -> Command {
    let mut cmd = if scheme.language == "Julia" {
        let mut c = Command::new("julia");
        c.arg(scheme.path);
        c.arg(format!("--level={}", level)); 
        c
    } else {
        let resolved_path = format!("{}_lvl{}", scheme.path, level);
        Command::new(resolved_path)
    };

    cmd.arg(format!("--keys={}", keys))
       .arg(format!("--warmup={}", warmup))
       .arg(format!("--mode={}", mode));

    if let Some(iters) = iterations {
        cmd.arg(format!("--iterations={}", iters));
    }
    cmd
}

pub fn run_keygen_only_batch(
    scheme: &SchemeConfig, 
    batch_id: usize, 
    num_keys: usize,
    level: u32,
    warmup: usize
) -> Vec<KeygenRow> {
    let mut cmd = build_cmd(scheme, 0, num_keys, None, level, warmup);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[-] Failed to spawn Mode 0 for {}: {}", scheme.name, e);
            return Vec::new();
        }
    };

    if !child.wait().map(|status| status.success()).unwrap_or(false) {
        eprintln!("[-] {} binary crashed during Keygen profiling!", scheme.language);
        return Vec::new();
    }

    let thermal_state = get_thermal_pressure_state();
    let raw_data = io::read_telemetry_file("sqisign_telemetry.bin");

    raw_data.into_iter().map(|row| KeygenRow {
        batch_id,
        key_index: row.key_index,
        thermal_state,
        level,
        keygen_cycles: row.keygen_cycles as f64 / 1_000_000.0,
    }).collect()
}

// ============================================================================
// MODE 1: TRADITIONAL FULL SUITE
// ============================================================================
// pub fn run_full_two_phase_batch(
//     scheme: &SchemeConfig, 
//     batch_id: usize, 
//     num_keys: usize, 
//     iterations_per_key: usize,
//     level: u32
// ) -> Vec<BenchmarkRow> {
//     let timestamp = Utc::now().to_rfc3339();
//     let mut all_rows = Vec::new();

//     // PHASE 1: MODE 0 (Generate Key Arrays)
//     let mut cmd0 = build_cmd(scheme, 0, num_keys, None, level);
//     if !cmd0.spawn().and_then(|mut c| c.wait()).map(|s| s.success()).unwrap_or(false) {
//         return Vec::new();
//     }

//     let keygen_data = io::read_telemetry_file("sqisign_telemetry.bin");

//     // Extract Keys
//     let mut pk_bytes = Vec::new();
//     let mut sk_bytes = Vec::new();
//     let _ = File::open("fixed_pk.bin").and_then(|mut f| f.read_to_end(&mut pk_bytes));
//     let _ = File::open("fixed_sk.bin").and_then(|mut f| f.read_to_end(&mut sk_bytes));
    
//     let pk_size = if num_keys > 0 { pk_bytes.len() / num_keys } else { 0 };
//     let sk_size = if num_keys > 0 { sk_bytes.len() / num_keys } else { 0 };

//     let mut pk_hex_list = Vec::with_capacity(num_keys);
//     let mut sk_hex_list = Vec::with_capacity(num_keys);
//     for i in 0..num_keys {
//         if pk_bytes.len() >= (i + 1) * pk_size && sk_bytes.len() >= (i + 1) * sk_size {
//             let pk_chunk = &pk_bytes[(i * pk_size)..((i + 1) * pk_size)];
//             let sk_chunk = &sk_bytes[(i * sk_size)..((i + 1) * sk_size)];
//             pk_hex_list.push(pk_chunk.iter().map(|b| format!("{:02X}", b)).collect::<String>());
//             sk_hex_list.push(sk_chunk.iter().map(|b| format!("{:02X}", b)).collect::<String>());
//         } else {
//             pk_hex_list.push("ERR".to_string());
//             sk_hex_list.push("ERR".to_string());
//         }
//     }

//     // PHASE 2: MODE 1 (Sign/Verify)
//     let mut cmd1 = build_cmd(scheme, 1, num_keys, Some(iterations_per_key), level);
//     if !cmd1.spawn().and_then(|mut c| c.wait()).map(|s| s.success()).unwrap_or(false) {
//         return Vec::new();
//     }

//     let thermal_state = get_thermal_pressure_state();
//     let sv_data = io::read_telemetry_file("sqisign_telemetry.bin");

//     // PHASE 3: Zippering
//     for k in 0..num_keys as u64 {
//         let kg_row = match keygen_data.iter().find(|r| r.key_index == k) {
//             Some(row) => row,
//             None => continue,
//         };
        
//         let key_runs: Vec<&TelemetryRow> = sv_data.iter().filter(|r| r.key_index == k).collect();
//         if key_runs.is_empty() { continue; }

//         let sign_strings: Vec<String> = key_runs.iter().map(|r| format!("{:.3}", r.sign_cycles as f64 / 1_000_000.0)).collect();
//         let verify_strings: Vec<String> = key_runs.iter().map(|r| format!("{:.3}", r.verify_cycles as f64 / 1_000_000.0)).collect();

//         all_rows.push(BenchmarkRow {
//             timestamp: timestamp.clone(),
//             variant: scheme.name.to_string(),
//             batch_id,
//             key_index: k,
//             message_hex: "VARIOUS_RANDOM".to_string(),
//             public_key_hex: pk_hex_list[k as usize].clone(),
//             secret_key_hex: sk_hex_list[k as usize].clone(),
//             thermal_state,
//             level,
//             keygen_cycles: kg_row.keygen_cycles as f64 / 1_000_000.0,
//             sign_cycles_array: sign_strings.join(","),
//             verify_cycles_array: verify_strings.join(","),
//         });
//     }

//     all_rows
// }


// ============================================================================
// MODE 2: FIXED KEY, RANDOM MESSAGES
// ============================================================================
pub fn run_fixed_key_batch(
    scheme: &SchemeConfig, 
    batch_id: usize, 
    keys: usize, 
    iterations: usize, 
    level: u32,
    warmup: usize
) -> Vec<BenchmarkRow> {
    let timestamp = Utc::now().to_rfc3339();
    let mut cmd = build_cmd(scheme, 2, keys, Some(iterations), level, warmup);

    if !cmd.spawn().and_then(|mut c| c.wait()).map(|s| s.success()).unwrap_or(false) {
        eprintln!("[-] {} binary crashed during Mode 2!", scheme.language);
        return Vec::new();
    }

    let thermal_state = get_thermal_pressure_state();
    let raw_data = io::read_telemetry_file("sqisign_telemetry.bin");
    
    if raw_data.is_empty() { return Vec::new(); }

    // 1. Slurp the batch-specific keys and messages
    let mut pk_bytes = Vec::new();
    let _ = File::open("batch_pk.bin").and_then(|mut f| f.read_to_end(&mut pk_bytes));
    let pk_hex = pk_bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>();

    let mut sk_bytes = Vec::new();
    let _ = File::open("batch_sk.bin").and_then(|mut f| f.read_to_end(&mut sk_bytes));
    let sk_hex = sk_bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>();

    let mut msg_bytes = Vec::new();
    let _ = File::open("batch_messages.bin").and_then(|mut f| f.read_to_end(&mut msg_bytes));

    let batch_keygen_cycles = raw_data[0].keygen_cycles as f64 / 1_000_000.0;
    
    // 2. Map every single signature iteration into its own independent CSV row
    raw_data.into_iter().enumerate().map(|(message_idx, row)| {
        
        // Extract the specific 32-byte message hash for this iteration
        let msg_hex = if msg_bytes.len() >= (message_idx + 1) * 32 {
            let chunk = &msg_bytes[(message_idx * 32)..((message_idx + 1) * 32)];
            chunk.iter().map(|b| format!("{:02X}", b)).collect::<String>()
        } else {
            "ERR_MSG_NOT_FOUND".to_string()
        };

        BenchmarkRow {
            timestamp: timestamp.clone(),
            variant: scheme.name.to_string(),
            batch_id,
            key_index: message_idx as u64, // Repurposed to track message iteration
            message_hex: msg_hex,
            public_key_hex: pk_hex.clone(),
            secret_key_hex: sk_hex.clone(),
            thermal_state,
            level,
            keygen_cycles: batch_keygen_cycles,
            sign_cycles_array: format!("{:.3}", row.sign_cycles as f64 / 1_000_000.0),
            verify_cycles_array: format!("{:.3}", row.verify_cycles as f64 / 1_000_000.0),
        }
    }).collect()
}

// ============================================================================
// MODE 2: FIXED KEY, RANDOM MESSAGES (Normalized Relational Output)
// ============================================================================
// pub fn run_fixed_key_batch(
//     scheme: &SchemeConfig, 
//     batch_id: usize, 
//     keys: usize, 
//     iterations: usize, 
//     level: u32
// ) -> (Vec<KeyMetadataRow>, Vec<SignatureTelemetryRow>) {
//     let timestamp = Utc::now().to_rfc3339();
//     let mut cmd = build_cmd(scheme, 2, keys, Some(iterations), level);

//     if !cmd.spawn().and_then(|mut c| c.wait()).map(|s| s.success()).unwrap_or(false) {
//         eprintln!("[-] {} binary crashed during Mode 2!", scheme.language);
//         return (Vec::new(), Vec::new());
//     }

//     let thermal_state = get_thermal_pressure_state();
//     let raw_data = io::read_telemetry_file("sqisign_telemetry.bin");
//     if raw_data.is_empty() { return (Vec::new(), Vec::new()); }

//     // 1. Read the single batch key
//     let mut pk_bytes = Vec::new();
//     let _ = File::open("batch_pk.bin").and_then(|mut f| f.read_to_end(&mut pk_bytes));
//     let pk_hex = pk_bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>();

//     let mut sk_bytes = Vec::new();
//     let _ = File::open("batch_sk.bin").and_then(|mut f| f.read_to_end(&mut sk_bytes));
//     let sk_hex = sk_bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>();

//     // 2. Build the SINGLE Metadata Row for this batch
//     let batch_keygen_cycles = raw_data[0].keygen_cycles as f64 / 1_000_000.0;
    
//     let metadata_rows = vec![KeyMetadataRow {
//         timestamp: timestamp.clone(),
//         variant: scheme.name.to_string(),
//         level,
//         batch_id,
//         key_index: 0, // This is Key 0 for this batch
//         public_key_hex: pk_hex,
//         secret_key_hex: sk_hex,
//         keygen_cycles: batch_keygen_cycles,
//     }];

//     // 3. Read the messages and build the 1,000 Telemetry Rows
//     let mut msg_bytes = Vec::new();
//     let _ = File::open("batch_messages.bin").and_then(|mut f| f.read_to_end(&mut msg_bytes));

//     let telemetry_rows = raw_data.into_iter().enumerate().map(|(message_idx, row)| {
        
//         let msg_hex = if msg_bytes.len() >= (message_idx + 1) * 32 {
//             let chunk = &msg_bytes[(message_idx * 32)..((message_idx + 1) * 32)];
//             chunk.iter().map(|b| format!("{:02X}", b)).collect::<String>()
//         } else {
//             "ERR_MSG_NOT_FOUND".to_string()
//         };

//         SignatureTelemetryRow {
//             batch_id,
//             key_index: 0, // Points back to Key 0 in the metadata table
//             iteration_index: message_idx as u64, 
//             message_hex: msg_hex,
//             thermal_state,
//             sign_cycles: row.sign_cycles as f64 / 1_000_000.0,
//             verify_cycles: row.verify_cycles as f64 / 1_000_000.0,
//         }
//     }).collect();

//     (metadata_rows, telemetry_rows)
// }

// ============================================================================
// MODE 3: FIXED MESSAGE, RANDOM KEYS
// ============================================================================
pub fn run_fixed_msg_batch(
    scheme: &SchemeConfig, 
    batch_id: usize, 
    num_keys: usize, 
    level: u32,
    warmup: usize
) -> Vec<BenchmarkRow> {
    let timestamp = Utc::now().to_rfc3339();
    let mut cmd = build_cmd(scheme, 3, num_keys, None, level, warmup);

    if !cmd.spawn().and_then(|mut c| c.wait()).map(|s| s.success()).unwrap_or(false) {
        eprintln!("[-] {} binary crashed during Mode 3!", scheme.language);
        return Vec::new();
    }

    let thermal_state = get_thermal_pressure_state();
    let raw_data = io::read_telemetry_file("sqisign_telemetry.bin");
    if raw_data.is_empty() { return Vec::new(); }

    // 1. Read the single static 32-byte message challenge for this batch
    let mut msg_bytes = Vec::new();
    let _ = File::open("batch_msg_single.bin").and_then(|mut f| f.read_to_end(&mut msg_bytes));
    let static_msg_hex = msg_bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>();

    // 2. Read and slice all 1,000 generated keys from disk
    let mut pk_bytes = Vec::new();
    let mut sk_bytes = Vec::new();
    let _ = File::open("batch_pk.bin").and_then(|mut f| f.read_to_end(&mut pk_bytes));
    let _ = File::open("batch_sk.bin").and_then(|mut f| f.read_to_end(&mut sk_bytes));
    
    let pk_size = if num_keys > 0 { pk_bytes.len() / num_keys } else { 0 };
    let sk_size = if num_keys > 0 { sk_bytes.len() / num_keys } else { 0 };

    // 3. Map every unique key evaluation to its own complete CSV row
    raw_data.into_iter().enumerate().map(|(idx, row)| {
        
        let pk_hex = if pk_bytes.len() >= (idx + 1) * pk_size && pk_size > 0 {
            let chunk = &pk_bytes[(idx * pk_size)..((idx + 1) * pk_size)];
            chunk.iter().map(|b| format!("{:02X}", b)).collect::<String>()
        } else {
            "ERR_PK_NOT_FOUND".to_string()
        };

        let sk_hex = if sk_bytes.len() >= (idx + 1) * sk_size && sk_size > 0 {
            let chunk = &sk_bytes[(idx * sk_size)..((idx + 1) * sk_size)];
            chunk.iter().map(|b| format!("{:02X}", b)).collect::<String>()
        } else {
            "ERR_SK_NOT_FOUND".to_string()
        };

        BenchmarkRow {
            timestamp: timestamp.clone(),
            variant: scheme.name.to_string(),
            batch_id,
            key_index: row.key_index,
            message_hex: static_msg_hex.clone(), // Same static message for all 1000 keys in this batch!
            public_key_hex: pk_hex,              // Unique public key per row!
            secret_key_hex: sk_hex,              // Unique secret key per row!
            thermal_state,
            level,
            keygen_cycles: row.keygen_cycles as f64 / 1_000_000.0,
            sign_cycles_array: format!("{:.3}", row.sign_cycles as f64 / 1_000_000.0),
            verify_cycles_array: format!("{:.3}", row.verify_cycles as f64 / 1_000_000.0),
        }
    }).collect()
}

// ============================================================================
// MODE 3: FIXED MESSAGE, RANDOM KEYS (Normalized Relational Output)
// ============================================================================
// pub fn run_fixed_msg_batch(
//     scheme: &SchemeConfig, 
//     batch_id: usize, 
//     num_keys: usize, 
//     level: u32
// ) -> (Vec<KeyMetadataRow>, Vec<SignatureTelemetryRow>) {
//     let timestamp = Utc::now().to_rfc3339();
//     let mut cmd = build_cmd(scheme, 3, num_keys, None, level);

//     if !cmd.spawn().and_then(|mut c| c.wait()).map(|s| s.success()).unwrap_or(false) {
//         eprintln!("[-] {} binary crashed during Mode 3!", scheme.language);
//         return (Vec::new(), Vec::new());
//     }

//     let thermal_state = get_thermal_pressure_state();
//     let raw_data = io::read_telemetry_file("sqisign_telemetry.bin");
//     if raw_data.is_empty() { return (Vec::new(), Vec::new()); }

//     // 1. Read the single static message challenge
//     let mut msg_bytes = Vec::new();
//     let _ = File::open("batch_msg_single.bin").and_then(|mut f| f.read_to_end(&mut msg_bytes));
//     let static_msg_hex = msg_bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>();

//     // 2. Read the 1,000 generated keys
//     let mut pk_bytes = Vec::new();
//     let mut sk_bytes = Vec::new();
//     let _ = File::open("batch_pk.bin").and_then(|mut f| f.read_to_end(&mut pk_bytes));
//     let _ = File::open("batch_sk.bin").and_then(|mut f| f.read_to_end(&mut sk_bytes));
    
//     let pk_size = if num_keys > 0 { pk_bytes.len() / num_keys } else { 0 };
//     let sk_size = if num_keys > 0 { sk_bytes.len() / num_keys } else { 0 };

//     // 3. Map into the Tuple and Unzip into two separate Vectors
//     let (metadata_rows, telemetry_rows): (Vec<KeyMetadataRow>, Vec<SignatureTelemetryRow>) = 
//         raw_data.into_iter().enumerate().map(|(idx, row)| {
        
//         let pk_hex = if pk_bytes.len() >= (idx + 1) * pk_size && pk_size > 0 {
//             let chunk = &pk_bytes[(idx * pk_size)..((idx + 1) * pk_size)];
//             chunk.iter().map(|b| format!("{:02X}", b)).collect::<String>()
//         } else {
//             "ERR_PK_NOT_FOUND".to_string()
//         };

//         let sk_hex = if sk_bytes.len() >= (idx + 1) * sk_size && sk_size > 0 {
//             let chunk = &sk_bytes[(idx * sk_size)..((idx + 1) * sk_size)];
//             chunk.iter().map(|b| format!("{:02X}", b)).collect::<String>()
//         } else {
//             "ERR_SK_NOT_FOUND".to_string()
//         };

//         let meta = KeyMetadataRow {
//             timestamp: timestamp.clone(),
//             variant: scheme.name.to_string(),
//             level,
//             batch_id,
//             key_index: row.key_index, // Will count 0 to 999
//             public_key_hex: pk_hex,
//             secret_key_hex: sk_hex,
//             keygen_cycles: row.keygen_cycles as f64 / 1_000_000.0,
//         };

//         let telem = SignatureTelemetryRow {
//             batch_id,
//             key_index: row.key_index, // Links directly back to the meta row
//             iteration_index: 0,       // Always 0! Each key only signs the static message once.
//             message_hex: static_msg_hex.clone(), // Same message across all 1000 rows
//             thermal_state,
//             sign_cycles: row.sign_cycles as f64 / 1_000_000.0,
//             verify_cycles: row.verify_cycles as f64 / 1_000_000.0,
//         };

//         (meta, telem)
//     }).unzip();

//     (metadata_rows, telemetry_rows)
// }

// ============================================================================
// COOLDOWN
// ============================================================================
pub fn thermal_cooldown(seconds: u64) {
    println!("    - Enforcing {}s thermal dissipation...", seconds);
    thread::sleep(Duration::from_secs(seconds));
}