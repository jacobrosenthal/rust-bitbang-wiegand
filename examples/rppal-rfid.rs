use bitbang_wiegand::WiegandInput;
use rppal::gpio::Gpio;

const DATA0: u8 = 18;
const DATA1: u8 = 17;

fn main() -> ! {
    // Retrieve the GPIO pin and configure it as an output.
    let data0 = Gpio::new().unwrap().get(DATA0).unwrap().into_input_pullup();
    let data1 = Gpio::new().unwrap().get(DATA1).unwrap().into_input_pullup();

    let mut wiegand = WiegandInput::new(data0, data1);

    loop {
        match wiegand.read() {
            Ok(data) => println!("facility:{} id:{}", data.facility, data.id),
            Err(e) => println!("error reading: {:?}", e),
        }
    }
}
