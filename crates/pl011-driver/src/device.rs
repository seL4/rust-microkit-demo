use core::ops::Deref;

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{register_bitfields, register_structs};

use embedded_hal::serial;

use banscii_pl011_driver_interface_types::IrqDevice;

register_structs! {
    #[allow(non_snake_case)]
    pub Pl011RegisterBlock {
        (0x000 => DR: ReadWrite<u8>),
        (0x001 => _reserved0),
        (0x018 => FR: ReadOnly<u32, FR::Register>),
        (0x01c => _reserved1),
        (0x038 => IMSC: ReadWrite<u32, IMSC::Register>),
        (0x03c => _reserved2),
        (0x044 => ICR: WriteOnly<u32, ICR::Register>),
        (0x048 => @END),
    }
}

register_bitfields! {
    u32,

    FR [
        TXFF OFFSET(5) NUMBITS(1) [],
        RXFE OFFSET(4) NUMBITS(1) [],
    ],

    IMSC [
        RXIM OFFSET(4) NUMBITS(1) [],
    ],

    ICR [
        ALL OFFSET(0) NUMBITS(11) [],
    ],
}

#[derive(Clone, Debug)]
pub struct Pl011Device {
    ptr: *const Pl011RegisterBlock,
}

impl Pl011Device {
    pub unsafe fn new(ptr: *const Pl011RegisterBlock) -> Self {
        Self { ptr }
    }

    fn ptr(&self) -> *const Pl011RegisterBlock {
        self.ptr
    }

    pub fn init(&self) {
        self.IMSC.write(IMSC::RXIM::SET);
    }
}

impl IrqDevice for Pl011Device {
    fn handle_irq(&self) {
        self.ICR.write(ICR::ALL::SET);
    }
}

impl Deref for Pl011Device {
    type Target = Pl011RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadError {
    // XXX Errors besides `WouldBlock`?
}

impl serial::Read<u8> for Pl011Device {
    type Error = ReadError;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        // XXX Worry about whether FIFO bit is set?
        if self.FR.matches_all(FR::RXFE::CLEAR) {
            nb::Result::Ok(self.DR.get())
        } else {  // FIFO (or register) is empty; don't block
            Err(nb::Error::WouldBlock)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteError {
    // XXX Errors besides `WouldBlock`?
}

impl serial::Write<u8> for Pl011Device {
    type Error = WriteError;

    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        // XXX Worry about whether FIFO bit is set?
        if self.FR.matches_all(FR::TXFF::SET) { // FIFO
            Err(nb::Error::WouldBlock)
        } else {
            nb::Result::Ok(self.DR.set(byte))
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        // XXX Guessing at how to implement this...
        if self.FR.matches_all(FR::TXFF::SET) { // FIFO
            Err(nb::Error::WouldBlock)
        } else {
            nb::Result::Ok(())
        }
    }
}
