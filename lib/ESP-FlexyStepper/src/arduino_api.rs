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
/*
#[no_mangle]
pub extern "C" fn vTaskDelete(h: TaskHandle_t) {}

#[no_mangle]
pub extern "C" fn uxTaskGetStackHighWaterMark(h: TaskHandle_t) -> u32 {
    0
}
*/

#[no_mangle]
pub extern "C" fn configASSERT(_h: TaskHandle_t) {}

#[no_mangle]
pub extern "C" fn pinMode(_pin: u8, _mode: u8) {}

#[no_mangle]
pub extern "C" fn digitalWrite(_pin: u8, _value: u8) {}

#[no_mangle]
pub extern "C" fn digitalRead(_pin: u8) -> u8 {
    0
}

#[no_mangle]
pub extern "C" fn delay(_ms: u32) {}

#[no_mangle]
pub extern "C" fn millis() -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn micros() -> u32 {
    0
}
