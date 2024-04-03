use alloc::boxed::Box;
use embedded_hal::digital::OutputPin;

use esp_println::println;

use super::bindings::{HIGH, LOW, OUTPUT};
use super::{CURRENT_CPU_SPEED_MHZ, PINS};

/*
use super::bindings::{TaskHandle_t, Task_t};
#[no_mangle]
pub extern "C" fn disableCore0WDT() {}

#[no_mangle]
pub extern "C" fn disableCore1WDT() {}

#[no_mangle]
pub extern "C" fn xTaskCreatePinnedToCore(
    _task: Task_t,
    _name: *const u8,
    _stack_size: u32,
    _param: *const core::ffi::c_void,
    _priority: u32,
    _task_handle: TaskHandle_t,
    _core_id: usize,
) {
}

#[no_mangle]
pub extern "C" fn vTaskDelete(h: TaskHandle_t) {}

#[no_mangle]
pub extern "C" fn uxTaskGetStackHighWaterMark(h: TaskHandle_t) -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn configASSERT(_h: TaskHandle_t) {}

#[no_mangle]
pub extern "C" fn digitalRead(_pin: u8) -> u8 {
    0
}
*/

fn find_registred_pin(
    pin: u8,
) -> &'static mut Box<dyn OutputPin<Error = core::convert::Infallible>> {
    unsafe { PINS[pin as usize].as_mut() }.expect("Pin not registered!")
}

#[no_mangle]
pub extern "C" fn pinMode(pin: u8, mode: u8) {
    if OUTPUT != mode {
        panic!("Invalid pin {} config!", pin)
    } else {
        println!("Pin {} -> OUTPUT", pin);
    }
}

#[no_mangle]
pub extern "C" fn digitalWrite(pin_id: u8, value: u8) {
    let pin = find_registred_pin(pin_id);
    match value {
        LOW => pin.set_low().unwrap(),
        HIGH => pin.set_high().unwrap(),
        _ => panic!("Invalid pin {} value ({})!", pin_id, value),
    }
}

#[no_mangle]
pub extern "C" fn delay(millis: u32) {
    unsafe {
        esp_hal::xtensa_lx::timer::delay(
            millis * 1_000 / CURRENT_CPU_SPEED_MHZ.load(core::sync::atomic::Ordering::Relaxed),
        );
    }
}

#[no_mangle]
pub extern "C" fn millis() -> u32 {
    // cycle_time_millis = 1_000 / CURRENT_CPU_SPEED

    unsafe {
        esp_hal::xtensa_lx::timer::get_cycle_count()
            / (CURRENT_CPU_SPEED_MHZ.load(core::sync::atomic::Ordering::Relaxed) * 1000)
    }
}

#[no_mangle]
pub extern "C" fn micros() -> u32 {
    // cycle_time_micros = 1_000_000 / CURRENT_CPU_SPEED_MHZ

    unsafe {
        esp_hal::xtensa_lx::timer::get_cycle_count()
            / CURRENT_CPU_SPEED_MHZ.load(core::sync::atomic::Ordering::Relaxed)
    }
}
