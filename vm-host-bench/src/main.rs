use std::{
    cell::Cell,
    hint::black_box,
    mem::size_of,
    time::{Duration, Instant},
};

use vm_core::{Host, VM, VmFlags};

const LOOP_100: &[u8] = &[
    0x01, 0, 0x21, 0, 0x01, 100, 0x21, 1, 0x20, 0, 0x20, 1, 0x31, 0x41, 24, 0x20, 0, 0x01, 1,
    0x02, 0x21, 0, 0x40, 8, 0xFF,
];
const LOOP_100_INSTR: u64 = 908;

const ARITH_200: &[u8] = &[
    0x01, 0, 0x21, 0, 0x01, 200, 0x21, 1, 0x20, 0, 0x20, 1, 0x31, 0x41, 24, 0x20, 0, 0x01, 3,
    0x02, 0x21, 0, 0x40, 8, 0xFF,
];
const ARITH_200_INSTR: u64 = 611;

const MEM_ACCESS: &[u8] = &[
    0x01, 0, 0x21, 0, 0x01, 0, 0x21, 1, 0x01, 50, 0x21, 2, 0x20, 0, 0x20, 2, 0x31, 0x41, 35,
    0x20, 0, 0x01, 1, 0x02, 0x21, 0, 0x20, 1, 0x01, 2, 0x02, 0x21, 1, 0x40, 12, 0xFF,
];
const MEM_ACCESS_INSTR: u64 = 660;

struct SizeHost;

impl Host for SizeHost {
    fn print(&self, _value: i32) {}
    fn native_call(&self, _id: u8, _arg: i32) -> i32 {
        0
    }
    fn report_flags(&self, _flags: VmFlags) {}
}

struct RecordingHost<'a> {
    flags: &'a Cell<VmFlags>,
}

impl Host for RecordingHost<'_> {
    fn print(&self, _value: i32) {}
    fn native_call(&self, _id: u8, _arg: i32) -> i32 {
        0
    }
    fn report_flags(&self, flags: VmFlags) {
        self.flags.set(flags);
    }
}

fn run_vm_once(bytecode: &[u8]) -> Duration {
    let flags = Cell::new(VmFlags::new());
    let host = RecordingHost { flags: &flags };
    let mut vm = VM::new(black_box(bytecode), host);
    let start = Instant::now();
    vm.run();
    let elapsed = start.elapsed();
    let reported = flags.get();
    assert!(
        !reported.any_error(),
        "benchmark bytecode triggered VM flags: {:?}",
        reported
    );
    black_box(elapsed)
}

fn native_loop_100() -> i32 {
    let limit = black_box(100i32);
    let mut x = 0i32;
    while x < limit {
        x += 1;
    }
    black_box(x)
}

fn bench_program(name: &str, bytecode: &[u8], runs: u32, instruction_count: u64) -> u64 {
    for _ in 0..10 {
        let _ = run_vm_once(bytecode);
    }

    let mut samples = Vec::with_capacity(runs as usize);
    for _ in 0..runs {
        samples.push(run_vm_once(bytecode).as_nanos() as u64);
    }

    let total_ns: u64 = samples.iter().sum();
    let avg_ns = total_ns / runs as u64;
    let min_ns = *samples.iter().min().unwrap();
    let max_ns = *samples.iter().max().unwrap();
    let mean = avg_ns as f64;
    let variance = samples
        .iter()
        .map(|sample| {
            let diff = *sample as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / runs as f64;
    let stddev_ns = variance.sqrt();
    let throughput_mips = if avg_ns > 0 {
        instruction_count as f64 * 1_000.0 / avg_ns as f64
    } else {
        0.0
    };

    println!("----------------------------------------");
    println!("TEST: {name}");
    println!("Runs         : {runs}");
    println!("Bytecode     : {} bytes", bytecode.len());
    println!("Instructions : {instruction_count}");
    println!("Avg time     : {avg_ns} ns ({:.3} us)", avg_ns as f64 / 1_000.0);
    println!("Min time     : {min_ns} ns");
    println!("Max time     : {max_ns} ns");
    println!("Stddev       : {:.2} ns", stddev_ns);
    println!("Throughput   : {:.3} MIPS", throughput_mips);
    println!();

    avg_ns
}

fn bench_native_loop_avg(runs: u32) -> u64 {
    for _ in 0..10 {
        let _ = native_loop_100();
    }

    let mut total_ns = 0u64;
    for _ in 0..runs {
        let start = Instant::now();
        let result = native_loop_100();
        total_ns += black_box(start.elapsed().as_nanos() as u64);
        black_box(result);
    }
    total_ns / runs as u64
}

fn main() {
    let runs = 1_000u32;

    println!();
    println!("BMP VM host benchmark");
    println!("Build       : release");
    println!("Warmup      : 10 runs discarded");
    println!("Timed runs  : {runs}");
    println!();

    let loop_avg = bench_program("Loop x<100 (x=x+1)", LOOP_100, runs, LOOP_100_INSTR);
    let _ = bench_program("Arith x<200 (x=x+3)", ARITH_200, runs, ARITH_200_INSTR);
    let _ = bench_program("Memory x<50 two-var", MEM_ACCESS, runs, MEM_ACCESS_INSTR);

    let native_avg = bench_native_loop_avg(runs);
    let overhead_ratio = if native_avg > 0 {
        loop_avg as f64 / native_avg as f64
    } else {
        0.0
    };

    let vm_size = size_of::<VM<'static, SizeHost>>();
    println!("----------------------------------------");
    println!("Native comparison");
    println!("Native loop avg : {native_avg} ns");
    println!("VM loop avg     : {loop_avg} ns");
    println!("Overhead ratio  : {:.3}x", overhead_ratio);
    println!();
    println!("Static VM footprint");
    println!("VM<SizeHost>    : {vm_size} bytes");
    println!("stack[]         : {} bytes", 32 * size_of::<i32>());
    println!("memory[]        : {} bytes", 32 * size_of::<i32>());
    println!("call_stack[]    : {} bytes", 32 * size_of::<usize>());
}
