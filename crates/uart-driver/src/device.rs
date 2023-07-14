use core::ops::Deref;

use tock_registers::interfaces::{Readable, Writeable, ReadWriteable};
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{register_bitfields, register_structs};

use sel4cp::debug_print;
use embedded_hal::serial;

use uart_interface_types::IrqDevice;

#[link(name = "uartps")]
extern "C" {
    pub fn uartps_init() -> bool;
    pub fn uartps_handle_irq();
    pub fn uartps_rx(byte: *mut u8) -> bool;
}


// ZCU102 has PS UART, see https://docs.xilinx.com/r/en-US/ug1085-zynq-ultrascale-trm/Example-Read-Message-from-RXFIFO-Using-Interrupt-Method
// UART controller section
// or https://github.com/Xilinx/embeddedsw/tree/master/XilinxProcessorIPLib/drivers/uartps
register_structs! {
    #[allow(non_snake_case)]
    pub UartRegisterBlock {
        // must be u32 so the addr offsets match, sigh
        (0x000 => CR: ReadWrite<u32, Control::Register>),
        (0x004 => MR: ReadWrite<u32, Mode::Register>),
        (0x008 => IER: WriteOnly<u32, Interrupt::Register>),
        (0x00C => IDR: WriteOnly<u32, Interrupt::Register>),
        (0x010 => IMR: ReadOnly<u32, Interrupt::Register>),
        (0x014 => ISR: ReadWrite<u32, Interrupt::Register>),
        (0x018 => BAUDGEN: ReadWrite<u32, BaudRateGen::Register>),
        (0x01C => TXTOUT: ReadWrite<u32, ReceiverTimeout::Register>),
        (0x020 => RXWM: ReadWrite<u32, ReceiverFifoTrigger::Register>),
        (0x024 => MODEMCR: ReadWrite<u32, ModemControl::Register>),
        (0x028 => MODEMSR: ReadWrite<u32,ModemStatus::Register>),
        (0x02C => SR: ReadOnly<u32, ChannelStatus::Register>),
        (0x030 => FIFO: ReadWrite<u8>),
        (0x31 => _reserved),
        (0x034 => BAUDDIV: ReadWrite<u32, BaudRateDivider::Register>),
        (0x038 => FLOWDEL: ReadWrite<u32, FlowDelay::Register>),
        (0x03C => _reserved1),
        (0x040 => _reserved2),
        (0x044 => TXWM: ReadWrite<u32, TxFifoTrigger::Register>),
        (0x048 => RXBS: ReadWrite<u32, RxFifoByteStatus::Register>),
        (0x04C => @END),
    }
}

