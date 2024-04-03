#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use alloc::sync::Arc;
use core::mem::MaybeUninit;

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Ticker};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    cpu_control::{CpuControl, Stack},
    embassy::{self, executor::Executor},
    get_core,
    gpio::IO,
    mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, PeripheralClockConfig, MCPWM},
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
};
use esp_println::println;
use static_cell::make_static;

use esp_flexystepper_rs::ESP_FlexyStepper;

const MUL: u16 = 100;

static mut APP_CORE_STACK: Stack<8192> = Stack::new();

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

#[embassy_executor::task]
async fn process_movement(steppers: [Arc<Mutex<CriticalSectionRawMutex, ESP_FlexyStepper>>; 2]) {
    println!(
        "Starting process_movement() on core {}",
        get_core() as usize
    );

    loop {
        for stepper in steppers.iter() {
            let mut stepper = stepper.lock().await;
            unsafe {
                stepper.processMovement();
            }
        }
    }
}

fn make_stepper(
    step_pin: u8,
    dir_pin: u8,
    od: bool,
    max_speed: f32,
    ac: f32,
    dac: f32,
) -> Arc<Mutex<CriticalSectionRawMutex, ESP_FlexyStepper>> {
    let mut stepper = unsafe { ESP_FlexyStepper::new() };

    unsafe {
        stepper.connectToPins(step_pin, dir_pin, od);
        stepper.setSpeedInStepsPerSecond(max_speed);
        stepper.setAccelerationInStepsPerSecondPerSecond(ac);
        stepper.setDecelerationInStepsPerSecondPerSecond(dac);
    }

    Arc::new(Mutex::new(stepper))
}

#[main]
async fn main(_spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    init_heap();
    embassy::init(&clocks, timg0);

    let cpu_speed_mhz = clocks.cpu_clock.to_MHz();
    println!("CPU freq={} MHz", cpu_speed_mhz);

    esp_flexystepper_rs::set_current_cpu_speed(cpu_speed_mhz);
    esp_flexystepper_rs::register_gpio({
        use alloc::boxed::Box;
        use embedded_hal::digital::OutputPin;

        type PinT = Box<dyn OutputPin<Error = core::convert::Infallible>>;

        let mut map: alloc::collections::BTreeMap<_, PinT> = alloc::collections::BTreeMap::new();
        map.insert(17, Box::new(io.pins.gpio17.into_push_pull_output()));
        map.insert(18, Box::new(io.pins.gpio18.into_push_pull_output()));
        map.insert(15, Box::new(io.pins.gpio15.into_push_pull_output()));
        map.insert(16, Box::new(io.pins.gpio16.into_push_pull_output()));

        map
    });

    let mut cpu_control = CpuControl::new(system.cpu_control);

    let steppers = [
        make_stepper(17, 18, false, 10000.0, 8000.0, 8000.0), // X
        make_stepper(15, 16, false, 10000.0, 8000.0, 8000.0), // Y
    ];

    let c_steppers = steppers.clone();
    let _guard = cpu_control
        .start_app_core(unsafe { &mut APP_CORE_STACK }, move || {
            let executor = make_static!(Executor::new());
            executor.run(|spawner| {
                spawner.spawn(process_movement(c_steppers)).ok();
            });
        })
        .unwrap();

    let clock_cfg = PeripheralClockConfig::with_frequency(&clocks, 32u32.MHz()).unwrap();
    let mut mcpwm = MCPWM::new(peripherals.MCPWM0, clock_cfg);

    let pwm_pin1 = io.pins.gpio25;
    let pwm_pin2 = io.pins.gpio26;

    // connect operator0 to timer0
    mcpwm.operator0.set_timer(&mcpwm.timer0);
    // connect operator0 to pin
    let mut pwm_pin1 = mcpwm
        .operator0
        .with_pin_a(pwm_pin1, PwmPinConfig::UP_ACTIVE_HIGH);

    // connect operator1 to timer0
    let mut pwm_pin2 = mcpwm
        .operator1
        .with_pin_a(pwm_pin2, PwmPinConfig::UP_ACTIVE_HIGH);

    // start timer with timestamp values in the range of 0..=99 and a frequency of
    // 20 kHz
    let timer_clock_cfg = clock_cfg
        .timer_clock_with_frequency(100 * MUL, PwmWorkingMode::Increase, 100u32.Hz())
        .unwrap();
    mcpwm.timer0.start(timer_clock_cfg);

    println!("Starting pwm control on core {}", get_core() as usize);
    let mut ticker = Ticker::every(Duration::from_secs(3));
    loop {
        ticker.next().await;
        pwm_pin1.set_timestamp((100 - 3) * MUL); // inv 0%
        unsafe {
            steppers[0].lock().await.setTargetPositionRelativeInMillimeters(110.0);
            steppers[1].lock().await.setTargetPositionRelativeInMillimeters(90.0);
        }
        ticker.next().await;
        pwm_pin2.set_timestamp((10 + 3) * MUL); // 10%
        unsafe {
            steppers[0].lock().await.setTargetPositionRelativeInMillimeters(-90.0);
            steppers[1].lock().await.setTargetPositionRelativeInMillimeters(-100.0);
        }
        ticker.next().await;
        pwm_pin1.set_timestamp((90 - 3) * MUL); // inv 10%
        unsafe {
            steppers[0].lock().await.setTargetPositionRelativeInMillimeters(110.0);
            steppers[1].lock().await.setTargetPositionRelativeInMillimeters(90.0);
        }
        ticker.next().await;
        pwm_pin2.set_timestamp((0 + 3) * MUL); // 0%
        unsafe {
            steppers[0].lock().await.setTargetPositionRelativeInMillimeters(-90.0);
            steppers[1].lock().await.setTargetPositionRelativeInMillimeters(-100.0);
        }
    }
}
