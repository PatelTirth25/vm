#![no_std]
#![no_main]

use core::{cell::Cell, hint::black_box, mem::size_of};

use esp_backtrace as _;
use esp_hal::{delay::Delay, main};
use esp_println::println;
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

fn micros_now() -> u64 {
    esp_hal::time::now().duration_since_epoch().to_micros()
}

fn run_vm_once(bytecode: &[u8]) -> u64 {
    let flags = Cell::new(VmFlags::new());
    let host = RecordingHost { flags: &flags };
    let mut vm = VM::new(black_box(bytecode), host);
    let start = micros_now();
    vm.run();
    let elapsed = micros_now() - start;
    let reported = flags.get();
    assert!(
        !reported.any_error(),
        "benchmark bytecode triggered VM flags: {:?}",
        reported
    );
    black_box(elapsed)
}

fn bench_esp32(name: &str, bytecode: &[u8], runs: u32, instruction_count: u64) {
    for _ in 0..5 {
        let _ = run_vm_once(bytecode);
    }

    let mut total_us = 0u64;
    let mut min_us = u64::MAX;
    let mut max_us = 0u64;

    for _ in 0..runs {
        let sample = run_vm_once(bytecode);
        total_us += sample;
        if sample < min_us {
            min_us = sample;
        }
        if sample > max_us {
            max_us = sample;
        }
    }

    let avg_us = total_us / runs as u64;
    let throughput_kips = if avg_us > 0 {
        (instruction_count * 1_000_000) / avg_us / 1_000
    } else {
        0
    };

    println!("----------------------------------------");
    println!("TEST: {name}");
    println!("Runs         : {runs}");
    println!("Bytecode     : {} bytes", bytecode.len());
    println!("Instructions : {instruction_count}");
    println!("Avg time     : {avg_us} us");
    println!("Min time     : {min_us} us");
    println!("Max time     : {max_us} us");
    println!("Throughput   : {throughput_kips} KIPS");
    println!();
}

#[main]
fn main() -> ! {
    let _peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();
    let runs = 200u32;

    delay.delay_millis(500u32);

    println!();
    println!("BMP VM ESP32 benchmark");
    println!("Clock        : 240 MHz default");
    println!("Warmup       : 5 runs discarded");
    println!("Timed runs   : {runs}");
    println!();

    bench_esp32("Loop x<100 (x=x+1)", LOOP_100, runs, LOOP_100_INSTR);
    bench_esp32("Arith x<200 (x=x+3)", ARITH_200, runs, ARITH_200_INSTR);
    bench_esp32("Memory x<50 two-var", MEM_ACCESS, runs, MEM_ACCESS_INSTR);

    println!("----------------------------------------");
    println!("Static VM footprint");
    println!("VM<SizeHost> : {} bytes", size_of::<VM<'static, SizeHost>>());
    println!("stack[]      : {} bytes", 32 * size_of::<i32>());
    println!("memory[]     : {} bytes", 32 * size_of::<i32>());
    println!("call_stack[] : {} bytes", 32 * size_of::<usize>());

    loop {
        delay.delay_millis(5_000u32);
        println!("[bench] complete. reset to rerun.");
    }
}
