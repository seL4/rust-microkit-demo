#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, memory_region_symbol, Channel, Handler};
use sel4cp::message::{MessageInfo};
use sel4cp::debug_print;
use sel4_shared_ring_buffer::RawRingBuffer;

use smoltcp::phy::{Device, TxToken, RxToken};
use smoltcp::time::Instant;

#[allow(unused_imports)]
use eth_driver_interface as interface;

const DRIVER: Channel = Channel::new(2);
const ETH_TEST: Channel = Channel::new(3);

#[protection_domain]
fn init() -> ThisHandler {
    let device = unsafe {
        interface::EthDevice::new(
            DRIVER,
            memory_region_symbol!(tx_free_region_start: *mut RawRingBuffer),
            memory_region_symbol!(tx_used_region_start: *mut RawRingBuffer),
            memory_region_symbol!(tx_buf_region_start: *mut [interface::Buf], n = interface::TX_BUF_SIZE),
            memory_region_symbol!(rx_free_region_start: *mut RawRingBuffer),
            memory_region_symbol!(rx_used_region_start: *mut RawRingBuffer),
            memory_region_symbol!(rx_buf_region_start: *mut [interface::Buf], n = interface::RX_BUF_SIZE),
        )
    };
    
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
                    None => debug_print!("Didn't get a TX token\n"),
                    Some(tx) => {
                        debug_print!("Got a TX token\nSending some data: 42\n");
                        tx.consume(1, |buffer| buffer[0] = 42)
                    }
                }
                loop {
                    match self.device.receive(Instant::from_millis(100)) {
                        None => continue,
                        Some((rx, _tx)) => {
                            debug_print!("Got an RX token\n");
                            rx.consume(|buffer| debug_print!("RX token contains {}\n", buffer[0]));
                            break;
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}
