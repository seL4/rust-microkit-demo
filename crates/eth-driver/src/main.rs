#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, Channel, Handler};
use banscii_eth_driver_interface as interface;

#[protection_domain]
fn init() -> ThisHandler {
    todo!()
}

struct ThisHandler();

impl Handler for ThisHandler {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        todo!()
    }
}
