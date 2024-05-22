#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::{Hertz, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_init();

    info!("Hello World!");

    let mut led = Output::new(p.PB2, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}

fn embassy_init() -> embassy_stm32::Peripherals {
    use embassy_stm32::rcc::*;

    let mut config = embassy_stm32::Config::default();
    
    config.rcc.hse = Some(Hse {
        freq: Hertz(8_000_000),
        mode: HseMode::Bypass,
    });
    config.rcc.pll_src = PllSource::HSE;
    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL168,
        divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 168 / 2 = 168Mhz.
        divq: Some(PllQDiv::DIV7), // 8mhz / 4 * 168 / 7 = 48Mhz, for USB
        divr: None,
    });
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV4;
    config.rcc.apb2_pre = APBPrescaler::DIV2;
    config.rcc.sys = Sysclk::PLL1_P;

    config.rcc.mux.clk48sel = mux::Clk48sel::PLL1_Q;

    embassy_stm32::init(config)
}