use vm_core::{Host, VM, VmFlags};

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

    let bytecode = [
        0x01, 10,    // PUSH 10
        0x21, 0,     // STORE 0
        0x20, 0,     // LOAD 0
        0x01, 1,     // PUSH 1
        0x02,        // ADD
        0x21, 0,     // STORE 0
        0x20, 0,     // LOAD 0
        0xFF,        // HALT
    ];

    let mut vm = VM::new(&bytecode, HostStd);
    vm.run();
}
