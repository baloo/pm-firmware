#![feature(generator_trait)]
#![feature(generators)]
#![feature(async_closure)]
#![feature(never_type)]
#![no_std]
#![no_main]

use stm32f4xx_hal as hal;

use core::cell::UnsafeCell;
use core::ops::Generator;
use core::pin::Pin;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::serial::{Read, Write};
use embedded_hal::timer::CountDown;

use futures_util::future;

use crate::hal::gpio::GpioExt;
use crate::hal::interrupt;
use crate::hal::rcc::{Clocks, RccExt};
use crate::hal::serial::config::StopBits;
use crate::hal::serial::{self, IntrHandler, Serial};
use crate::hal::stm32;
use crate::hal::time::U32Ext;
use crate::hal::timer::{Event, Timer};

mod app;
mod utils;

mod executor;
use executor::Executor;

struct Platform {
    device: stm32::Peripherals,
    cortex: cortex_m::Peripherals,
}

use panic_semihosting as _;

impl Platform {
    fn initialize() -> Option<Self> {
        if let (Some(device), Some(cortex)) =
            (stm32::Peripherals::take(), cortex_m::Peripherals::take())
        {
            Some(Platform { device, cortex })
        } else {
            None
        }
    }

    fn enable_peripheral_clocks<F>(&mut self, f: F)
    where
        F: FnOnce(&stm32::rcc::RegisterBlock),
    {
        f(&self.device.RCC)
    }
}

#[entry]
fn main() -> ! {
    if let Some(mut platform) = Platform::initialize() {
        // TODO: after power on, the periphericals are not clock, one needs to enable them
        //       APB1/ APB2, see block diagram of mcu
        //       see reference manual
        //       AHB1 handles
        //          - bit 0 GPIOA
        //          - bit 25 ethernet mac en
        //          - bit 26 ethernet mac tx
        //          - bit 27 ethernet mac rx
        //          - bit 28 ethernet ptp en
        platform.enable_peripheral_clocks(|rcc| {
            rcc.apb2enr.write(|w| w.syscfgen().enabled());
            rcc.ahb1enr.write(|w| w.gpioaen().enabled());
        });
        let rcc = platform.device.RCC.constrain();
        let clocks = rcc
            .cfgr
            .hclk(120.mhz())
            .sysclk(120.mhz())
            .pclk1(42.mhz())
            .pclk2(42.mhz())
            .freeze();

        let gpioa = platform.device.GPIOA.split();

        let mut serial = Serial::uart4(
            platform.device.UART4,
            (
                gpioa.pa0.into_alternate_af8(), // TX
                gpioa.pa1.into_alternate_af8(), // RX
            ),
            serial::config::Config::default()
                .baudrate(9600.bps())
                .parity_none()
                .wordlength_8()
                .stopbits(StopBits::STOP1),
            clocks,
        )
        .unwrap();

        let (tx, rx) = serial.split();
        let all = app::run(rx, tx);

        struct RacyCell<T>(UnsafeCell<T>);
        impl<T> RacyCell<T> {
            const fn new(value: T) -> Self {
                RacyCell(UnsafeCell::new(value))
            }
            #[allow(clippy::mut_from_ref)]
            unsafe fn get_mut_unchecked(&self) -> &mut T {
                &mut *self.0.get()
            }
        }
        unsafe impl<T> Sync for RacyCell<T> {}
        static EXECUTOR: RacyCell<Executor> = RacyCell::new(Executor::new());
        let out = unsafe { EXECUTOR.get_mut_unchecked() }.block_on(all);
    }
    hprintln!("what?");

    loop {}
}

#[interrupt]
fn UART4() {
    IntrHandler::<stm32::UART4>::interrupt();
}
