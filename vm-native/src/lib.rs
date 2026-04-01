#![cfg_attr(not(feature = "std"), no_std)]

pub struct GpioController {
    pins: [PinState; 256],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PinState {
    Low,
    High,
    #[default]
    Floating,
}

impl GpioController {
    pub const fn new() -> Self {
        Self {
            pins: [PinState::Floating; 256],
        }
    }

    pub fn high(&mut self, pin: u8) -> i32 {
        self.pins[pin as usize] = PinState::High;
        #[cfg(feature = "std")]
        println!("[GPIO] Pin {} set HIGH (LED ON)", pin);
        0
    }

    pub fn low(&mut self, pin: u8) -> i32 {
        self.pins[pin as usize] = PinState::Low;
        #[cfg(feature = "std")]
        println!("[GPIO] Pin {} set LOW (LED OFF)", pin);
        0
    }

    pub fn read(&self, pin: u8) -> i32 {
        match self.pins[pin as usize] {
            PinState::High => 1,
            PinState::Low => 0,
            PinState::Floating => -1,
        }
    }

    pub fn toggle(&mut self, pin: u8) -> i32 {
        let current = self.pins[pin as usize];
        let new_state = match current {
            PinState::High => PinState::Low,
            PinState::Low => PinState::High,
            PinState::Floating => PinState::High,
        };
        self.pins[pin as usize] = new_state;

        #[cfg(feature = "std")]
        match new_state {
            PinState::High => println!("[GPIO] Pin {} toggled HIGH", pin),
            PinState::Low => println!("[GPIO] Pin {} toggled LOW", pin),
            PinState::Floating => println!("[GPIO] Pin {} toggled FLOATING", pin),
        }

        0
    }

    pub fn status(&self, pin: u8) -> &'static str {
        match self.pins[pin as usize] {
            PinState::High => "HIGH",
            PinState::Low => "LOW",
            PinState::Floating => "FLOATING",
        }
    }
}

impl Default for GpioController {
    fn default() -> Self {
        Self::new()
    }
}

pub fn native_gpio(id: u8, pin: u8, controller: &mut GpioController) -> i32 {
    match id {
        0 => controller.high(pin),
        1 => controller.low(pin),
        2 => controller.read(pin),
        3 => controller.toggle(pin),
        _ => {
            #[cfg(feature = "std")]
            println!("[GPIO] Unknown GPIO function: {}", id);
            -1
        }
    }
}
