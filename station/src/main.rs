#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{prelude::*, pac, timer::Timer};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello!");

    // Get access to peripherals
    let mut core = cortex_m::Peripherals::take().unwrap();
    let pac = pac::Peripherals::take().unwrap();

    {   // enable debug cycle counter
        core.DCB.enable_trace();
        core.DWT.enable_cycle_counter();
    }

    let mut flash = pac.FLASH.constrain();
    let rcc = pac.RCC.constrain();

    // Setup ADC clock
    let clocks = rcc.cfgr.adcclk(2.MHz()).freeze(&mut flash.acr);

    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(core.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();

    let mut gpiob = pac.GPIOB.split();
    let mut gpioc = pac.GPIOC.split();

    // Btn and LED
    let _btn = gpiob.pb12.into_pull_up_input(&mut gpiob.crh);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    loop {
        led.set_low();
        let time = pac::DWT::cycle_count();
        rprintln!("{}", time);
        block!(timer.wait()).unwrap();
        led.set_high();
    }
}
