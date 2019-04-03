use bitbang_wiegand::{Read, Wiegand};
use nb::block;
use rppal::{gpio::Gpio, hal::Timer};

const HID_DATA0: u8 = 2;
const HID_DATA1: u8 = 3;

const THREE_ONE_EIGHT_DATA0: u8 = 14;
const THREE_ONE_EIGHT_DATA1: u8 = 15;
const THREE_ONE_EIGHT_GND: u8 = 4;

fn main() -> ! {
    // Retrieve the GPIO pin and configure it as an input.
    let hid_data0 = Gpio::new()
        .unwrap()
        .get(HID_DATA0)
        .unwrap()
        .into_input_pullup();
    let hid_data1 = Gpio::new()
        .unwrap()
        .get(HID_DATA1)
        .unwrap()
        .into_input_pullup();

    // Retrieve the GPIO pin and configure it as an input.
    let three_data0 = Gpio::new()
        .unwrap()
        .get(THREE_ONE_EIGHT_DATA0)
        .unwrap()
        .into_input_pullup();
    let three_data1 = Gpio::new()
        .unwrap()
        .get(THREE_ONE_EIGHT_DATA1)
        .unwrap()
        .into_input_pullup();

    //todo probably not ideal
    let mut gnd = Gpio::new()
        .unwrap()
        .get(THREE_ONE_EIGHT_GND)
        .unwrap()
        .into_output();
    gnd.set_low();

    let mut wiegand1 = Wiegand::new(hid_data0, hid_data1);
    let mut wiegand2 = Wiegand::new(three_data0, three_data1);

    let timer = Timer::new();

    loop {
        match wiegand1.read(timer) {
            Err(nb::Error::Other(e)) => println!("wiegand1 read error {:?}", e),
            Err(nb::Error::WouldBlock) => {}
            Ok(data) => println!("facility:{} id:{}", data.facility, data.id),
        }

        match wiegand2.read(timer) {
            Err(nb::Error::Other(e)) => println!("wiegand2 read error {:?}", e),
            Err(nb::Error::WouldBlock) => {}
            Ok(data) => println!("facility:{} id:{}", data.facility, data.id),
        }
    }
}
