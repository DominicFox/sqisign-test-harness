# SQISign-Test-Harness
This repository contains the code for my MSc Thesis.

It is a custom benchmarking test harness designed to evaluate the stack memory usage and execution latency of the currently implemented SQISign variants.

# Running The Code
## System Requirements
* **Architecture:** Native AArch64 environment (e.g., Apple Silicon).
* **Toolchain:** Rust (`cargo`), `cmake`, `make`, and a C compiler (`clang`/`gcc`).
* **Julia:** Required specifically for executing the SQISign2D-East variant.

The 2D-East variant relies on Julia and requires the `Nemo` algebra package. To install it, run the following command in your terminal:
```bash
julia -e 'using Pkg; Pkg.add("Nemo")'
```

## 1. Clone the Repository
To ensure all cryptographic submodules are fetched correctly, you must clone the repository recursively. 

```bash
git clone --recurse-submodules https://github.com/DominicFox/sqisign-test-harness.git
cd sqisign-test-harness
```

## 2. Compile C Binaries
Before running the orchestrator, the C binaries for telemetry extraction must be compiled within each of the variant submodules.

For **each** variant in the repository, you must execute the following build sequence from the root of that folder (Note: For the `SQISignHD` variant, you must navigate into its `Signature` subfolder first before creating the build directory)
Execute these commands inside each target directory:
```bash
mkdir build
cd build
cmake -DSQISIGN_BUILD_TYPE=ref -DCMAKE_BUILD_TYPE=Release -DCMAKE_C_FLAGS="-Wno-macro-redefined" ..
make
```

## 3. Execute the Benchmarking Suite
Once all C binaries have compiled, return to the root directory of the main repository.
Run the Rust orchestrator using elevated privileges. `sudo` is required because the benchmarking harness directly reads CPU cycle counters from the kernel to ensure latency measurement integrity.
```bash
sudo cargo run --release
```

**Default Execution Parameters:**
By default, the orchestrator will evaluate each variant across all applicable NIST Security Levels.
- **Latency Telemetry:** Executes 1 batch of 100 iterations for each execution mode (KeyGen, Fixed Key, Fixed Message).
- **Stack Profiling:** Enabled by default, capturing 1 batch of 50 iterations per variant.

### Customising Benchmarking Parameters
Execution modes, iteration counts, and active variants are configured directly in the orchestrator's source code.

To modify the default behaviour, open `src/main.rs` and locate the `BENCHMARK PARAMETERS` block. You can adjust the following variables before running `sudo cargo run --release`:
- **Execution Modes:** Toggle `run_keygen_profiling`, `run_fixed_key_suite`, `run_fixed_msg_suite`, or `run_stack_suite` to `true`/`false` to isolate specific benchmarking modes.
- **Iteration Counts:** Modify variables like `keys_for_profiling = 100`, `num_rand_messages = 100`, or `stack_iterations = 50` to increase or decrease the sample sizes.
- **Target Variants:** To disable a specific SQISign variant or change its target NIST security levels (1, 3, or 5), modify its `SchemeConfig` block within the `schemes` vector (e.g., set `is_enabled: false`).

## 4. Output Data
Upon execution, the orchestrator automatically routes all captured telemetry to the `results/` directory at the root of the repository.
Files are cleanly separated by variant and execution mode. Depending on the active parameters, you will find the following dataset structures:
- `*_keygen_benchmark.csv`: Dedicated key generation latency and CPU cycle metrics.
- `*_fixed_key_benchmark.csv`: Signing and verification telemetry spanning $N$ messages signed under a single fixed key pair.
- `*_fixed_msg_benchmark.csv`: Signing and verification telemetry across $N$ distinct key pairs signing identical messages.
- `*_stack_benchmark.csv`: High-water mark stack memory consumption profiles (measured in bytes) for the scheme's core operations.






