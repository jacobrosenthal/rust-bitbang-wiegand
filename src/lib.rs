#![cfg_attr(not(test), no_std)]

use bit_field::BitField;
use core::convert::TryFrom;
use core::time::Duration;
use embedded_hal::{
    blocking::delay::DelayUs,
    digital::{InputPin, OutputPin},
    timer::CountDown,
};
use nb;

#[cfg(feature = "unstable")]
use futures::{Async, Poll, Stream};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    Parity,
    TimedOut,
    TimedOutUp(u32),
    TimedOutDown(u32),
    _Extensible,
}

pub struct WiegandInput<Data0, Data1>
where
    Data0: InputPin,
    Data1: InputPin,
{
    data0: Data0,
    data1: Data1,
}

pub struct WiegandOutput<Data0, Data1>
where
    Data0: OutputPin,
    Data1: OutputPin,
{
    data0: Data0,
    data1: Data1,
}

pub trait Write {
    fn write(&mut self, delay: &mut DelayUs<u32>, data: u32);
}

// is blocking 2.1ms * 26 = 54.6 ms
impl<Data0, Data1> WiegandOutput<Data0, Data1>
where
    Data0: OutputPin,
    Data1: OutputPin,
{
    pub fn new(data0: Data0, data1: Data1) -> Self {
        Self { data0, data1 }
    }

    pub fn write(&mut self, delay: &mut DelayUs<u32>, data: WiegandData) {
        let data_out: u32 = data.into();

        let mut mask = 0x2000000;

        for _bit in 0..26 {
            if data_out & mask == mask {
                self.data1.set_low();
                delay.delay_us(40);
                self.data1.set_high();
            } else {
                self.data0.set_low();
                delay.delay_us(40);
                self.data0.set_high();
            }
            mask >>= 1;
            delay.delay_us(2000);
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WiegandData {
    pub facility: u8,
    pub id: u16,
}

fn is_odd(data: u8) -> bool {
    (data % 2) != 0
}

fn count_ones(input: u32) -> u8 {
    let mut ones = 0;
    let mut data_out = input;
    for _bit in 0..31 {
        if data_out & 1 == 1 {
            ones = ones + 1;
        }
        data_out >>= 1;
    }
    ones
}

impl TryFrom<u32> for WiegandData {
    type Error = Error;

    fn try_from(item: u32) -> Result<Self, Error> {
        let id = item.get_bits(1..17) as u16;
        let facility = item.get_bits(17..25) as u8;

        let even_bit = item.get_bit(25);
        let odd_bit = item.get_bit(0);

        let bottom = item.get_bits(1..13);
        let top = item.get_bits(13..25);

        let valid =
            (is_odd(count_ones(top)) == even_bit) && (!is_odd(count_ones(bottom)) == odd_bit);

        if valid {
            Ok(Self { facility, id })
        } else {
            Err(Error::Parity)
        }
    }
}

impl From<WiegandData> for u32 {
    fn from(item: WiegandData) -> Self {
        let mut blah: u32 = 0;
        blah.set_bits(1..17, item.id.into());
        blah.set_bits(17..25, item.facility.into());

        let bottom = blah.get_bits(1..13);
        let top = blah.get_bits(13..25);

        //if bottom 12 non parity is even, then put a 1 at the bottom
        blah.set_bit(0, !is_odd(count_ones(bottom)));

        //if top 12 non parity is odd, then put a 1 at the top
        blah.set_bit(25, is_odd(count_ones(top)));

        blah
    }
}

impl<Data0, Data1> WiegandInput<Data0, Data1>
where
    Data0: InputPin,
    Data1: InputPin,
{
    pub fn new(data0: Data0, data1: Data1) -> Self {
        Self { data0, data1 }
    }
}

pub trait Read<T>
where
    T: CountDown,
{
    fn read(&mut self, timer: &mut T) -> nb::Result<WiegandData, Error>;
}

// might need +Cancel if this doesnt accept https://github.com/rust-embedded/embedded-hal/issues/106
// requires 10s of microsecond resolution
// even though is non blocking waiting to start, is blocking during reading 2.1ms * 26 = 54.6 ms
impl<Data0, Data1, T, U> Read<T> for WiegandInput<Data0, Data1>
where
    Data0: InputPin,
    Data1: InputPin,
    T: embedded_hal::timer::CountDown<Time = U>,
    U: From<Duration>,
{
    fn read(&mut self, timer: &mut T) -> nb::Result<WiegandData, Error> {
        while self.data0.is_high() && self.data1.is_high() {
            return Err(nb::Error::WouldBlock);
        }

        let mut data_in: u32 = 0;

        for bit in 0..26 {
            // first time through we already found the bit above
            if bit != 0 {
                // Ive seen ~2ms here so 3ms?
                timer.start(Duration::from_micros(4000));
                while self.data0.is_high() && self.data1.is_high() {
                    match timer.wait() {
                        Err(nb::Error::Other(_e)) => unreachable!(),
                        Err(nb::Error::WouldBlock) => {}
                        Ok(()) => return Err(nb::Error::Other(Error::TimedOutUp(data_in))),
                    }
                }
            }

            // shift first because we have room to spare on the left
            // and because we dont want to be shifted over after bit 26
            data_in <<= 1;

            if self.data1.is_low() {
                data_in |= 1;
            }

            // Ive seen ~100us here, so 1ms?
            timer.start(Duration::from_micros(1000));
            while self.data0.is_low() || self.data1.is_low() {
                match timer.wait() {
                    Err(nb::Error::Other(_e)) => unreachable!(),
                    Err(nb::Error::WouldBlock) => {}
                    Ok(()) => return Err(nb::Error::Other(Error::TimedOutDown(data_in))),
                }
            }
        }

        match WiegandData::try_from(data_in) {
            Err(e) => Err(nb::Error::Other(e)),
            Ok(v) => Ok(v),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn even_parity_success() {
        let data = WiegandData::try_from(0x37E0002);

        assert_eq!(
            data,
            Ok(WiegandData {
                facility: 0xBF,
                id: 0x01
            })
        );
    }

    #[test]
    fn even_parity_fail() {
        let data = WiegandData::try_from(0x17E0002);

        assert_eq!(data, Err(Error::Parity));
    }

    #[test]
    fn odd_parity_success() {
        let data = WiegandData::try_from(0x2511055);

        assert_eq!(
            data,
            Ok(WiegandData {
                facility: 0x28,
                id: 0x882A
            })
        );
    }
    #[test]
    fn odd_parity_fail() {
        let data = WiegandData::try_from(0x2511015);
        assert_eq!(data, Err(Error::Parity));
    }

    #[test]
    fn into_u32_max() {
        let data = WiegandData {
            facility: 0xFF,
            id: 0xFFFF,
        };

        let data_out: u32 = data.into();
        assert_eq!(data_out, 0x1FFFFFF);
    }

    #[test]
    fn into_u32_min() {
        let data = WiegandData {
            facility: 0x0,
            id: 0x0,
        };

        let data_out: u32 = data.into();
        assert_eq!(data_out, 0x0000001);
    }

    #[test]
    fn into_u32_additional() {
        let data = WiegandData {
            facility: 0x7E,
            id: 0x7FFE,
        };

        let data_out: u32 = data.into();
        assert_eq!(data_out, 0x2FCFFFC);
    }

}

#[cfg(feature = "unstable")]
pub struct WiegandStream<R, T, U>
where
    R: Read<T>,
    T: embedded_hal::timer::CountDown<Time = U>,
    U: From<Duration>,
{
    pub wiegand: R,
    pub timer: T,
}

#[cfg(feature = "unstable")]
impl<R, T, U> Stream for WiegandStream<R, T, U>
where
    R: Read<T>,
    T: embedded_hal::timer::CountDown<Time = U>,
    U: From<Duration>,
{
    type Item = WiegandData;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Error> {
        Ok(Async::Ready(Some(nb::try_nb!(self
            .wiegand
            .read(&mut self.timer)))))
    }
}
