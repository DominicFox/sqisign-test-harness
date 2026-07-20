mod models;
mod execute;
mod io; // Assuming you put read_telemetry_file and write_to_csv here

use models::SchemeConfig;


fn main() {
    execute::apply_macos_thread_policies();

    let schemes = vec![
        SchemeConfig { 
            name: "SQISign NIST Round 2 Submission", 
            path: "../Schemes/the-sqisign/build/apps/benchmark_binary", 
            language: "C",
            levels: vec![1, 3, 5],
            output_csv_fixed_key: "results/sqisign_fixed_key_benchmark.csv",
            output_csv_fixed_msg: "results/sqisign_fixed_msg_benchmark.csv",
            output_csv_keygen: "results/sqisign_keygen_benchmark.csv",
            is_enabled: false
        },
        SchemeConfig { 
            name: "SQISign2D-West", 
            path: "../Schemes/sqisign2d-west-ac24/build/test/test_sqisigndim2_modified", 
            language: "C",
            levels: vec![1, 3, 5],
            output_csv_fixed_key: "results/sqisign2dwest_fixed_key_benchmark.csv",
            output_csv_fixed_msg: "results/sqisign2dwest_fixed_msg_benchmark.csv",
            output_csv_keygen: "results/sqisign2dwest_keygen_benchmark.csv",
            is_enabled: false
        },
        SchemeConfig { 
            name: "SQISign2D-West Heuristic", 
            path: "../Schemes/sqisign2d-west-ac24/build/test/test_sqisigndim2_heuristic_modified", 
            language: "C",
            levels: vec![1, 3, 5],
            output_csv_fixed_key: "results/sqisign2dwestheuristic_fixed_key_benchmark.csv",
            output_csv_fixed_msg: "results/sqisign2dwestheuristic_fixed_msg_benchmark.csv",
            output_csv_keygen: "results/sqisign2dwestheuristic_keygen_benchmark.csv",
            is_enabled: false
        },
        SchemeConfig { 
            name: "SQISignHD", 
            path: "../Schemes/SQISignHD-lib/Signature/build/test/sqisign_test_sqisignhd_modified",
            language: "C",
            levels: vec![1, 3, 5],
            output_csv_fixed_key: "results/sqisignhd_fixed_key_benchmark.csv",
            output_csv_fixed_msg: "results/sqisignhd_fixed_msg_benchmark.csv",
            output_csv_keygen: "results/sqisignhd_keygen_benchmark.csv",
            is_enabled: false
        },
        SchemeConfig { 
            name: "SQISign NIST Round 1 Submission", 
            path: "../Schemes/the-sqisign-v1/build/apps/benchmark_binary", 
            language: "C",
            levels: vec![3, 5],
            output_csv_fixed_key: "results/sqisignv1_fixed_key_benchmark.csv",
            output_csv_fixed_msg: "results/sqisignv1_fixed_msg_benchmark.csv",
            output_csv_keygen: "results/sqisignv1_keygen_benchmark.csv",
            is_enabled: true
        },
        SchemeConfig { 
            name: "SQISign2D-East", 
            path: "../Schemes/SQIsign2D-East.jl/benchmark_2deast.jl", //Just change number
            language: "Julia",
            levels: vec![1,3,5],
            output_csv_fixed_key: "results/sqisign2deast_fixed_key_benchmark.csv",
            output_csv_fixed_msg: "results/sqisign2deast_fixed_msg_benchmark.csv",
            output_csv_keygen: "results/sqisign2deast_keygen_benchmark.csv",
            is_enabled: true
        },
        
    ];

    // GLOBAL PARAMS
    let total_batches = 10;
    let warmup_iterations = 15;
    let thermal_secs = 15;

    // KEYGEN PROFILING PARAMS (Lots of keys, no iteration)
    let run_keygen_profiling = true;  // Test distribution of key generation
    let keys_for_profiling = 1000;

    // FIXED KEY PARAMS (1 Key, N Messages per batch)
    let run_fixed_key_suite = true; // Mode 2: 1 Key, 1000 random messages
    let num_rand_messages = 1000;
    let keys_for_suite = 1;
    
    // FIXED MSG PARAMS (N Keys, 1 Message per batch)
    let run_fixed_msg_suite = true; // Mode 3: 1000 Keys, 1 constant message
    let num_keys = 1000;
    


    // OLD FULL SUITE PARAMS (Low volume of keys, high iterations)
    // let run_full_suite = false;        // Test signature generation/verification
    // let keys_for_suite = 1;
    // let iterations_per_key = 5;    

    println!("[*] Starting Master Overnight Benchmark Suite");

    for scheme in schemes.into_iter().filter(|s| s.is_enabled) {

        for &level in &scheme.levels {
            println!("\n#############################################################################");
            println!(" Initiating Suite for: {} (NIST Level {})", scheme.name, level);
            println!("#############################################################################");
            
            // ---------------------------------------------------------
            // PASS 1: Dedicated Keygen Profiling
            // ---------------------------------------------------------
            if run_keygen_profiling {
                println!("\n>>> PHASE A: KEYGEN PROFILING (L{})", level);
                for batch_id in 1..=total_batches {
                    println!("\n  > Batch {} of {}", batch_id, total_batches);
                    let keygen_rows = execute::run_keygen_only_batch(&scheme, batch_id, keys_for_profiling, level, warmup_iterations);
                    
                    if !keygen_rows.is_empty() {
                        io::write_keygen_csv(keygen_rows, scheme.output_csv_keygen);
                    }
                    execute::thermal_cooldown(thermal_secs);
                }
            }

            // if run_fixed_key_suite {
            //     println!("\n>>> PHASE B: FIXED KEY (SIGN/VERIFY) (L{})", level);
                
            //     // Declare the split CSV paths
            //     let csv_meta = format!("{}_METADATA.csv", scheme.output_csv_fixed_key);
            //     let csv_telem = format!("{}_TELEMETRY.csv", scheme.output_csv_fixed_key);
                
            //     for batch_id in 1..=total_batches {
            //         println!("\n  > Batch {} of {}", batch_id, total_batches);
                    
            //         // Unpack the tuple
            //         let (meta_rows, telem_rows) = execute::run_fixed_key_batch(&scheme, batch_id, 1, num_rand_messages, level);
                    
            //         if !meta_rows.is_empty() {
            //             io::write_metadata_csv(meta_rows, &csv_meta);
            //         }
            //         if !telem_rows.is_empty() {
            //             io::write_telemetry_csv(telem_rows, &csv_telem);
            //         }
                    
            //         execute::thermal_cooldown(thermal_secs);
            //     }
            // }

            if run_fixed_msg_suite {
                println!("\n>>> PHASE B: FIXED KEY (SIGN/VERIFY) (L{})", level);
                for batch_id in 1..=total_batches {
                    println!("\n  > Batch {} of {}", batch_id, total_batches);
                    let benchmark_rows = execute::run_fixed_key_batch(&scheme, batch_id,1, num_rand_messages, level, warmup_iterations);
                    
                    if !benchmark_rows.is_empty() {
                        io::write_full_csv(benchmark_rows, scheme.output_csv_fixed_key);
                    }
                    execute::thermal_cooldown(thermal_secs);
                }
            }

            // ---------------------------------------------------------
            // PASS 2: Fixed Message, Random Keys (Mode 3)
            // ---------------------------------------------------------
            // if run_fixed_msg_suite {
            //     println!("\n>>> PHASE C: FIXED MESSAGE (SIGN/VERIFY) (L{})", level);
                
            //     // Declare the split CSV paths
            //     let csv_meta = format!("{}_METADATA.csv", scheme.output_csv_fixed_msg);
            //     let csv_telem = format!("{}_TELEMETRY.csv", scheme.output_csv_fixed_msg);
                
            //     for batch_id in 1..=total_batches {
            //         println!("\n  > Batch {} of {}", batch_id, total_batches);
                    
            //         // Unpack the tuple (Requesting 1000 keys)
            //         let (meta_rows, telem_rows) = execute::run_fixed_msg_batch(&scheme, batch_id, num_keys, level);
                    
            //         if !meta_rows.is_empty() {
            //             io::write_metadata_csv(meta_rows, &csv_meta);
            //         }
            //         if !telem_rows.is_empty() {
            //             io::write_telemetry_csv(telem_rows, &csv_telem);
            //         }
                    
            //         execute::thermal_cooldown(thermal_secs);
            //     }
            // }

            if run_fixed_msg_suite {
                println!("\n>>> PHASE C: FIXED MSG (SIGN/VERIFY) (L{})", level);
                for batch_id in 1..=total_batches {
                    println!("\n  > Batch {} of {}", batch_id, total_batches);
                    let benchmark_rows = execute::run_fixed_msg_batch(&scheme, batch_id, num_keys, level, warmup_iterations);
                    
                    if !benchmark_rows.is_empty() {
                        io::write_full_csv(benchmark_rows, scheme.output_csv_fixed_msg);
                    }
                    execute::thermal_cooldown(thermal_secs);
                }
            }
        }
    }
    
    println!("\n[*] Run Complete. Data safely stored.");
}
