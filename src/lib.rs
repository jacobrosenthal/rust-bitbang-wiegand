#![feature(integer_atomics)]
#![no_std]

use atomic::{AtomicBool, AtomicU32};
use embedded_hal::digital::InputPin;
use nb;

pub enum Error {
    #[doc(hidden)]
    _Extensible,
}

pub struct Wiegand<Data0, Data1>
where
    Data0: InputPin,
    Data1: InputPin,
{
    data0: Data0,
    data1: Data1,
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

pub trait Read<WiegandData> {
    /// Read error
    type Error;

    /// Reads a single word from the serial interface
    fn read(&mut self) -> nb::Result<WiegandData, Error>;
}

impl<Data0, Data1> Read<WiegandData> for Wiegand<Data0, Data1>
where
    Data0: InputPin,
    Data1: InputPin,
{
    type Error = Error;

    // isnt this screaming to be a generator?
    //https://github.com/rust-lang/rust/issues/43122
    fn read(&mut self) -> nb::Result<WiegandData, Error> {
        // how to singleton this instead so we can static this for reentrancy
        //start at 1, so when that becomes 27th bit were done
        static mut data_in: AtomicU32 = AtomicU32(1);
        static mut still_low: AtomicBool = AtomicBool::new(false);;

        if self.data0.is_high() && self.data1.is_high() {
            still_low = false;
            return Err(nb::Error::WouldBlock);
        } else if still_low {
            return Err(nb::Error::WouldBlock);
        } else {
            //shift first because we have room to spare on the left
            //and because we dont want to be shifted over after bit 26
            data_in <<= 1;

            if self.data1.is_low() {
                data_in |= 1;
            }

            //detect 27th bit, this is probably too cute, just use a u8
            if data_in & 0x4000000 == 0x4000000 {
                //reset data_in
                return Ok(data_in.into());
            }

            //need to detect that we havent gone high yet....
            still_low = true;

            return Err(nb::Error::WouldBlock);
        }
    }
}
