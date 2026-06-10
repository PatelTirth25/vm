#![cfg_attr(target_arch = "xtensa", no_std)]
#![cfg_attr(target_arch = "xtensa", no_main)]

#[cfg(not(target_arch = "xtensa"))]
use core::cell::RefCell;
#[cfg(not(target_arch = "xtensa"))]
use std::{env, fs};

#[cfg(target_arch = "xtensa")]
use core::cell::RefCell;

#[cfg(target_arch = "xtensa")]
use esp_backtrace as _;
#[cfg(target_arch = "xtensa")]
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output},
    main,
};
#[cfg(target_arch = "xtensa")]
use esp_println::println;

use vm_core::{Host, VmFlags, VM};
#[cfg(not(target_arch = "xtensa"))]
use vm_native::GpioController;

#[cfg(target_arch = "xtensa")]
const LED_PIN: u8 = 4; // Using GPIO4 for external LED
#[cfg(target_arch = "xtensa")]
const DEMO_PROGRAM: &[u8] = &[
    0x01, LED_PIN, // PUSH 4 (LED pin = GPIO4)
    0x50, 10, // CALL_NATIVE: LED ON (id=10)
    0x01, 100, // PUSH 100ms delay
    0x50, 20, // CALL_NATIVE: delay_ms (id=20)
    0x01, LED_PIN, // PUSH 4 (LED pin)
    0x50, 11, // CALL_NATIVE: LED OFF (id=11)
    0x01, 100, // PUSH 100ms delay
    0x50, 20,   // CALL_NATIVE: delay_ms (id=20)
    0xFF, // HALT
];

#[cfg(not(target_arch = "xtensa"))]
struct Esp32Host {
    gpio: RefCell<GpioController>,
}

#[cfg(not(target_arch = "xtensa"))]
impl Host for Esp32Host {
    fn print(&self, _value: i32) {}

    fn native_call(&self, id: u8, arg: i32) -> i32 {
        let pin = arg as u8;
        match id {
            10 => {
                let _ = self.gpio.borrow_mut().high(pin);
                0
            }
            11 => {
                let _ = self.gpio.borrow_mut().low(pin);
                0
            }
            12 => self.gpio.borrow().read(pin) as i32,
            13 => {
                let _ = self.gpio.borrow_mut().toggle(pin);
                0
            }
            _ => 0,
        }
    }

    fn report_flags(&self, flags: VmFlags) {
        if flags.any_error() {}
    }
}

#[cfg(target_arch = "xtensa")]
struct Esp32XtensaHost<'led, 'pin, 'delay> {
    led: &'led RefCell<Output<'pin>>,
    delay: &'delay Delay,
}

#[cfg(target_arch = "xtensa")]
impl Host for Esp32XtensaHost<'_, '_, '_> {
    fn print(&self, value: i32) {
        println!("VM OUTPUT = {}", value);
    }

    fn native_call(&self, id: u8, arg: i32) -> i32 {
        let pin = arg as u8;
        if pin != LED_PIN && id != 20 {
            println!(
                "Ignoring GPIO{} request; demo host only exposes GPIO{}",
                pin, LED_PIN
            );
            return 0;
        }

        match id {
            10 => {
                println!("[NATIVE] LED ON");
                self.led.borrow_mut().set_high();
                0
            }
            11 => {
                println!("[NATIVE] LED OFF");
                self.led.borrow_mut().set_low();
                0
            }
            12 => self.led.borrow().is_set_high() as i32,
            13 => {
                println!("[NATIVE] LED TOGGLE");
                self.led.borrow_mut().toggle();
                0
            }
            20 => {
                // delay_ms
                println!("[NATIVE] delay {}ms", arg);
                self.delay.delay_millis(arg as u32);
                0
            }
            _ => {
                println!("[NATIVE] Unknown id: {}", id);
                0
            }
        }
    }

    fn report_flags(&self, flags: VmFlags) {
        if flags.any_error() {
            println!("VM FLAGS = {:?}", flags);
        }
    }
}

#[cfg(target_arch = "xtensa")]
fn run_program() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let led = RefCell::new(Output::new(peripherals.GPIO4, Level::Low));
    let delay = Delay::new();

    println!("Running LED blink demo on external LED via GPIO{}", LED_PIN);
    println!(
        "Connect LED positive leg to GPIO{}, LED negative leg to GND",
        LED_PIN
    );

    // Test LED manually first
    println!("[TEST] LED ON");
    led.borrow_mut().set_high();
    delay.delay_millis(500);
    println!("[TEST] LED OFF");
    led.borrow_mut().set_low();
    delay.delay_millis(500);
    println!("[TEST] Manual blink done, starting VM...");

    loop {
        println!("[VM] Starting bytecode...");
        {
            let host = Esp32XtensaHost {
                led: &led,
                delay: &delay,
            };
            let mut vm = VM::new(DEMO_PROGRAM, host);
            vm.run();
        }
        println!("[VM] Bytecode done, waiting 500ms...");
        delay.delay_millis(500);
    }
}

#[cfg(not(target_arch = "xtensa"))]
fn run_program(bytecode: &[u8]) {
    let host = Esp32Host {
        gpio: RefCell::new(GpioController::new()),
    };

    let mut vm = VM::new(bytecode, host);
    vm.run();
}

#[cfg(target_arch = "xtensa")]
#[main]
fn main() -> ! {
    run_program()
}

#[cfg(not(target_arch = "xtensa"))]
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run -- <file_path>");
        return;
    }

    let contents = fs::read_to_string(&args[1]).expect("Failed to read bytecode file");

    let bytecode: Vec<u8> = contents
        .lines()
        .flat_map(|line| {
            let cleaned = line.split(';').next().unwrap_or(line).trim();
            if cleaned.is_empty() {
                return Vec::new();
            }
            cleaned
                .split(',')
                .filter_map(|part| {
                    let trimmed = part.trim();
                    if trimmed.is_empty() {
                        return None;
                    }
                    if trimmed.starts_with("0x") {
                        let hex_str = trimmed.trim_start_matches("0x");
                        u8::from_str_radix(hex_str, 16).ok()
                    } else {
                        trimmed.parse::<u8>().ok()
                    }
                })
                .collect::<Vec<u8>>()
        })
        .collect();

    run_program(&bytecode);
}
