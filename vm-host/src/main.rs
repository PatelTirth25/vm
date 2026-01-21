use std::{env, fs};

use vm_core::{Host, VmFlags, VM};

struct HostStd;

struct HostUart;

impl Host for HostUart {
    fn print(&self, _value: i32) {
        // write to UART / RTT / semihosting
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

    fn report_flags(&self, flags: VmFlags) {
        if flags.any_error() {
            println!("VM FLAGS = {:?}", flags);
        }
    }
}

fn main() {
    // Program:
    // x = 10
    // x = x + 1
    // print(x)

    // let bytecode = [
    //     0x01, 10,    // PUSH 10
    //     0x21, 0,     // STORE 0
    //     0x20, 0,     // LOAD 0
    //     0x01, 1,     // PUSH 1
    //     0x02,        // ADD
    //     0x21, 0,     // STORE 0
    //     0x20, 0,     // LOAD 0
    //     0xFF,        // HALT
    // ];

    let args: Vec<String> = env::args().collect();

    // Ensure a file path argument was provided.
    if args.len() < 2 {
        eprintln!("Usage: cargo run -- <file_path>");
        return;
    }

    // The file path is the second argument (index 1).
    let file_path = &args[1];

    println!("Reading file: {}", file_path);

    // Read the file's content into a string.
    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");

    println!("File contents:\n{}", contents);

    // Parse the file content into bytecode
    let bytecode: Vec<u8> = contents
        .lines()
        .flat_map(|line| {
            // Remove comments and trim whitespace
            let cleaned = line.split(";").next().unwrap_or(line).trim();
            if cleaned.is_empty() {
                return Vec::new();
            }

            // Split by comma and parse each hex value
            cleaned
                .split(',')
                .filter_map(|part| {
                    let trimmed = part.trim();
                    if trimmed.is_empty() {
                        return None;
                    }
                    
                    // Parse as hex if it has 0x prefix, otherwise as decimal
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

    // Create and run the VM
    let mut vm = VM::new(&bytecode, HostStd);
    vm.run();
}
