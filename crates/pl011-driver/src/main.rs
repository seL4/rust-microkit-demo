#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, memory_region_symbol, Channel};

use uart_interface_types::*;

mod device;

use device::{Pl011Device, Pl011RegisterBlock};

// FIXME These probably don't need to be global...
const DEVICE: Channel = Channel::new(0);
const ASSISTANT: Channel = Channel::new(1);

#[protection_domain]
fn init() -> SerialHandler<Pl011Device> {
    let device = unsafe { Pl011Device::new(
        memory_region_symbol!(pl011_register_block: *mut Pl011RegisterBlock).as_ptr(),
    ) };
    device.init();

    SerialHandler::<Pl011Device>::new(device, DEVICE, ASSISTANT)
}
