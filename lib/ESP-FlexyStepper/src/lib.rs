#![no_std]

extern crate alloc;

mod arduino_api;
mod bindings;

use core::{convert::Infallible, sync::atomic::AtomicU32};

use alloc::{boxed::Box, collections::BTreeMap};
pub use bindings::ESP_FlexyStepper;
use embedded_hal::digital::OutputPin;

const NONE_PIN: Option<Box<(dyn embedded_hal::digital::OutputPin<Error = Infallible> + 'static)>> =
    None;

static mut CURRENT_CPU_SPEED_MHZ: AtomicU32 = AtomicU32::new(240);

static mut PINS: [Option<Box<dyn OutputPin<Error = core::convert::Infallible>>>; 40] =
    [NONE_PIN; 40];

pub fn set_current_cpu_speed(mhz: u32) {
    unsafe {
        CURRENT_CPU_SPEED_MHZ.store(mhz, core::sync::atomic::Ordering::Relaxed);
    }
}

pub fn register_gpio(gpios: BTreeMap<u8, Box<dyn OutputPin<Error = core::convert::Infallible>>>) {
    for (k, v) in gpios {
        unsafe {
            PINS[k as usize] = Some(v);
        }
    }
}
