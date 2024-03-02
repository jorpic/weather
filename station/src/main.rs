#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{prelude::*, pac, i2c, timer::Timer};

use embedded_drivers::bmp180::BMP180;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello!");

    // Get access to peripherals
    let mut core = cortex_m::Peripherals::take().unwrap();
    let pac = pac::Peripherals::take().unwrap();

    {   // enable debug cycle counter (required by I2C)
        core.DWT.enable_cycle_counter();
        // FIXME: ??
        core.DCB.enable_trace();
    }

    let mut flash = pac.FLASH.constrain();
    let mut afio = pac.AFIO.constrain();
    let rcc = pac.RCC.constrain();

    // Setup ADC clock
    let clocks = rcc.cfgr.adcclk(2.MHz()).freeze(&mut flash.acr);

    let mut delay = pac.TIM1.delay_ms(&clocks);

    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(core.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();

    let mut gpioa = pac.GPIOA.split();
    let mut gpiob = pac.GPIOB.split();

    // I2C
    let i2c = {
        let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
        let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

        i2c::BlockingI2c::i2c1(
            pac.I2C1,
            (scl, sda),
            &mut afio.mapr,
            i2c::Mode::Fast {
                frequency: 100.kHz(),
                duty_cycle: i2c::DutyCycle::Ratio16to9,
            },
            clocks,
            1000, // start_timeout_us
            10,   // start_retries
            1000, // addr_timeout_us
            1000, // data_timeout_us
        )
    };

    let mut bmp180 = BMP180::new(i2c);
    bmp180.init();

    loop {
        rprintln!("t: {}", bmp180.get_temperature(&mut delay));
        rprintln!("p: {}", bmp180.get_pressure(&mut delay));
        block!(timer.wait()).unwrap();
    }
}
