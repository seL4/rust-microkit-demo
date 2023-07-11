#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, memory_region_symbol, Channel, Handler};
use sel4cp::message::{MessageInfo};

#[allow(unused_imports)]
use ethernet_interface_types as interface;
use ethernet_interface_types::TX_BUF_SIZE;
use ethernet_interface_types::Buf;
use ethernet_interface_types::RX_BUF_SIZE;

const DRIVER: Channel = Channel::new(2);

#[protection_domain]
fn init() -> ThisHandler {
    let device = unsafe { interface::new_eth_device!(DRIVER, tx_buf_region_start,rx_buf_region_start) };
    
    ThisHandler{
        device,
    }
}

struct ThisHandler{
    device: interface::EthDevice,
}

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
