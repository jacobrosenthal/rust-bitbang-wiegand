use bitbang_wiegand::{WiegandData, WiegandOutput};
use rppal::{gpio::Gpio, hal::Delay};
use std::{thread, time};

const DATA0: u8 = 18;
const DATA1: u8 = 17;

fn main() -> ! {
    // Retrieve the GPIO pin and configure it as an output.
    let data0 = Gpio::new().unwrap().get(DATA0).unwrap().into_output();
    let data1 = Gpio::new().unwrap().get(DATA1).unwrap().into_output();

    let mut wiegand = WiegandOutput::new(data0, data1);
    let mut delay = Delay::new();

    let sleep = time::Duration::from_millis(3000);

    loop {
        let data = WiegandData {
            facility: 255,
            id: 1,
        };

        wiegand.write(&mut delay, data);
        thread::sleep(sleep);
    }
}