// see https://www.xilinx.com/htmldocs/registers/ug1087/ug1087-zynq-ultrascale-registers.html
register_bitfields! {
    u32,
    Control [
        RXRES OFFSET(0) NUMBITS(1) [],
        TXRES OFFSET(1) NUMBITS(1) [],
        RXEN OFFSET(2) NUMBITS(1) [],
        RXDIS OFFSET(3) NUMBITS(1) [],
        TXEN OFFSET(4) NUMBITS(1) [],
        TXDIS OFFSET(5) NUMBITS(1) [],
        RSSTO OFFSET(6) NUMBITS(1) [],
        STTBRK OFFSET(7) NUMBITS(1) [],
        STPBRK OFFSET(8) NUMBITS(1) [],
    ],
    Mode [
        CLKS OFFSET(0) NUMBITS(1) [],
        CHRL OFFSET(1) NUMBITS(2) [],
        PAR OFFSET(3) NUMBITS(3) [],
        NBSTOP OFFSET(6) NUMBITS(2) [],
        CHMODE OFFSET(8) NUMBITS(2) [],
        WSIZE OFFSET(12) NUMBITS(2) [],
    ],
    Interrupt [
        RTRIG OFFSET(0) NUMBITS(1) [],
        REMPTY OFFSET(1) NUMBITS(1) [],
        RFULL OFFSET(2) NUMBITS(1) [],
        TEMPTY OFFSET(3) NUMBITS(1) [],
        TFULL OFFSET(4) NUMBITS(1) [],
        ROVR OFFSET(5) NUMBITS(1) [],
        FRAME OFFSET(6) NUMBITS(1) [],
        PARE OFFSET(7) NUMBITS(1) [],
        TIMEOUT OFFSET(8) NUMBITS(1) [],
        DMSI OFFSET(9) NUMBITS(1) [],
        TTRIG OFFSET(10) NUMBITS(1) [],
        TNFUL OFFSET(11) NUMBITS(1) [],
        TOVR OFFSET(12) NUMBITS(1) [],
        RBRK OFFSET(13) NUMBITS(1) [],
    ],
    BaudRateGen [
        CD OFFSET(0) NUMBITS(16) [],
    ],
    ReceiverTimeout [
        RTO OFFSET(0) NUMBITS(8) [],
    ],
    ReceiverFifoTrigger [
        RTRIG OFFSET(0) NUMBITS(6) [],
    ],
    ModemControl [
        DTR OFFSET(0) NUMBITS(1) [],
        RTS OFFSET(1) NUMBITS(1) [],
        FCM OFFSET(5) NUMBITS(1) [],
    ],
    ModemStatus [
        DCTS OFFSET(0) NUMBITS(1) [],
        DDSR OFFSET(1) NUMBITS(1) [],
        TERI OFFSET(2) NUMBITS(1) [],
        DDCD OFFSET(3) NUMBITS(1) [],
        CTS OFFSET(4) NUMBITS(1) [],
        DSR OFFSET(5) NUMBITS(1) [],
        RI OFFSET(6) NUMBITS(1) [],
        DCD OFFSET(7) NUMBITS(1) [],
        FCMS OFFSET(8) NUMBITS(1) [],
    ],
    ChannelStatus[
        RTRIG OFFSET(0) NUMBITS(1) [],
        REMPTY OFFSET(1) NUMBITS(1) [],
        RFULL OFFSET(2) NUMBITS(1) [],
        TEMPTY OFFSET(3) NUMBITS(1) [],
        TFULL OFFSET(4) NUMBITS(1) [],
        RACTIVE OFFSET(10) NUMBITS(1) [],
        TACTIVE OFFSET(11) NUMBITS(1) [],
        FDELT OFFSET(12) NUMBITS(1) [],
        TTRIG OFFSET(13) NUMBITS(1) [],
        TNFUL OFFSET(14) NUMBITS(1) [],
    ],
    BaudRateDivider [
        BDIV OFFSET(0) NUMBITS(8) [],
    ],
    FlowDelay [
        FDEL OFFSET(0) NUMBITS(16) [],
    ],
    TxFifoTrigger [
        TTRIG OFFSET(0) NUMBITS(6) [],
    ],
    RxFifoByteStatus [
        byte0_par_err OFFSET(0) NUMBITS(1) [],
        byte0_frm_err OFFSET(1) NUMBITS(1) [],
        byte0_break OFFSET(2) NUMBITS(1) [],
        byte1_par_err OFFSET(3) NUMBITS(1) [],
        byte1_frm_err OFFSET(4) NUMBITS(1) [],
        byte1_break OFFSET(5) NUMBITS(1) [],
        byte2_par_err OFFSET(6) NUMBITS(1) [],
        byte2_frm_err OFFSET(7) NUMBITS(1) [],
        byte2_break OFFSET(8) NUMBITS(1) [],
        byte3_par_err OFFSET(9) NUMBITS(1) [],
        byte3_frm_err OFFSET(10) NUMBITS(1) [],
        byte3_break OFFSET(11) NUMBITS(1) [],
    ]
 }

pub struct UartDevice {
    ptr: *const UartRegisterBlock,
}

impl UartDevice {
    pub unsafe fn new(ptr: *const UartRegisterBlock) -> Self {
        Self { ptr }
    }

    fn ptr(&self) -> *const UartRegisterBlock {
        self.ptr
    }

    pub fn init(&self) {
        unsafe {
            if !uartps_init() {
                debug_print!("Uart init error.\n");
            }
        }
    }

}

impl IrqDevice for UartDevice {
    fn handle_irq(&self) {
        unsafe {
            uartps_handle_irq();
        }
    }
}

impl Deref for UartDevice {
    type Target = UartRegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadError {
    // XXX Errors besides `WouldBlock`?
}

impl serial::Read<u8> for UartDevice {
    type Error = ReadError;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut val: u8 = 0;
        unsafe {
            if uartps_rx(&mut val) {
                return nb::Result::Ok(val);
            } else {
                return Err(nb::Error::WouldBlock);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteError {
    // XXX Errors besides `WouldBlock`?
}

impl serial::Write<u8> for UartDevice {
    type Error = WriteError;

    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        // // Disable the UART transmit interrupts to allow this call to stop a
        // // previous operation that may be interrupt driven.
        // self.IDR.modify_no_read(self.IMR.extract(), Interrupt::TEMPTY::SET + Interrupt::TFULL::SET);
        // // send data
        self.FIFO.set(byte);
        // // If interrupts are enabled as indicated by the receive interrupt, then
        // // enable the TX FIFO empty interrupt, so further action can be taken
        // // for this sending.
        // if self.IMR.matches_any(Interrupt::RFULL::SET + Interrupt::REMPTY::SET + Interrupt::ROVR::SET) {
        //     self.IER.modify_no_read(self.IMR.extract(), Interrupt::TEMPTY::SET);
        // }
        nb::Result::Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        // no flushing right now
        nb::Result::Ok(())
    }
}
