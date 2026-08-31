mod models;
mod execute;
mod io; 

use models::SchemeConfig;


fn main() {
    // Attempt to force high process priority on macOS to reduce scheduler noise.
    execute::apply_macos_thread_policies();

    let schemes = vec![
        SchemeConfig { 
            name: "SQISign NIST Round 2 Submission", 
            path: "variant/sqisign-v2/build/apps/benchmark_binary", 
            language: "C",
            levels: vec![1, 3, 5],
            output_csv_fixed_key: "results/sqisign_fixed_key_benchmark.csv",
            output_csv_fixed_msg: "results/sqisign_fixed_msg_benchmark.csv",
            output_csv_keygen: "results/sqisign_keygen_benchmark.csv",
            stack_path: "variant/sqisign-v2/build/apps/stack_probe",
            output_csv_stack: "results/sqisign_stack_benchmark.csv",
            is_enabled: true
        },
        SchemeConfig { 
            name: "SQISign2D-West", 
            path: "variant/sqisign-2dwest/build/test/test_sqisigndim2_modified", 
            language: "C",
            levels: vec![1, 3, 5],
            output_csv_fixed_key: "results/sqisign2dwest_fixed_key_benchmark.csv",
            output_csv_fixed_msg: "results/sqisign2dwest_fixed_msg_benchmark.csv",
            output_csv_keygen: "results/sqisign2dwest_keygen_benchmark.csv",
            stack_path: "variant/sqisign-2dwest/build/test/stack_probe_2dwest",
            output_csv_stack: "results/sqisign2dwest_stack_benchmark.csv",
            is_enabled: true
        },
        SchemeConfig { 
            name: "SQISign2D-West Heuristic", 
            path: "variant/sqisign-2dwest/build/test/test_sqisigndim2_heuristic_modified", 
            language: "C",
            levels: vec![1, 3, 5],
            output_csv_fixed_key: "results/sqisign2dwestheuristic_fixed_key_benchmark.csv",
            output_csv_fixed_msg: "results/sqisign2dwestheuristic_fixed_msg_benchmark.csv",
            output_csv_keygen: "results/sqisign2dwestheuristic_keygen_benchmark.csv",
            stack_path: "variant/sqisign-2dwest/build/test/stack_probe_2dwestheuristic",
            output_csv_stack: "results/sqisign2dwestheuristic_stack_benchmark.csv",
            is_enabled: true
        },
        SchemeConfig { 
            name: "SQISignHD", 
            path: "variant/sqisignhd/Signature/build/test/sqisign_test_sqisignhd_modified",
            language: "C",
            levels: vec![1, 3, 5],
            output_csv_fixed_key: "results/sqisignhd_fixed_key_benchmark.csv",
            output_csv_fixed_msg: "results/sqisignhd_fixed_msg_benchmark.csv",
            output_csv_keygen: "results/sqisignhd_keygen_benchmark.csv",
            stack_path: "variant/sqisignhd/Signature/build/test/stack_probe_hd",
            output_csv_stack: "results/sqisignhd_stack_benchmark.csv",
            is_enabled: true
        },
        SchemeConfig { 
            name: "SQISign NIST Round 1 Submission", 
            path: "variant/sqisign-v1/build/apps/benchmark_binary", 
            language: "C",
            levels: vec![1, 3, 5],
            output_csv_fixed_key: "results/sqisignv1_fixed_key_benchmark.csv",
            output_csv_fixed_msg: "results/sqisignv1_fixed_msg_benchmark.csv",
            output_csv_keygen: "results/sqisignv1_keygen_benchmark.csv",
            stack_path: "variant/sqisign-v1/build/apps/stack_probe",
            output_csv_stack: "results/sqisignv1_stack_benchmark.csv",
            is_enabled: true
        },
        SchemeConfig { 
            name: "SQISign2D-East", 
            path: "variant/sqisign-2deast/benchmark_2deast.jl", 
            language: "Julia",
            levels: vec![1, 3, 5],
            output_csv_fixed_key: "results/sqisign2deast_fixed_key_benchmark.csv",
            output_csv_fixed_msg: "results/sqisign2deast_fixed_msg_benchmark.csv",
            output_csv_keygen: "results/sqisign2deast_keygen_benchmark.csv",
            stack_path: "",
            output_csv_stack: "results/sqisign2deast_stack_benchmark.csv",
            is_enabled: true
        },
        
    ];

    // ============================================================================
    //                             BENCHMARK PARAMETERS
    // ============================================================================

    // GLOBAL PARAMS
    let total_batches = 5;
    let warmup_iterations = 15;
    let thermal_secs = 15;

    // MODE 0: KEYGEN PROFILING PARAMS (Lots of keys, no iteration)
    let run_keygen_profiling = true;  
    let keys_for_profiling = 100;

    // MODE 1: FIXED KEY PARAMS (1 Key, N Messages per batch)
    let run_fixed_key_suite = true; 
    let num_rand_messages = 100;
    
    // MODE 2: FIXED MSG PARAMS (N Keys, 1 Message per batch)
    let run_fixed_msg_suite = true; 
    let num_keys = 100;

    // MODE 3: STACK PROFILING PARAMS
    let run_stack_suite = true;
    let stack_iterations = 50;
    let stack_batches = 1;
    

    println!("[*] Starting Benchmark Suite");

    for scheme in schemes.into_iter().filter(|s| s.is_enabled) {

        for &level in &scheme.levels {
            println!("\n#############################################################################");
            println!(" Initiating Suite for: {} (NIST Level {})", scheme.name, level);
            println!("#############################################################################");
            
            // ---------------------------------------------------------
            // MODE 0: Dedicated Keygen Profiling
            // ---------------------------------------------------------
            if run_keygen_profiling {
                println!("\n>>> MODE 0: KEYGEN PROFILING (L{})", level);
                for batch_id in 1..=total_batches {
                    println!("\n  > Batch {} of {}", batch_id, total_batches);
                    let keygen_rows = execute::run_keygen_only_batch(&scheme, batch_id, keys_for_profiling, level, warmup_iterations);
                    
                    if !keygen_rows.is_empty() {
                        io::write_keygen_csv(keygen_rows, scheme.output_csv_keygen);
                    }
                    execute::thermal_cooldown(thermal_secs);
                }
            }

            // ---------------------------------------------------------
            // MODE 1: Fixed Key Suite
            // ---------------------------------------------------------
            if run_fixed_key_suite {
                println!("\n>>> MODE 1: FIXED KEY (SIGN/VERIFY) (L{})", level);
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
            // MODE 2: Fixed Message Suite
            // ---------------------------------------------------------
            if run_fixed_msg_suite {
                println!("\n>>> MODE 2: FIXED MSG (SIGN/VERIFY) (L{})", level);
                for batch_id in 1..=total_batches {
                    println!("\n  > Batch {} of {}", batch_id, total_batches);
                    let benchmark_rows = execute::run_fixed_msg_batch(&scheme, batch_id, num_keys, level, warmup_iterations);
                    
                    if !benchmark_rows.is_empty() {
                        io::write_full_csv(benchmark_rows, scheme.output_csv_fixed_msg);
                    }
                    execute::thermal_cooldown(thermal_secs);
                }
            }

            // ---------------------------------------------------------
            // MODE 3: Stack Profiling
            // ---------------------------------------------------------
            if run_stack_suite {
                if scheme.stack_path.is_empty() {
                    println!("\n>>> MODE 3: STACK PROFILING (L{}) -- skipped, no probe for {}", level, scheme.language);
                } else {
                    println!("\n>>> MODE 3: STACK PROFILING (L{})", level);
                    for batch_id in 1..=stack_batches {
                        println!("\n  > Batch {} of {}", batch_id, stack_batches);
                        let stack_rows = execute::run_stack_batch(&scheme, batch_id, stack_iterations, level);

                        if !stack_rows.is_empty() {
                            io::write_stack_csv(stack_rows, scheme.output_csv_stack);
                        }
                        // No cooldown: peak stack depth does not depend on temperature.
                    }
                }
            }
        }
    }
    
    println!("\n[*] Run Complete. Data safely stored.");
}
