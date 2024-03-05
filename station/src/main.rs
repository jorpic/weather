#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m_rt::entry;
use stm32f1xx_hal::{prelude::*, pac, i2c, usb};

use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use embedded_drivers::bmp180::BMP180;
use lsm303dlhc::Lsm303dlhc;

static LOOP_FLAG: AtomicBool = AtomicBool::new(false);

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello!");

    // Get access to peripherals
    let mut core = cortex_m::Peripherals::take().unwrap();
    let pac = pac::Peripherals::take().unwrap();

    {   // enable debug cycle counter (required by I2C)
        core.DWT.enable_cycle_counter();
        core.DCB.enable_trace(); // FIXME: what is this?
    }

    let mut flash = pac.FLASH.constrain();
    let mut afio = pac.AFIO.constrain();
    let rcc = pac.RCC.constrain();

    // Setup ADC clock
    let clocks = rcc.cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    let mut delay = pac.TIM1.delay_ms(&clocks);

    let gpioa = pac.GPIOA.split();
    let mut gpiob = pac.GPIOB.split();
    let mut gpioc = pac.GPIOC.split();

    // LED
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // USB CDC-ACM
    assert!(clocks.usbclk_valid());

    let usb_bus = usb::UsbBus::new(usb::Peripheral {
        usb: pac.USB,
        pin_dm: gpioa.pa11,
        pin_dp: gpioa.pa12,
    });

    let mut usb_serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Weather station")
        .serial_number("v0.0")
        .device_class(USB_CLASS_CDC)
        .self_powered(true)
        .build();

    // I2C
    let i2c = {
        let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
        let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

        let i2c = i2c::BlockingI2c::i2c1(
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
        );
        shared_bus::BusManagerSimple::new(i2c)
    };

    rprintln!("Entering loop");

    let mut loop_count = 0usize;
    loop {
        usb_dev.poll(&mut [&mut usb_serial]);

        if !LOOP_FLAG.swap(false, Ordering::Relaxed) {
            continue;
        }

        let mut lsm303 = Lsm303dlhc::new(i2c.acquire_i2c()).unwrap();
        let m = lsm303.mag().unwrap();
        rprintln!("m: {} {} {}", m.x, m.y, m.z);
        rprintln!("t: {}", lsm303.temp().unwrap());

        let mut bmp180 = BMP180::new(i2c.acquire_i2c());
        bmp180.init();
        rprintln!("t: {}", bmp180.get_temperature(&mut delay));
        rprintln!("p: {}", bmp180.get_pressure(&mut delay));

        let _ = usb_serial.write(b"hello world!\r\n");
        led.set_low();
        delay.delay_ms(500u16);
        led.set_high();

        if loop_count >= 10 {
            rprintln!("TODO: Send to server");
            loop_count = 0;
        }
    }
}
