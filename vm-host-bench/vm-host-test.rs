/// BMP VM Benchmark — vm-host
/// Measures: execution time, throughput (instructions/sec), overhead vs native Rust
/// Run: cargo run --release
/// Results printed to stdout — copy into your IEEE report

use std::time::Instant;
use vm_core::{Host, VmFlags, VM};

// ─── Silent host — no print overhead during benchmark ──────────────────────
struct BenchHost;

impl Host for BenchHost {
    fn print(&self, _value: i32) {}
    fn native_call(&self, _id: u8, _arg: i32) -> i32 { 0 }
    fn report_flags(&self, _flags: VmFlags) {}
}

// ─── Benchmark programs (hand-assembled bytecode) ──────────────────────────

/// Test 1: tight loop — x = 0, while x < 100: x = x + 1
/// Exercises: Push, Store, Load, CmpLt, JmpIfFalse, Add, Jmp, Halt
/// Instruction count per iteration: 9 instructions × 100 iters + 6 setup = 906
const LOOP_100: &[u8] = &[
    0x01, 0,    // [0]  Push 0
    0x21, 0,    // [2]  Store slot0  (x=0)
    0x01, 100,  // [4]  Push 100
    0x21, 1,    // [6]  Store slot1  (limit=100)
    // loop start at pc=8
    0x20, 0,    // [8]  Load slot0   (x)
    0x20, 1,    // [10] Load slot1   (limit)
    0x31,       // [12] CmpLt
    0x41, 22,   // [13] JmpIfFalse → exit at pc=22
    0x20, 0,    // [15] Load slot0
    0x01, 1,    // [17] Push 1
    0x02,       // [19] Add
    0x21, 0,    // [20] Store slot0
    0x40, 8,    // [22] Jmp → pc=8   ← WRONG, fix below
    // exit at pc=24
    0xFF,       // [24] Halt
];

// corrected: Jmp target must be 8, JmpIfFalse target must be 24
const LOOP_100_FIXED: &[u8] = &[
    0x01, 0,    // [0]  Push 0
    0x21, 0,    // [2]  Store slot0
    0x01, 100,  // [4]  Push 100
    0x21, 1,    // [6]  Store slot1
    // loop start pc=8
    0x20, 0,    // [8]  Load slot0
    0x20, 1,    // [10] Load slot1
    0x31,       // [12] CmpLt
    0x41, 23,   // [13] JmpIfFalse → 23 (halt)
    0x20, 0,    // [15] Load slot0
    0x01, 1,    // [17] Push 1
    0x02,       // [19] Add
    0x21, 0,    // [20] Store slot0
    0x40, 8,    // [22] Jmp → 8
    0xFF,       // [23] Halt
];

/// Test 2: arithmetic — repeated add/sub/mul operations
/// x = 0, while x < 200: x = x + 3
const ARITH_200: &[u8] = &[
    0x01, 0,    // [0]  Push 0
    0x21, 0,    // [2]  Store slot0
    0x01, 200,  // [4]  Push 200     ← max 255, fits u8
    0x21, 1,    // [6]  Store slot1
    // loop pc=8
    0x20, 0,    // [8]  Load slot0
    0x20, 1,    // [10] Load slot1
    0x31,       // [12] CmpLt
    0x41, 23,   // [13] JmpIfFalse → 23
    0x20, 0,    // [15] Load slot0
    0x01, 3,    // [17] Push 3
    0x02,       // [19] Add
    0x21, 0,    // [20] Store slot0
    0x40, 8,    // [22] Jmp → 8
    0xFF,       // [23] Halt
];

/// Test 3: memory access — two variables alternating load/store
/// x=0, y=0, while x<50: x=x+1, y=y+2
const MEM_ACCESS: &[u8] = &[
    0x01, 0,   // [0]  Push 0
    0x21, 0,   // [2]  Store slot0 (x=0)
    0x01, 0,   // [4]  Push 0
    0x21, 1,   // [6]  Store slot1 (y=0)
    0x01, 50,  // [8]  Push 50
    0x21, 2,   // [10] Store slot2 (limit=50)
    // loop pc=12
    0x20, 0,   // [12] Load x
    0x20, 2,   // [14] Load limit
    0x31,      // [16] CmpLt
    0x41, 35,  // [17] JmpIfFalse → 35
    0x20, 0,   // [19] Load x
    0x01, 1,   // [21] Push 1
    0x02,      // [23] Add
    0x21, 0,   // [24] Store x
    0x20, 1,   // [26] Load y
    0x01, 2,   // [28] Push 2
    0x02,      // [30] Add
    0x21, 1,   // [31] Store y
    0x40, 12,  // [33] Jmp → 12
    0xFF,      // [35] Halt
];

// ─── Benchmark runner ───────────────────────────────────────────────────────

fn run_once(bytecode: &[u8]) -> std::time::Duration {
    let host = BenchHost;
    let mut vm = VM::new(bytecode, host);
    let t0 = Instant::now();
    vm.run();
    t0.elapsed()
}

