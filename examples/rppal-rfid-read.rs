use bitbang_wiegand::{Read, WiegandInput};
use rppal::{gpio::Gpio, hal::Timer};

const READER1_DATA0: u8 = 14;
const READER1_DATA1: u8 = 15;

const READER2_DATA0: u8 = 17;
const READER2_DATA1: u8 = 18;

fn main() -> ! {
    let reader1_data0 = Gpio::new()
        .unwrap()
        .get(READER1_DATA0)
        .unwrap()
        .into_input_pullup();
    let reader1_data1 = Gpio::new()
        .unwrap()
        .get(READER1_DATA1)
        .unwrap()
        .into_input_pullup();

    let reader2_data0 = Gpio::new()
        .unwrap()
        .get(READER2_DATA0)
        .unwrap()
        .into_input_pullup();
    let reader2_data1 = Gpio::new()
        .unwrap()
        .get(READER2_DATA1)
        .unwrap()
        .into_input_pullup();

    let mut reader1 = WiegandInput::new(reader1_data0, reader1_data1);
    let mut reader2 = WiegandInput::new(reader2_data0, reader2_data1);

    let mut timer = Timer::new();

    loop {
        match reader1.read(&mut timer) {
            Err(nb::Error::Other(e)) => println!("reader1 read error {:?}", e),
            Err(nb::Error::WouldBlock) => {}
            Ok(data) => println!("facility:{} id:{}", data.facility, data.id),
        }

        match reader2.read(&mut timer) {
            Err(nb::Error::Other(e)) => println!("reader2 read error {:?}", e),
            Err(nb::Error::WouldBlock) => {}
            Ok(data) => println!("facility:{} id:{}", data.facility, data.id),
        }
    }
}
