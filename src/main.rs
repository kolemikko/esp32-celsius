use embedded_hal::digital::v2::OutputPin;
use esp_idf_hal::peripherals::Peripherals;
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let mut led = pins.gpio19.into_output().unwrap();

    loop {
        led.set_high().unwrap();
        thread::sleep(Duration::from_secs(2));
        led.set_low().unwrap();
    }

    // Ok(())
}
