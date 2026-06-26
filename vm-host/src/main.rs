use std::{cell::RefCell, env, fs};

use vm_core::{Host, VmFlags, VM};
use vm_native::GpioController;

struct HostStd {
    gpio: RefCell<GpioController>,
}

struct HostUart;

impl Host for HostUart {
    fn print(&self, _value: i32) {
        // write to UART / RTT / semihosting
    }

    fn native_call(&self, _id: u8, _args: &[i32]) -> i32 {
        0
    }

    fn report_flags(&self, flags: VmFlags) {
        if flags.any_error() {
            // report flags to UART / RTT / semihosting
        }
    }
}

impl Host for HostStd {
    fn print(&self, value: i32) {
        println!("VM OUTPUT = {}", value);
    }

    fn print_char(&self, c: u8) {
        print!("{}", c as char);
    }

    fn native_call(&self, id: u8, args: &[i32]) -> i32 {
        match id {
            0 => {
                print!("NATIVE PRINT:");
                for a in args { print!(" {}", a); }
                println!();
                0
            }
            1 => {
                let arg = args.first().copied().unwrap_or(0);
                let result = arg * 2;
                println!("NATIVE DOUBLE: {} -> {}", arg, result);
                result
            }
            2 => {
                let arg = args.first().copied().unwrap_or(0);
                let result = arg * arg;
                println!("NATIVE SQUARE: {} -> {}", arg, result);
                result
            }
            3 => {
                let sum: i32 = args.iter().sum();
                println!("NATIVE SUM {:?} = {}", args, sum);
                sum
            }
            10 => {
                let pin = args.first().copied().unwrap_or(0) as u8;
                let mut gpio = self.gpio.borrow_mut();
                gpio.high(pin)
            }
            11 => {
                let pin = args.first().copied().unwrap_or(0) as u8;
                let mut gpio = self.gpio.borrow_mut();
                gpio.low(pin)
            }
            12 => {
                let pin = args.first().copied().unwrap_or(0) as u8;
                let gpio = self.gpio.borrow();
                gpio.read(pin)
            }
            13 => {
                let pin = args.first().copied().unwrap_or(0) as u8;
                let mut gpio = self.gpio.borrow_mut();
                gpio.toggle(pin)
            }
            _ => {
                println!("Unknown native function id: {}", id);
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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run -- <file_path>");
        return;
    }

    let file_path = &args[1];

    println!("Reading file: {}", file_path);

    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");

    println!("File contents:\n{}", contents);

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

    println!("Parsed bytecode: {:?}", bytecode);

    let host = HostStd {
        gpio: RefCell::new(GpioController::new()),
    };

    let mut vm = VM::new(&bytecode, host);
    vm.run();
}
