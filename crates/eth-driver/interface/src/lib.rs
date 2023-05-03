#![no_std]

use zerocopy::{AsBytes, FromBytes};
use sel4cp::{message::NoMessageValue, memory_region::{MemoryRegion, ReadWrite}};
use smoltcp::{phy, time::Instant};
use core::default::default;
use heapless;

const MTU: usize = 1500;

/// Number of buffers available for transmitting frames. Set to an arbitrary value for now.
const TX_BUF_SIZE: usize = 8;
/// Number of buffers available for receiving frames. Set to an arbitrary value for now.
const RX_BUF_SIZE: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Default, AsBytes, FromBytes)]
#[repr(C)]
pub struct PduSlot {
    pub(crate) index: usize,
    pub(crate) length: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, AsBytes, FromBytes)]
#[repr(C)]
pub struct RingBuffer<const SIZE: usize> {
    take_index: usize,
    put_index: usize,
    pdu_slots: [PduSlot; SIZE],
}

impl<const SIZE: usize> RingBuffer<SIZE> {
    pub fn flush(&mut self) {
        self.read_index = 0;
        self.write_index = 0;
        self.entries = [0u8; SIZE];
    }

    pub fn len(&self) -> usize {
        ((SIZE + self.write_index) - self.read_index) % SIZE
    }

    pub fn is_full(&self) -> bool {
        self.len() == SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn take_pdu(&mut self) -> Option<PduSlot> {
        if self.is_empty() {
            None
        } else {
            let pdu = self.entries[self.take_index];
            self.take_index += 1;

            Some(pdu)
        }
    }

    pub fn put_pdu(&mut self, pdu: PduSlot, length: usize) -> Option<()> {
        if self.is_full() {
            None
        } else {
            pdu.length = length;
            self.entries[self.put_index] = pdu;
            self.put_index += 1;

            Some(())
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, AsBytes, FromBytes)]
#[repr(C)]
pub struct PduBuffer<const SIZE: usize> {
    used: RingBuffer<SIZE>,
    free: RingBuffer<SIZE>, // XXX Maybe remove length field here?
    bufs: [[u8; MTU]; SIZE],
}

// XXX Synchronization needed
impl<const SIZE: usize> PduBuffer<SIZE> {
    pub fn init(&mut self) {
        self.used.flush();
        self.free.flush();
        for i in 0..SIZE {
            self.free.put_pdu(i, 0);
        }
    }

    pub fn take_pdu(&mut self) -> Option<usize> {
        self.free.take_pdu()
    }

    pub fn put_pdu(&mut self, pdu: PduSlot, length: usize) -> Option<()> {
        self.free.put_pdu(pdu, length)
    }

    pub fn read_pdu(&mut self) -> Option<heapless::Vec<u8, MTU>> {
        let pdu = self.used.take_pdu()?;
        let mut res = heapless::Vec::<u8, MTU>::new();
        for i in 0..pdu.length {
            res.push(self.bufs[pdu.index][i]);
        }

        // XXX This is safe, as long as no one is cloning PDUs...
        self.free.put_pdu(pdu, 0)?;

        res
    }

    pub fn write_pdu(&mut self, pdu: PduSlot, bytes: &[u8]) -> Option<()> {
        let pdu = self.free.take()?;
        for i in 0..bytes.len() {
            self.bufs[pdu.index][i] = bytes[i];
        }

        self.used.put_pdu(pdu, bytes.len())?;

        Some(())
    }
}

pub struct EthDevice {
    tx_ring: MemoryRegion<PduBuffer<TX_BUF_SIZE>, ReadWrite>,
    rx_ring: MemoryRegion<PduBuffer<RX_BUF_SIZE>, ReadWrite>,
}

impl EthDevice {
    pub fn new(
        tx_ring: MemoryRegion<PduBuffer<TX_BUF_SIZE>, ReadWrite>,
        rx_ring: MemoryRegion<PduBuffer<RX_BUF_SIZE>, ReadWrite>,
    ) -> Self {
        Self {
            tx_ring,
            rx_ring,
        }
    }
}

struct TxToken(pub(crate) PduSlot);

impl phy::TxToken for TxToken {
    fn consume<R, F>(self, len: usize, f: impl FnOnce(&mut [u8]) -> R) -> R {
        todo!()
    }
}

struct RxToken(pub(crate) PduSlot);

impl phy::RxToken for RxToken {
    fn consume<R, F>(self, f: impl FnOnce(&mut [u8]) -> R) -> R {
        todo!()
    }
}

impl phy::Device for EthDevice {
    type TxToken = TxToken;
    type RxToken = RxToken;

    fn receive(&mut self, timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        todo!()
    }

    fn transmit(&mut self, timestamp: Instant) -> Option<Self::TxToken<'_>> {
        todo!()
    }

    fn capabilities(&self) -> phy::DeviceCapabilities {
        phy::DeviceCapabilities {
            medium: phy::Medium::Ethernet,
            max_transmission_unit: MTU,
            max_burst_size: None,
            checksum: phy::ChecksumCapabilities { // XXX Which of these do we want here?
                ipv4: phy::Checksum::None,
                udp: phy::Checksum::None,
                tcp: phy::Checksum::None,
                icmpv4: phy::Checksum::None,
                icmpv6: phy::Checksum::None,
            }
        }
    }
}
