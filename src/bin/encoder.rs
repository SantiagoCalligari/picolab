#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::rotary_encoder::{Direction, PioEncoder, PioEncoderProgram};
use embassy_rp::peripherals::PIO0;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!( struct Irqs { 
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});


#[embassy_executor::task]
async fn encoder_control( mut encoder: PioEncoder<'static, PIO0, 0 >){
    let mut count = 0;
    loop {
        info!("Count: {}", count);
        count += match encoder.read().await {
            Direction::Clockwise => 1,
            Direction::CounterClockwise => -1,
        };
    }
}


#[embassy_executor::main]
async fn main(spawner: Spawner){
    let p = embassy_rp::init(Default::default());
    let Pio { mut common, sm0, ..} = Pio::new(p.PIO0, Irqs); 
    let prg = PioEncoderProgram::new(&mut common);
    let encoder = PioEncoder::new( &mut common, sm0, p.PIN_4, p.PIN_5, &prg );
    spawner.spawn(encoder_control(encoder));

}
