#![cfg_attr(target_arch = "xtensa", no_std)]
#![cfg_attr(target_arch = "xtensa", no_main)]

#[cfg(not(target_arch = "xtensa"))]
use core::cell::RefCell;

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

const LED_PIN: u8 = 2;
const DEMO_PROGRAM: &[u8] = &[0x01, 0,
0x21, 0,
0x01, 2,
0x21, 1,
0x20, 0,
0x20, 1,
0x31,
0x41, 27,
0x20, 0,
0x01, 1,
0x02,
0x21, 0,
0x20, 0,
0x51,
0x40, 8,
0xFF,
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
struct Esp32XtensaHost<'led, 'pin> {
    led: &'led RefCell<Output<'pin>>,
}

#[cfg(target_arch = "xtensa")]
impl Host for Esp32XtensaHost<'_, '_> {
    fn print(&self, value: i32) {
        println!("VM OUTPUT = {}", value);
    }

    fn native_call(&self, id: u8, arg: i32) -> i32 {
        let pin = arg as u8;
        if pin != LED_PIN {
            println!(
                "Ignoring GPIO{} request; demo host only exposes GPIO{}",
                pin, LED_PIN
            );
            return 0;
        }

        match id {
            10 => {
                self.led.borrow_mut().set_high();
                0
            }
            11 => {
                self.led.borrow_mut().set_low();
                0
            }
            12 => self.led.borrow().is_set_high() as i32,
            13 => {
                self.led.borrow_mut().toggle();
                0
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

#[cfg(target_arch = "xtensa")]
fn run_program() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let led = RefCell::new(Output::new(peripherals.GPIO2, Level::Low));
    let delay = Delay::new();

    println!("Running embedded bytecode demo on GPIO{}", LED_PIN);

    loop {
        let host = Esp32XtensaHost { led: &led };
        let mut vm = VM::new(DEMO_PROGRAM, host);
        vm.run();
        delay.delay_millis(500);
    }
}

#[cfg(not(target_arch = "xtensa"))]
fn run_program() {
    let host = Esp32Host {
        gpio: RefCell::new(GpioController::new()),
    };

    let mut vm = VM::new(DEMO_PROGRAM, host);
    vm.run();
}

#[cfg(target_arch = "xtensa")]
#[main]
fn main() -> ! {
    run_program()
}

#[cfg(not(target_arch = "xtensa"))]
fn main() {
    run_program();
}
