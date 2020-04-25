# `TM1637`
> A platform agnostic driver to a LED-display powered by the TM1637 chip

## What works
- Displaying raw bits and HEX-digits
- Controlling brightness

## TODO

## Example
Here is a simple example for using the MAX7219 on a stm32f103xx device with stm32f103xx_hal:
```rust
#![deny(unsafe_code)]
#![no_std]

extern crate cortex_m;
extern crate stm32f103xx_hal as hal;

use hal::delay::Delay;
use hal::prelude::*;
use hal::stm32f103xx::Peripherals;

extern crate tm1637;
use tm1637::{TM1637};

fn main() {

    let dp = Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let mut clk = gpiob.pb6.into_open_drain_output(&mut gpiob.crl);
    let mut dio = gpiob.pb7.into_open_drain_output(&mut gpiob.crl);

    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut tm = TM1637::new(&mut clk, &mut dio, &mut delay);
    tm.init().unwrap();
    tm.clear().unwrap();
    loop {
        for i in 0..255 {
            tm.print_hex(0, &[i, i + 1]).unwrap();

            tm.print_raw(3, &[i]).unwrap();

            tm.set_brightness(i >> 5).unwrap();
        }
    }
}
```

## License
Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
