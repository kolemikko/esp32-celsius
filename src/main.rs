#![no_std]
#![no_main]

use core::fmt::Write;
use dht11::Dht11;
use esp32c3_hal::{
    clock::ClockControl, gpio::IO, pac::Peripherals, prelude::*, system::SystemExt,
    timer::TimerGroup, Delay, Rtc, Serial,
};
use esp_backtrace as _;
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut serial0 = Serial::new(peripherals.UART0);

    // Disable the watchdog timers. For the ESP32-C3, this includes the Super WDT,
    // the RTC WDT, and the TIMG WDTs.
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio5.into_push_pull_output();

    led.set_high().unwrap();

    let mut delay = Delay::new(&clocks);

    let pin = io.pins.gpio0.into_open_drain_output();
    let mut dht11 = Dht11::new(pin);

    loop {
        led.toggle().unwrap();
        match dht11.perform_measurement(&mut delay) {
            Ok(meas) => {
                writeln!(serial0, "{}", meas.temperature / 10).unwrap();
            }
            Err(_) => {}
        };

        delay.delay_ms(1000u32);
    }
}
