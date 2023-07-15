#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, memory_region_symbol, Channel, Handler};
use sel4cp::message::{MessageInfo};
use sel4cp::debug_print;

use smoltcp::phy::{Device, TxToken};
use smoltcp::time::Instant;

#[allow(unused_imports)]
use eth_driver_interface as interface;

const DRIVER: Channel = Channel::new(2);
const ETH_TEST: Channel = Channel::new(3);

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

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            ETH_TEST => {
                debug_print!("Got notification!\n");
                match self.device.transmit(Instant::from_millis(100)) {
                    None => {debug_print!("Didn't get a transmit token\n");},
                    Some(tx) => {
                        debug_print!("Sending some data\n");
                        tx.consume(4, |buffer| {buffer[0] = 1})
                    }
                }
                match self.device.receive(Instant::from_millis(100)) {
                    None => {debug_print!("Didn't get RX tokens\n");},
                    Some(tokens) => {
                        debug_print!("Got some Rx tokens\n");
                    }
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        debug_print!("Got here\n");
        Ok(match channel {
            DRIVER => self.device.server_tag_handler(msg_info),
            _ => unreachable!(),
        })
    }
}