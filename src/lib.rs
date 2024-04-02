#![no_std]

pub fn cycles() -> u64 {
    esp_hal::xtensa_lx::timer::get_cycle_count() as u64
}
