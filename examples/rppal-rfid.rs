use bitbang_wiegand::Wiegand;
use embedded_hal::blocking::Delay;
use rppal::gpio::Gpio;

const DATA0: u8 = 18;
const DATA1: u8 = 17;

fn main() -> ! {
    // Retrieve the GPIO pin and configure it as an output.
    let data0 = Gpio::new().unwrap().get(DATA0).unwrap().into_input();
    let data1 = Gpio::new().unwrap().get(DATA1).unwrap().into_input();

    let mut delay = Delay {};

    let mut wiegand = Wiegand::new(data0, data1, delay);

    loop {
        let data = wiegand.read();
        println!("facility:{} id:{}", data.facility, data.id);
    }
}