fn bench(name: &str, bytecode: &[u8], iters: u32, instruction_count: u64) {
    // Warmup
    for _ in 0..10 {
        let h = BenchHost;
        let mut vm = VM::new(bytecode, h);
        vm.run();
    }

    // Timed runs
    let mut durations = Vec::with_capacity(iters as usize);
    for _ in 0..iters {
        durations.push(run_once(bytecode));
    }

    let total_ns: u64 = durations.iter().map(|d| d.as_nanos() as u64).sum();
    let avg_ns = total_ns / iters as u64;
    let min_ns = durations.iter().map(|d| d.as_nanos() as u64).min().unwrap();
    let max_ns = durations.iter().map(|d| d.as_nanos() as u64).max().unwrap();

    // Variance
    let mean = avg_ns as f64;
    let variance: f64 = durations
        .iter()
        .map(|d| {
            let diff = d.as_nanos() as f64 - mean;
            diff * diff
        })
        .sum::<f64>() / iters as f64;
    let std_dev = variance.sqrt();

    // Throughput
    let instructions_per_sec = if avg_ns > 0 {
        (instruction_count as f64 / avg_ns as f64) * 1_000_000_000.0
    } else {
        0.0
    };

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  TEST: {}", name);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Runs           : {}", iters);
    println!("  Bytecode size  : {} bytes", bytecode.len());
    println!("  Avg time       : {} ns  ({:.3} µs)", avg_ns, avg_ns as f64 / 1000.0);
    println!("  Min time       : {} ns", min_ns);
    println!("  Max time       : {} ns", max_ns);
    println!("  Std deviation  : {:.2} ns", std_dev);
    println!("  Instructions   : {}", instruction_count);
    println!("  Throughput     : {:.2} M instr/sec", instructions_per_sec / 1_000_000.0);
    println!();
}

fn bench_native_loop(iters: u32) {
    // Equivalent Rust loop — no VM overhead
    fn native_loop_100() -> i32 {
        let mut x = 0i32;
        while x < 100 { x += 1; }
        x
    }

    // Warmup
    for _ in 0..10 { native_loop_100(); }

    let mut durations = Vec::with_capacity(iters as usize);
    for _ in 0..iters {
        let t0 = Instant::now();
        native_loop_100();
        durations.push(t0.elapsed());
    }

    let total_ns: u64 = durations.iter().map(|d| d.as_nanos() as u64).sum();
    let avg_ns = total_ns / iters as u64;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  NATIVE RUST: equivalent loop (x<100, x+=1)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Avg time       : {} ns  ({:.3} µs)", avg_ns, avg_ns as f64 / 1000.0);
    println!("  NOTE: ratio printed below");
    println!();

    avg_ns
}

fn main() {
    println!();
    println!("╔══════════════════════════════════════════╗");
    println!("║   BMP VM — Benchmark Report              ║");
    println!("║   Platform: vm-host (desktop Rust)       ║");
    println!("╚══════════════════════════════════════════╝");
    println!();
    println!("  Build: release mode (--release)");
    println!("  Warmup: 10 runs discarded before timing");
    println!();

    let iters = 1000u32;

    // Test 1: loop 100 — 9 instructions × 100 iters + 6 setup = 906
    bench("Loop x<100 (x=x+1)", LOOP_100_FIXED, iters, 906);

    // Test 2: arith 200 — 9 instructions × 67 iters (200/3) + 6 = 609
    // (200 / 3 = 66.6 → 67 loop iters until x >= 200)
    bench("Arith x<200 (x=x+3)", ARITH_200, iters, 609);

    // Test 3: mem access — 13 instructions × 50 iters + 10 setup = 660
    bench("Memory x<50 two-var", MEM_ACCESS, iters, 660);

    // Native comparison
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  OVERHEAD COMPARISON");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Get VM avg for loop 100
    let vm_avg = {
        for _ in 0..10 { let h = BenchHost; let mut v = VM::new(LOOP_100_FIXED, h); v.run(); }
        let mut sum = 0u64;
        for _ in 0..iters {
            let h = BenchHost;
            let mut v = VM::new(LOOP_100_FIXED, h);
            let t0 = Instant::now();
            v.run();
            sum += t0.elapsed().as_nanos() as u64;
        }
        sum / iters as u64
    };

    fn native_loop_100() -> i32 {
        let mut x = 0i32;
        while x < 100 { x += 1; }
        x
    }
    for _ in 0..10 { native_loop_100(); }
    let mut native_sum = 0u64;
    for _ in 0..iters {
        let t0 = Instant::now();
        native_loop_100();
        native_sum += t0.elapsed().as_nanos() as u64;
    }
    let native_avg = native_sum / iters as u64;

    let overhead_x = if native_avg > 0 { vm_avg as f64 / native_avg as f64 } else { 0.0 };

    println!("  VM avg (loop 100)     : {} ns", vm_avg);
    println!("  Native avg (loop 100) : {} ns", native_avg);
    println!("  VM overhead           : {:.1}× slower than native Rust", overhead_x);
    println!();
    println!("  Bytecode sizes:");
    println!("  Loop 100   : {} bytes", LOOP_100_FIXED.len());
    println!("  Arith 200  : {} bytes", ARITH_200.len());
    println!("  Mem access : {} bytes", MEM_ACCESS.len());
    println!();
    println!("  VM static memory footprint:");
    println!("  stack[]    : {} bytes  (32 × i32)", 32 * 4);
    println!("  memory[]   : {} bytes  (32 × i32)", 32 * 4);
    println!("  call_stack : {} bytes  (32 × usize)", 32 * 8);
    println!("  VM struct  : ~{} bytes total on stack", 32*4 + 32*4 + 32*8 + 8 + 8 + 8);
    println!();
    println!("╔══════════════════════════════════════════╗");
    println!("║   Done. Copy above numbers to report.    ║");
    println!("╚══════════════════════════════════════════╝");
}