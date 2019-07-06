use bitbang_wiegand::{Bit, WiegandInputInterrupted};
use rppal::{gpio::Gpio, gpio::Trigger};
use std::sync::mpsc::{channel, Receiver as ThreadIn, Sender as ThreadOut};
use std::sync::{Arc, Mutex};

const READER1_DATA0: u8 = 14;
const READER1_DATA1: u8 = 15;

fn main() -> ! {
    let (tx, rx): (ThreadOut<Bit>, ThreadIn<Bit>) = channel();

    let mut reader1_data0 = Gpio::new()
        .unwrap()
        .get(READER1_DATA0)
        .unwrap()
        .into_input_pullup();
    let mut reader1_data1 = Gpio::new()
        .unwrap()
        .get(READER1_DATA1)
        .unwrap()
        .into_input_pullup();

    let wrapped_tx1 = Arc::new(Mutex::new(tx.clone()));

    reader1_data0
        .set_async_interrupt(Trigger::FallingEdge, move |_| {
            wrapped_tx1
                .clone()
                .lock()
                .expect("lock poison on wrapped_tx")
                .send(Bit::Low)
                .expect("could not send reader1_data0 interrupt");
        })
        .expect("could not set reader1_data0 interrupt");

    let wrapped_tx2 = Arc::new(Mutex::new(tx.clone()));

    reader1_data1
        .set_async_interrupt(Trigger::FallingEdge, move |_| {
            wrapped_tx2
                .clone()
                .lock()
                .expect("lock poison on wrapped_tx")
                .send(Bit::High)
                .expect("could not send reader1_data0 interrupt");
        })
        .expect("could not set reader1_data1 interrupt");

    let mut reader = WiegandInputInterrupted::new();

    loop {
        while let Ok(bit) = rx.recv() {
            match reader.accumulate(bit) {
                Err(nb::Error::Other(e)) => {
                    let _ = println!("reader1 read error {:?}", e);
                    ()
                }
                Err(nb::Error::WouldBlock) => {}
                Ok(data) => {
                    let _ = println!("facility:{} id:{}", data.facility, data.id);
                    ()
                }
            }
        }
    }
}
