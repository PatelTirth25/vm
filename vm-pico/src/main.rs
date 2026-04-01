#![cfg_attr(target_arch = "arm", no_std)]
#![cfg_attr(target_arch = "arm", no_main)]

#[cfg(not(target_arch = "arm"))]
use core::cell::RefCell;

#[cfg(target_arch = "arm")]
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[cfg(target_arch = "arm")]
use cortex_m::delay::Delay;
#[cfg(target_arch = "arm")]
use cortex_m_rt::entry;
#[cfg(target_arch = "arm")]
use embedded_hal::digital::OutputPin;
#[cfg(target_arch = "arm")]
use hal::pac;
#[cfg(target_arch = "arm")]
use panic_halt as _;
#[cfg(target_arch = "arm")]
use rp2040_hal as hal;
#[cfg(target_arch = "arm")]
use rp2040_hal::clocks::Clock;

#[cfg(not(target_arch = "arm"))]
use vm_core::{Host, VmFlags, VM};
#[cfg(not(target_arch = "arm"))]
use vm_native::GpioController;

#[cfg(target_arch = "arm")]
const XTAL_FREQ_HZ: u32 = 12_000_000;
#[cfg(not(target_arch = "arm"))]
const LED_PIN: u8 = 14;
#[cfg(not(target_arch = "arm"))]
const TOGGLE_PROGRAM: &[u8] = &[0x01, LED_PIN, 0x50, 13, 0xFF];

#[cfg(not(target_arch = "arm"))]
struct PicoHost {
    gpio: RefCell<GpioController>,
}

#[cfg(not(target_arch = "arm"))]
impl Host for PicoHost {
    fn print(&self, _value: i32) {
        // optional UART debug
    }

    fn native_call(&self, id: u8, arg: i32) -> i32 {
        let pin = arg as u8;
        match id {
            10 => {
                self.gpio.borrow_mut().high(pin);
                0
            }
            11 => {
                self.gpio.borrow_mut().low(pin);
                0
            }
            12 => self.gpio.borrow().read(pin),
            13 => {
                self.gpio.borrow_mut().toggle(pin);
                0
            }
            _ => 0,
        }
    }

    fn report_flags(&self, flags: VmFlags) {
        if flags.any_error() {}
    }
}

#[cfg(target_arch = "arm")]
fn run_program() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = cortex_m::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led = pins.gpio14.into_push_pull_output();

    loop {
        led.set_high().ok();
        delay.delay_ms(500);
        led.set_low().ok();
        delay.delay_ms(500);
    }
}

#[cfg(not(target_arch = "arm"))]
fn run_program() {
    let host = PicoHost {
        gpio: RefCell::new(GpioController::new()),
    };

    let mut vm = VM::new(TOGGLE_PROGRAM, host);

    vm.run();
}

#[cfg(target_arch = "arm")]
#[entry]
fn main() -> ! {
    run_program()
}

#[cfg(not(target_arch = "arm"))]
fn main() {
    run_program();
}
