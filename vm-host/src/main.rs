use vm_core::{Host, VM};

struct HostStd;

struct HostUart;

impl Host for HostUart {
    fn print(&self, value: i32) {
        // write to UART / RTT / semihosting
    }
}

impl Host for HostStd {
    fn print(&self, value: i32) {
        println!("VM OUTPUT = {}", value);
    }
}

fn main() {
    let bytecode = [0x01, 10, 0x01, 20, 0x02, 0xFF];

    let mut vm = VM::new(&bytecode, HostStd);
    vm.run();
}
