#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::Peri;
use embassy_rp::gpio;
use embassy_time::Duration;
use embassy_time::{Ticker, Timer};
use gpio::{AnyPin, Level, Output};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn blink(pin: Peri<'static, AnyPin>) {
    let mut led = Output::new(pin, Level::Low);
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        led.toggle();
        info!("LED set {} ", led.get_output_level());

        ticker.next().await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    spawner.spawn(blink(p.PIN_25.into())).unwrap();

    let mut counter = 0;
    loop {
        counter = counter + 1;
        info!("Ran {} times asynchronously", counter);
        Timer::after_millis(500).await;
    }
}
