#![no_std]
#![no_main]

use defmt::{info, todo, error};
use embassy_executor::Spawner;
use embassy_rp::Peri;
use embassy_rp::gpio;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::I2c;
use embassy_rp::i2c::{SclPin, SdaPin, self, Async};
use embassy_rp::peripherals::I2C0;
use embassy_rp::peripherals::{PIN_0, PIN_1};
use embassy_time::Duration;
use embassy_time::Instant;
use embassy_time::{Ticker, Timer};
use gpio::{AnyPin, Level, Output};
use ssd1306::I2CDisplayInterface;
use ssd1306::{Ssd1306, prelude::*, mode::BufferedGraphicsMode, };
use ssd1306::size::DisplaySize128x64;
use {defmt_rtt as _, panic_probe as _};
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
use ratatui::{
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};
use core::ptr::addr_of_mut;

use alloc::boxed::Box;
bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<embassy_rp::peripherals::I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<embassy_rp::peripherals::I2C1>;
});

pub type Oled96<'d> = Ssd1306<
    I2CInterface<I2c<'d, embassy_rp::peripherals::I2C0, Async>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

use alloc::format;
extern crate alloc;

use core::{i128,mem::MaybeUninit};
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

const HEAP_SIZE: usize = 32768;
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

#[embassy_executor::task]
async fn print_oled(i2c: Peri<'static, I2C0>, sda: Peri<'static, PIN_0>, scl: Peri<'static, PIN_1>){
    let mut config:i2c::Config = i2c::Config::default();
    config.frequency = 1_000_000;

    let bus = I2c::new_async(i2c, scl, sda, Irqs, config);
    let interface = I2CDisplayInterface::new(bus);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, ssd1306::prelude::DisplayRotation::Rotate0).into_buffered_graphics_mode();

    display.init().unwrap_or_else(|e| {
        error!(
            "Fallo al inicializar el display oLED: {:?}",
            defmt::Debug2Format(&e)
        );
        panic!("Revisar cableado");
    });
    info!("Display Inicializado");
    
    let backend_config = EmbeddedBackendConfig {
        flush_callback: Box::new(|d: &mut Oled96| d.flush().unwrap()),
        ..Default::default()
    };
     info!("Creando el backend de MouseFood");
    let backend = EmbeddedBackend::new(&mut display, backend_config);
    info!("Creando la Terminal de MouseFood");
    let mut terminal = Terminal::new(backend).unwrap_or_else(|e| {
        error!(
            "Hubo un error creando la terminal, {:?} ",
            defmt::Debug2Format(&e)
        );
        panic!("Revisar cableado");
    });
    let mut counter: i128 = 2;
    loop {
        let start = Instant::now();
        terminal.draw(|f| draw(f, counter)).unwrap();
        let elapsed = start.elapsed();
        counter = counter + 1;
        info!("Tiempo de escritura: {} ms", elapsed.as_millis());

    }
}

#[embassy_executor::task]
async fn blink(pin: Peri<'static, AnyPin>) {
    let mut led = Output::new(pin, Level::Low);
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        led.toggle();
        ticker.next().await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    unsafe {
        HEAP.init(addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE);
    }

    info!("Heap inicializado con {} bytes", HEAP_SIZE);

    spawner.spawn(blink(p.PIN_25.into())).unwrap();
    spawner.spawn(print_oled(p.I2C0.into(), p.PIN_0, p.PIN_1)).unwrap();

    let mut counter = 0;
    loop {
        counter = counter + 1;
        Timer::after_millis(500).await;
    }
}

fn draw(frame: &mut Frame, counter: i128) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .title(" Pico ");

    let paragraph = Paragraph::new(format!("Hola {}", counter)).block(block);

    frame.render_widget(paragraph, frame.area());
}
