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

pub struct TM1637<'a, CLK, DIO> {
    clk: &'a mut CLK,
    dio: &'a mut DIO,
}

enum Bit {
    ZERO,
    ONE,
}

impl<'a, CLK, DIO, E> TM1637<'a, CLK, DIO>
where
    CLK: OutputPin<Error = E>,
    DIO: InputPin<Error = E> + OutputPin<Error = E>,
{
    pub fn new(clk: &'a mut CLK, dio: &'a mut DIO) -> Self {
        Self { clk, dio }
    }

    pub fn init<D: DelayUs<u16>>(&mut self, d: &mut D) -> Res<E> {
        self.start(d)?;
        self.send(ADDRESS_AUTO_INCREMENT_1_MODE, d)?;
        self.stop(d)?;

        Ok(())
    }

    pub fn clear<D: DelayUs<u16>>(&mut self, d: &mut D) -> Res<E> {
        self.print_raw_iter(0, core::iter::repeat(0).take(4), d)
    }

    pub fn print_raw<D: DelayUs<u16>>(&mut self, address: u8, bytes: &[u8], d: &mut D) -> Res<E> {
        self.print_raw_iter(address, bytes.iter().map(|b| *b), d)
    }

    pub fn print_hex<D: DelayUs<u16>>(&mut self, address: u8, digits: &[u8], d: &mut D) -> Res<E> {
        self.print_raw_iter(
            address,
            digits.iter().map(|digit| DIGITS[(digit & 0xf) as usize]),
            d,
        )
    }

    pub fn print_raw_iter<Iter: Iterator<Item = u8>, D: DelayUs<u16>>(
        &mut self,
        address: u8,
        bytes: Iter,
        d: &mut D,
    ) -> Res<E> {
        self.start(d)?;
        self.send(ADDRESS_COMMAND_BITS | (address & ADDRESS_COMMAND_MASK), d)?;
        for byte in bytes {
            self.send(byte, d)?;
        }
        self.stop(d)?;
        Ok(())
    }

    pub fn set_brightness<D: DelayUs<u16>>(&mut self, level: u8, d: &mut D) -> Res<E> {
        self.start(d)?;
        self.send(DISPLAY_CONTROL_BRIGHTNESS_BITS | (level & DISPLAY_CONTROL_BRIGHTNESS_MASK), d)?;
        self.stop(d)?;

        Ok(())
    }

    fn send<D: DelayUs<u16>>(&mut self, byte: u8, d: &mut D) -> Res<E> {
        let mut rest = byte;
        for _ in 0..8 {
            let bit = if rest & 1 != 0 { Bit::ONE } else { Bit::ZERO };
            self.send_bit_and_delay(bit, d)?;
            rest = rest >> 1;
        }

        // Wait for the ACK
        self.send_bit_and_delay(Bit::ONE, d)?;
        for _ in 0..255 {
            if self.dio.is_low()? {
                return Ok(());
            }
            self.delay(d);
        }

        Err(Error::Ack)
    }

    fn start<D: DelayUs<u16>>(&mut self, d: &mut D) -> Res<E> {
        self.send_bit_and_delay(Bit::ONE, d)?;
        self.dio.set_low()?;

        Ok(())
    }

    fn stop<D: DelayUs<u16>>(&mut self, d: &mut D) -> Res<E> {
        self.send_bit_and_delay(Bit::ZERO, d)?;
        self.dio.set_high()?;
        self.delay(d);

        Ok(())
    }

    fn send_bit_and_delay<D: DelayUs<u16>>(&mut self, value: Bit, d: &mut D) -> Res<E> {
        self.clk.set_low()?;
        if let Bit::ONE = value {
            self.dio.set_high()?;
        } else {
            self.dio.set_low()?;
        }
        self.clk.set_high()?;
        self.delay(d);

        Ok(())
    }

    fn delay<D: DelayUs<u16>>(&mut self, d: &mut D) {
        d.delay_us(DELAY_USECS);
    }
}

const MAX_FREQ_KHZ: u16 = 500;
const USECS_IN_MSEC: u16 = 1_000;
const DELAY_USECS: u16 = USECS_IN_MSEC / MAX_FREQ_KHZ;

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
