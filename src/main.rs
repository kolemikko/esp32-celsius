#![no_std]
#![no_main]

use core::cell::RefCell;
use core::fmt::Write;
use critical_section::Mutex;
use dht11::Dht11;
use esp32c3_hal::{
    clock::ClockControl,
    gpio::{Gpio9, IO},
    gpio_types::{Event, Input, Pin, PullDown},
    interrupt,
    pac::{self, Peripherals},
    prelude::*,
    system::SystemExt,
    timer::TimerGroup,
    Delay, Rtc, Serial,
};
use esp_backtrace as _;
use riscv_rt::entry;

static BUTTON: Mutex<RefCell<Option<Gpio9<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));

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

    led.set_low().unwrap();

    let mut delay = Delay::new(&clocks);

    let pin = io.pins.gpio0.into_open_drain_output();
    let mut dht11 = Dht11::new(pin);

    let mut button = io.pins.gpio9.into_pull_down_input();
    button.listen(Event::FallingEdge);

    critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(button));

    interrupt::enable(pac::Interrupt::GPIO, interrupt::Priority::Priority3).unwrap();

    unsafe {
        riscv::interrupt::enable();
    }

    let mut check_temperature: bool = true;

    loop {
        if check_temperature {
            led.set_high().unwrap();
            match dht11.perform_measurement(&mut delay) {
                Ok(meas) => {
                    writeln!(serial0, "{}", meas.temperature / 10).unwrap();
                    led.set_low().unwrap();
                    check_temperature = false;
                }
                Err(_) => {}
            };
        }
        delay.delay_ms(1000u32);
    }
}

#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        esp_println::println!("GPIO interrupt");
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}
