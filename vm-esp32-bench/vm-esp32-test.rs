//! BMP VM Benchmark — vm-esp32 target
//! Measures: execution time in µs using esp_timer_get_time()
//! Results printed over UART (see in serial monitor)
//!
//! Flash with: cargo espflash flash --release --monitor
//! or:         espflash flash target/xtensa-esp32-none-elf/release/vm-esp32-bench

#![no_std]
#![no_main]

use core::cell::RefCell;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output},
    main,
    time,         // esp-hal 0.23 time::now() gives Instant
};
use esp_println::println;
use vm_core::{Host, VmFlags, VM};

// ─── Silent benchmark host ──────────────────────────────────────────────────
struct BenchHost;

impl Host for BenchHost {
    fn print(&self, _v: i32) {}
    fn native_call(&self, _id: u8, _arg: i32) -> i32 { 0 }
    fn report_flags(&self, _f: VmFlags) {}
}

// ─── Same bytecodes as vm-host-bench ───────────────────────────────────────

/// Loop: x=0, while x<100: x=x+1  (906 instructions total)
const LOOP_100: &[u8] = &[
    0x01, 0,    0x21, 0,    // x = 0
    0x01, 100,  0x21, 1,    // limit = 100
    // pc=8 loop
    0x20, 0,    0x20, 1,    0x31,       // Load x, Load limit, CmpLt
    0x41, 23,                           // JmpIfFalse → 23 (halt)
    0x20, 0,    0x01, 1,    0x02,       // Load x, Push 1, Add
    0x21, 0,                            // Store x
    0x40, 8,                            // Jmp → 8
    0xFF,                               // Halt
];

/// Arith: x=0, while x<200: x=x+3  (609 instructions)
const ARITH_200: &[u8] = &[
    0x01, 0,    0x21, 0,
    0x01, 200,  0x21, 1,
    0x20, 0,    0x20, 1,    0x31,
    0x41, 23,
    0x20, 0,    0x01, 3,    0x02,
    0x21, 0,
    0x40, 8,
    0xFF,
];

/// Mem: x=0, y=0, while x<50: x=x+1, y=y+2  (660 instructions)
const MEM_ACCESS: &[u8] = &[
    0x01, 0,   0x21, 0,   // x=0
    0x01, 0,   0x21, 1,   // y=0
    0x01, 50,  0x21, 2,   // limit=50
    // pc=12 loop
    0x20, 0,   0x20, 2,   0x31,
    0x41, 35,
    0x20, 0,   0x01, 1,   0x02,   0x21, 0,   // x=x+1
    0x20, 1,   0x01, 2,   0x02,   0x21, 1,   // y=y+2
    0x40, 12,
    0xFF,
];

// ─── Timer helper ───────────────────────────────────────────────────────────
// esp-hal 0.23: use esp_hal::time::now() → returns Instant
// .duration_since_epoch().to_micros() gives µs since boot

fn micros_now() -> u64 {
    esp_hal::time::now().duration_since_epoch().to_micros()
}

fn bench_esp32(name: &str, bytecode: &[u8], iters: u32, instr_count: u64) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  TEST: {}", name);

    // Warmup
    for _ in 0..5 {
        let mut vm = VM::new(bytecode, BenchHost);
        vm.run();
    }

    let mut total_us: u64 = 0;
    let mut min_us: u64 = u64::MAX;
    let mut max_us: u64 = 0;

    for _ in 0..iters {
        let t0 = micros_now();
        let mut vm = VM::new(bytecode, BenchHost);
        vm.run();
        let dt = micros_now() - t0;

        total_us += dt;
        if dt < min_us { min_us = dt; }
        if dt > max_us { max_us = dt; }
    }

    let avg_us = total_us / iters as u64;

    // Throughput: instructions per second
    // avg_us → avg_ns = avg_us * 1000
    let throughput_mips = if avg_us > 0 {
        instr_count as f64 / (avg_us as f64 / 1_000_000.0) / 1_000_000.0
    } else {
        0.0
    };

    println!("  Bytecode size : {} bytes", bytecode.len());
    println!("  Runs          : {}", iters);
    println!("  Avg time      : {} µs", avg_us);
    println!("  Min time      : {} µs", min_us);
    println!("  Max time      : {} µs", max_us);
    println!("  Instructions  : {}", instr_count);
    // Throughput as integer KIPS to avoid float formatting issues on no_std
    let throughput_kips = if avg_us > 0 {
        (instr_count * 1_000_000) / avg_us / 1000
    } else {
        0
    };
    println!("  Throughput    : {} KIPS (kilo-instr/sec)", throughput_kips);
    println!();
}

#[main]
fn main() -> ! {
    let _peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    println!();
    println!("╔══════════════════════════════════════╗");
    println!("║  BMP VM — ESP32 Benchmark Report     ║");
    println!("║  Clock: 240 MHz (default esp32)      ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    delay.delay_millis(500u32); // let UART settle

    let iters = 200u32; // fewer iters on device, still statistically valid

    bench_esp32("Loop x<100 (x=x+1)",     LOOP_100,   iters, 906);
    bench_esp32("Arith x<200 (x=x+3)",    ARITH_200,  iters, 609);
    bench_esp32("Memory x<50 two-var",    MEM_ACCESS, iters, 660);

    // VM struct size (static cost — no heap)
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  VM STATIC MEMORY FOOTPRINT");
    println!("  stack[]    : {} bytes", 32 * 4);
    println!("  memory[]   : {} bytes", 32 * 4);
    println!("  call_stack : {} bytes", 32 * 4); // usize=4 on ESP32 (32-bit)
    println!("  VM struct  : ~{} bytes total", 32*4 + 32*4 + 32*4 + 12 + 4);
    println!();
    println!("  Done. Results above for IEEE report.");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    loop {
        delay.delay_millis(5000u32);
        println!("[bench] complete. reset to rerun.");
    }
}