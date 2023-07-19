#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, memory_region_symbol, Channel};

use uart_interface_types::*;

mod device;

use device::{UartDevice, UartRegisterBlock};

// FIXME These probably don't need to be global...
const UART_IRQ: Channel = Channel::new(53);
const ASSISTANT: Channel = Channel::new(1);

#[protection_domain]
fn init() -> SerialHandler<UartDevice> {
    let device = unsafe { UartDevice::new(
        memory_region_symbol!(uart_register_block: *mut UartRegisterBlock).as_ptr(),
    ) };
    device.init();

    SerialHandler::<UartDevice>::new(device, UART_IRQ, ASSISTANT)
}
