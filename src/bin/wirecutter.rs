#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Ticker};
use embedded_hal_02::digital::v2::OutputPin;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    cpu_control::{CpuControl, Stack},
    embassy::{self, executor::Executor},
    get_core,
    gpio::{GpioPin, Output, PushPull, IO},
    mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, PeripheralClockConfig, MCPWM},
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
};
use esp_println::println;
use static_cell::make_static;

const MUL: u16 = 100;

static mut APP_CORE_STACK: Stack<8192> = Stack::new();

/// Waits for a message that contains a duration, then flashes a led for that
/// duration of time.
#[embassy_executor::task]
async fn control_led(
    mut led: GpioPin<Output<PushPull>, 13>,
    _control: &'static Signal<CriticalSectionRawMutex, bool>,
) {
    println!("Starting control_led() on core {}", get_core() as usize);
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        ticker.next().await;
        esp_println::println!("LED on");
        led.set_low().unwrap();
        ticker.next().await;
        esp_println::println!("LED off");
        led.set_high().unwrap();
    }
}

#[main]
async fn main(_spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    let mut cpu_control = CpuControl::new(system.cpu_control);

    let led_ctrl_signal = &*make_static!(Signal::new());

    let led = io.pins.gpio13.into_push_pull_output();

    let _guard = cpu_control
        .start_app_core(unsafe { &mut APP_CORE_STACK }, move || {
            let executor = make_static!(Executor::new());
            executor.run(|spawner| {
                spawner.spawn(control_led(led, led_ctrl_signal)).ok();
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

    // pin will be high 50% of the time
    //pwm_pin.set_timestamp(50);

    println!("Starting pwm control on core {}", get_core() as usize);
    let mut ticker = Ticker::every(Duration::from_secs(3));
    loop {
        ticker.next().await;
        pwm_pin1.set_timestamp((100 - 3) * MUL); // inv 0%
        ticker.next().await;
        pwm_pin2.set_timestamp((10 + 3) * MUL); // 10%
        ticker.next().await;
        pwm_pin1.set_timestamp((90 - 3) * MUL); // inv 10%
        ticker.next().await;
        pwm_pin2.set_timestamp((0 + 3) * MUL); // 0%
    }
}
