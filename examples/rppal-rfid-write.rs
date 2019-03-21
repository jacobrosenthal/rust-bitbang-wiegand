use bitbang_wiegand::{Read, WiegandData, WiegandInput, WiegandOutput};
use rppal::{gpio::Gpio, hal::Delay, hal::Timer};
use std::{thread, time};

const READER_DATA0: u8 = 14;
const READER_DATA1: u8 = 15;

const WRITER_DATA0: u8 = 17;
const WRITER_DATA1: u8 = 18;

fn main() -> ! {
    // Retrieve the GPIO pin and configure it as an output.
    let reader0 = Gpio::new().unwrap().get(READER_DATA0).unwrap().into_input();
    let reader1 = Gpio::new().unwrap().get(READER_DATA1).unwrap().into_input();

    let writer0 = Gpio::new()
        .unwrap()
        .get(WRITER_DATA0)
        .unwrap()
        .into_output();
    let writer1 = Gpio::new()
        .unwrap()
        .get(WRITER_DATA1)
        .unwrap()
        .into_output();

    let mut reader = WiegandInput::new(reader0, reader1);
    let mut writer = WiegandOutput::new(writer0, writer1);
    let mut delay = Delay::new();
    let mut timer = Timer::new();

    let sleep = time::Duration::from_millis(5000);

    loop {
        match reader.read(&mut timer) {
            Err(nb::Error::Other(e)) => println!("read error {:?}", e),
            Err(nb::Error::WouldBlock) => {}
            Ok(data) => {
                println!("facility:{} id:{}", data.facility, data.id);
                thread::sleep(sleep);

                let data = WiegandData {
                    facility: 255,
                    id: 1,
                };

                writer.write(&mut delay, data);
            }
        }
    }
}
