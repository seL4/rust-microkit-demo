#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, Channel, Handler};
use sel4cp::message::{MessageInfo};

//#[allow(unused_imports)]
//use ethernet_interface_types as interface;

#[protection_domain]
fn init() -> ThisHandler {
    ThisHandler{}
}

struct ThisHandler();

impl Handler for ThisHandler {
    type Error = !;

    fn notified(&mut self, _channel: Channel) -> Result<(), Self::Error> {
        todo!()
    }

    fn protected(
        &mut self,
        _channel: Channel,
        _msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        todo!()
    }
}
