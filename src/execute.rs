use std::process::Command;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Read;
use chrono::Utc;

use crate::models::{BenchmarkRow, SchemeConfig, TelemetryRow, KeygenRow, StackRow};
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

// On macOS, this function attempts to set the process priority to the highest level (-20) to reduce scheduling delays during benchmarking. 
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
// MODE 1: FIXED KEY, RANDOM MESSAGES
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
    let mut cmd = build_cmd(scheme, 1, keys, Some(iterations), level, warmup);

    if !cmd.spawn().and_then(|mut c| c.wait()).map(|s| s.success()).unwrap_or(false) {
        eprintln!("[-] {} binary crashed during Mode 1!", scheme.language);
        return Vec::new();
    }

    let thermal_state = get_thermal_pressure_state();
    let raw_data = io::read_telemetry_file("sqisign_telemetry.bin");
    
    if raw_data.is_empty() { return Vec::new(); }

    // Slurp the batch-specific keys and messages
    let mut pk_bytes = Vec::new();
    let _ = File::open("batch_pk.bin").and_then(|mut f| f.read_to_end(&mut pk_bytes));
    let pk_hex = pk_bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>();

    let mut sk_bytes = Vec::new();
    let _ = File::open("batch_sk.bin").and_then(|mut f| f.read_to_end(&mut sk_bytes));
    let sk_hex = sk_bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>();

    let mut msg_bytes = Vec::new();
    let _ = File::open("batch_messages.bin").and_then(|mut f| f.read_to_end(&mut msg_bytes));

    let batch_keygen_cycles = raw_data[0].keygen_cycles as f64 / 1_000_000.0;
    
    // Map every single signature iteration into its own independent CSV row
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
// MODE 2: FIXED MESSAGE, RANDOM KEYS
// ============================================================================
pub fn run_fixed_msg_batch(
    scheme: &SchemeConfig, 
    batch_id: usize, 
    num_keys: usize, 
    level: u32,
    warmup: usize
) -> Vec<BenchmarkRow> {
    let timestamp = Utc::now().to_rfc3339();
    let mut cmd = build_cmd(scheme, 2, num_keys, None, level, warmup);

    if !cmd.spawn().and_then(|mut c| c.wait()).map(|s| s.success()).unwrap_or(false) {
        eprintln!("[-] {} binary crashed during Mode 2!", scheme.language);
        return Vec::new();
    }

    let thermal_state = get_thermal_pressure_state();
    let raw_data = io::read_telemetry_file("sqisign_telemetry.bin");
    if raw_data.is_empty() { return Vec::new(); }

    // Read the single static 32-byte message challenge for this batch
    let mut msg_bytes = Vec::new();
    let _ = File::open("batch_msg_single.bin").and_then(|mut f| f.read_to_end(&mut msg_bytes));
    let static_msg_hex = msg_bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>();

    // Read and slice all generated keys from disk
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
// MODE 3: STACK HIGH-WATER PROFILING
// ============================================================================
// Unlike the cycle-count modes this metric is fairly deterministic given the inputs:
// it is unaffected by thermal state, E/P core placement or scheduler noise.
// No warmup, no cooldown, and far fewer iterations are needed.
pub fn run_stack_batch(
    scheme: &SchemeConfig,
    batch_id: usize,
    iterations: usize,
    level: u32,
) -> Vec<StackRow> {
    if scheme.stack_path.is_empty() {
        return Vec::new();
    }

    let timestamp = Utc::now().to_rfc3339();
    let binary = format!("{}_lvl{}", scheme.stack_path, level);
    let telemetry_file = "sqisign_stack_telemetry.bin";

    let mut cmd = Command::new(&binary);
    cmd.arg(format!("--iterations={}", iterations))
       .arg(format!("--out={}", telemetry_file));

    match cmd.spawn().and_then(|mut c| c.wait()) {
        Ok(status) if status.success() => {}
        Ok(_) => {
            eprintln!("[-] {} stack probe exited non-zero!", scheme.name);
            return Vec::new();
        }
        Err(e) => {
            eprintln!("[-] Failed to spawn stack probe '{}': {}", binary, e);
            return Vec::new();
        }
    }

    let raw_data = io::read_stack_telemetry_file(telemetry_file);
    let mut rows = Vec::with_capacity(raw_data.len() * 3);

    // Melt each iteration into one row per primitive.
    for r in raw_data {
        for (primitive, stack, resident) in [
            ("keygen", r.keygen_stack, r.keygen_resident),
            ("sign", r.sign_stack, r.sign_resident),
            ("verify", r.verify_stack, r.verify_resident),
        ] {
            rows.push(StackRow {
                timestamp: timestamp.clone(),
                variant: scheme.name.to_string(),
                batch_id,
                level,
                iteration: r.iter,
                primitive: primitive.to_string(),
                stack_bytes: stack,
                resident_bytes: resident,
                ok: r.ok,
            });
        }
    }

    rows
}

// ============================================================================
// COOLDOWN
// ============================================================================
pub fn thermal_cooldown(seconds: u64) {
    println!("    - Enforcing {}s thermal dissipation...", seconds);
    thread::sleep(Duration::from_secs(seconds));
}