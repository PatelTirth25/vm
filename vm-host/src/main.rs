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
    let bytecode = [0x01, 10, 0x01, 20, 0x02, 0xFF];

    let mut vm = VM::new(&bytecode, HostStd);
    vm.run();
}
