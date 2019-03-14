#![no_std]

use embedded_hal::digital::InputPin;
use embedded_hal::timer::{CountDown, Periodic};
use nb::block;

pub struct Wiegand<Data0, Data1, Timer>
where
    Data0: InputPin,
    Data1: InputPin,
    Timer: CountDown + Periodic,
{
    data0: Data0,
    data1: Data1,
    timer: Timer,
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

impl<Data0, Data1, Timer> Wiegand<Data0, Data1, Timer>
where
    Data0: InputPin,
    Data1: InputPin,
    Timer: CountDown + Periodic,
{
    pub fn new(data0: Data0, data1: Data1, timer: Timer) -> Self {
        Self {
            data0,
            data1,
            timer,
        }
    }
}

pub trait Read<WiegandData> {
    /// Read error
    type Error;

    /// Reads a single word from the serial interface
    fn read(&mut self) -> nb::Result<WiegandData, Self::Error>;
}

impl<Data0, Data1, Timer> Read<WiegandData> for Wiegand<Data0, Data1, Timer>
where
    Data0: InputPin,
    Data1: InputPin,
    Timer: CountDown + Periodic,
{
    type Error = ();

    fn read(&mut self) -> nb::Result<WiegandData, Self::Error> {
        let mut data_in: u32 = 0;

        for _bit in 0..26 {
            while self.data0.is_high() && self.data1.is_high() {
                block!(self.timer.wait()).ok();
            }

            //shift first because we have room to spare on the left
            //and because we dont want to be shifted over after bit 26
            data_in <<= 1;

            if self.data1.is_low() {
                data_in |= 1;
            }

            while self.data0.is_low() || self.data1.is_low() {
                block!(self.timer.wait()).ok();
            }
        }

        Ok(data_in.into())
    }
}
