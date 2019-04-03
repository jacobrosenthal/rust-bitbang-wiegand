#![no_std]

use rppal::{hal::Hertz};
use embedded_hal::digital::InputPin;
use embedded_hal::timer::CountDown;

use nb;

pub struct Wiegand<Data0, Data1>
where
    Data0: InputPin,
    Data1: InputPin,
{
    data0: Data0,
    data1: Data1,
}

#[derive(Debug)]
pub enum Error {
    TimedOut,
    _Extensible,
}

pub struct WiegandData {
    pub facility: u8,
    pub id: u16,
}

//todo.. cute but.. is it confusing?
impl From<u32> for WiegandData {
    fn from(item: u32) -> Self {
        let id: u16 = (item >> 1) as u16;
        let facility: u8 = (item >> 17) as u8;
        Self { facility, id }
    }
}

impl<Data0, Data1> Wiegand<Data0, Data1>
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
    fn read(&mut self, mut timer: T) -> nb::Result<WiegandData, Error>;
}

//might need +Cancel if this doesnt accept https://github.com/rust-embedded/embedded-hal/issues/106
impl<Data0, Data1, T, U> Read<T> for Wiegand<Data0, Data1>
where
    Data0: InputPin,
    Data1: InputPin,
    T: embedded_hal::timer::CountDown<Time = U>,
    U: From<Hertz>,
{
    fn read(&mut self, mut timer: T) -> nb::Result<WiegandData, Error> {
        while self.data0.is_high() && self.data1.is_high() {
            return Err(nb::Error::WouldBlock);
        }

        let mut data_in: u32 = 0;

        for _bit in 0..26 {
            //more than 100us here would be a problem so 1ms?
            //were not blocking on timer.wait so if its resolution is less than ms
            // say us, then this will tick at us until we go high again
            // or error if it gets up to 1ms
            timer.start(Hertz(1000));
            while self.data0.is_high() && self.data1.is_high() {
                match timer.wait() {
                    Err(nb::Error::Other(_e)) => unreachable!(),
                    Err(nb::Error::WouldBlock) => continue,
                    Ok(()) => return Err(nb::Error::Other(Error::TimedOut)),
                }
            }

            //shift first because we have room to spare on the left
            //and because we dont want to be shifted over after bit 26
            data_in <<= 1;

            if self.data1.is_low() {
                data_in |= 1;
            }

            //more than 1ms here would be a problem so 2ms?
            //were not blocking on timer.wait so if its resolution is less than ms
            // say us, then this will tick at us until we go high again
            // or error if it gets up to 2ms
            timer.start(Hertz(2000));
            while self.data0.is_low() || self.data1.is_low() {
                match timer.wait() {
                    Err(nb::Error::Other(_e)) => unreachable!(),
                    Err(nb::Error::WouldBlock) => continue,
                    Ok(()) => return Err(nb::Error::Other(Error::TimedOut)),
                }
            }
        }

        Ok(data_in.into())
    }
}
