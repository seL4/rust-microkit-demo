#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, memory_region_symbol, Channel, Handler};
#[allow(unused_imports)]
use eth_driver_interface as interface;

const CLIENT: Channel = Channel::new(2);

#[protection_domain]
fn init() -> interface::EthHandler {
    unsafe {
        interface::new_eth_handler!(CLIENT, tx_buf_region_start, rx_buf_region_start)
    }
}
