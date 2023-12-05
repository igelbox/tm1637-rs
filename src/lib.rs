#![deny(warnings)]
#![no_std]

extern crate embedded_hal as hal;

use hal::blocking::delay::DelayUs;
use hal::digital::v2::{InputPin, OutputPin};

#[derive(Debug)]
pub enum Error<E> {
    Ack,
    IO(E),
}

impl<E> From<E> for Error<E> {
    fn from(err: E) -> Error<E> {
        Error::IO(err)
    }
}

type Res<E> = Result<(), Error<E>>;

pub struct TM1637<'a, CLK, DIO, D> {
    clk: &'a mut CLK,
    dio: &'a mut DIO,
    delay: &'a mut D,
}

enum Bit {
    ZERO,
    ONE,
}

impl<'a, CLK, DIO, D, E> TM1637<'a, CLK, DIO, D>
where
    CLK: OutputPin<Error = E>,
    DIO: InputPin<Error = E> + OutputPin<Error = E>,
    D: DelayUs<u16>,
{
    pub fn new(clk: &'a mut CLK, dio: &'a mut DIO, delay: &'a mut D) -> Self {
        Self { clk, dio, delay }
    }

    pub fn init(&mut self) -> Res<E> {
        self.start()?;
        self.send(ADDRESS_AUTO_INCREMENT_1_MODE)?;
        self.stop()?;

        Ok(())
    }

    pub fn clear(&mut self) -> Res<E> {
        self.print_raw_iter(0, core::iter::repeat(0).take(4))
    }

    pub fn print_raw(&mut self, address: u8, bytes: &[u8]) -> Res<E> {
        self.print_raw_iter(address, bytes.iter().map(|b| *b))
    }

    pub fn print_hex(&mut self, address: u8, digits: &[u8]) -> Res<E> {
        self.print_raw_iter(
            address,
            digits.iter().map(|digit| DIGITS[(digit & 0xf) as usize]),
        )
    }

    pub fn print_raw_iter<Iter: Iterator<Item = u8>>(
        &mut self,
        address: u8,
        bytes: Iter,
    ) -> Res<E> {
        self.start()?;
        self.send(ADDRESS_COMMAND_BITS | (address & ADDRESS_COMMAND_MASK))?;
        for byte in bytes {
            self.send(byte)?;
        }
        self.stop()?;
        Ok(())
    }

    pub fn set_brightness(&mut self, level: u8) -> Res<E> {
        self.start()?;
        self.send(DISPLAY_CONTROL_BRIGHTNESS_BITS | (level & DISPLAY_CONTROL_BRIGHTNESS_MASK))?;
        self.stop()?;

        Ok(())
    }

    fn send(&mut self, byte: u8) -> Res<E> {
        let mut rest = byte;
        for _ in 0..8 {
            let bit = if rest & 1 != 0 { Bit::ONE } else { Bit::ZERO };
            self.send_bit_and_delay(bit)?;
            rest = rest >> 1;
        }

        // Wait for the ACK
        self.send_bit_and_delay(Bit::ONE)?;
        for _ in 0..255 {
            if self.dio.is_low()? {
                return Ok(());
            }
            self.delay();
            self.delay();
        }

        Err(Error::Ack)
    }

    fn start(&mut self) -> Res<E> {
        self.send_bit_and_delay(Bit::ONE)?;
        self.dio.set_low()?;

        Ok(())
    }

    fn stop(&mut self) -> Res<E> {
        self.send_bit_and_delay(Bit::ZERO)?;
        self.dio.set_high()?;
        self.delay();
        self.delay();

        Ok(())
    }

    fn send_bit_and_delay(&mut self, value: Bit) -> Res<E> {
        self.clk.set_low()?;
        if let Bit::ONE = value {
            self.dio.set_high()?;
        } else {
            self.dio.set_low()?;
        }
        self.delay();
        self.clk.set_high()?;
        self.delay();

        Ok(())
    }

    fn delay(&mut self) {
        self.delay.delay_us(DELAY_USECS);
    }
}

const MAX_FREQ_KHZ: u16 = 500;
const USECS_IN_MSEC: u16 = 1_000;
const DELAY_USECS: u16 = USECS_IN_MSEC.div_ceil(MAX_FREQ_KHZ * 2);

const ADDRESS_AUTO_INCREMENT_1_MODE: u8 = 0x40;

const ADDRESS_COMMAND_BITS: u8 = 0xc0;
const ADDRESS_COMMAND_MASK: u8 = 0x0f;

const DISPLAY_CONTROL_BRIGHTNESS_BITS: u8 = 0x88;
const DISPLAY_CONTROL_BRIGHTNESS_MASK: u8 = 0x07;

const DIGITS: [u8; 16] = [
    0x3f, 0x06, 0x5b, 0x4f, //
    0x66, 0x6d, 0x7d, 0x07, //
    0x7f, 0x6f, 0x77, 0x7c, //
    0x39, 0x5e, 0x79, 0x71, //
];
