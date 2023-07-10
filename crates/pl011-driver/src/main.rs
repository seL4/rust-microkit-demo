#![no_std]
#![no_main]
#![feature(never_type)]

use core::ptr;

use sel4cp::{protection_domain, Channel};

use uart_interface_types::*;

mod device;

use device::{Pl011Device, Pl011RegisterBlock};

// FIXME These probably don't need to be global...
const DEVICE: Channel = Channel::new(0);
const ASSISTANT: Channel = Channel::new(1);

#[no_mangle]
#[link_section = ".data"]
static mut pl011_register_block: *const Pl011RegisterBlock = ptr::null();

#[protection_domain]
fn init() -> SerialHandler<Pl011Device> {
    let device = unsafe { Pl011Device::new(pl011_register_block) };
    device.init();

    SerialHandler::<Pl011Device>::new(device, DEVICE, ASSISTANT)
}
