#![deny(unsafe_code)]
#![no_std]
#![no_main]

extern crate panic_halt;

extern crate stm32f103xx_hal as hal;

use hal::delay::Delay;
use hal::prelude::*;
use hal::stm32f103xx::Peripherals;
use embedded_hal::blocking::delay::DelayUs;

use cortex_m_rt::entry;

extern crate tm1637;
use tm1637::{ TM1637 };

struct NoDelay {}
impl DelayUs<u16> for NoDelay {
    fn delay_us(&mut self, us: u16) {}
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let mut clk = gpiob.pb6.into_open_drain_output(&mut gpiob.crl);
    let mut dio = gpiob.pb7.into_open_drain_output(&mut gpiob.crl);

    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut delay = NoDelay {}; // remove this line to use the real HW delay
    let mut tm = TM1637::new(&mut clk, &mut dio);
    tm.init(&mut delay); // append `.unwrap()` to catch and handle exceptions in cost of extra ROM size
    tm.clear(&mut delay);

    loop {
        for i in 0..255 {
            tm.print_hex(0, &[i, i + 1], &mut delay);

            tm.print_raw(3, &[i], &mut delay);

            tm.set_brightness(i >> 5, &mut delay);
        }
    }
}
