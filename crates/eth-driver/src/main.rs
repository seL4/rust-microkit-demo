#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, memory_region_symbol, Channel, Handler};
use sel4_shared_ring_buffer::RawRingBuffer;
#[allow(unused_imports)]
use eth_driver_interface as interface;

const CLIENT: Channel = Channel::new(2);

#[protection_domain]
fn init() -> interface::EthHandler {
    unsafe {
        interface::EthHandler::new(
            CLIENT,
            memory_region_symbol!(tx_free_region_start: *mut RawRingBuffer),
            memory_region_symbol!(tx_used_region_start: *mut RawRingBuffer),
            memory_region_symbol!(tx_buf_region_start: *mut [interface::Buf], n = interface::TX_BUF_SIZE),
            memory_region_symbol!(rx_free_region_start: *mut RawRingBuffer),
            memory_region_symbol!(rx_used_region_start: *mut RawRingBuffer),
            memory_region_symbol!(rx_buf_region_start: *mut [interface::Buf], n = interface::RX_BUF_SIZE),
        )
    }
}
