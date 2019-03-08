#![no_std]

use embedded_hal::digital::InputPin;

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

    pub fn read(&mut self) -> WiegandData {
        let mut data_in: u32 = 0;

        for _bit in 0..26 {
            while self.data0.is_high() && self.data1.is_high() {}

            //shift first because we have room to spare on the left
            //and because we dont want to be shifted over after bit 26
            data_in <<= 1;

            if self.data1.is_low() {
                data_in |= 1;
            }

            while self.data0.is_low() || self.data1.is_low() {}
        }

        data_in.into()
    }
}
