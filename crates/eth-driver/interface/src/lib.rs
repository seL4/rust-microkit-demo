#![no_std]

use zerocopy::{AsBytes, FromBytes};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sel4cp::{Channel, message::MessageInfo, memory_region::{MemoryRegion, ReadWrite}};
use smoltcp::{phy, time::Instant};

use crate::ring_buffer::*;

// Assuming a fixed (standard) MTU for now.
// TODO Revisit once we know more about hardware.
const MTU: usize = 1500;

/// Number of buffers available for transmitting frames. Set to an arbitrary value for now.
const TX_BUF_SIZE: usize = 8;
/// Number of buffers available for receiving frames. Set to an arbitrary value for now.
const RX_BUF_SIZE: usize = 8;

pub type Buf = [u8; MTU];
pub type Bufs<const SIZE: usize> = [Buf; SIZE];

pub struct EthDevice {
    tx_ring: RingBuffer<usize, TX_BUF_SIZE>,
    tx_bufs: MemoryRegion<Bufs<TX_BUF_SIZE>, ReadWrite>,
    tx_chan: Channel,
    rx_ring: RingBuffer<RxReadyMsg, RX_BUF_SIZE>,
    rx_bufs: MemoryRegion<Bufs<RX_BUF_SIZE>, ReadWrite>,
    rx_chan: Channel,
}

impl<'a> EthDevice {
    pub fn new(
        tx_bufs: MemoryRegion<Bufs<TX_BUF_SIZE>, ReadWrite>, // XXX Pass in a ptr?
        tx_chan: Channel,
        rx_bufs: MemoryRegion<Bufs<RX_BUF_SIZE>, ReadWrite>, // XXX Pass in a ptr?
        rx_chan: Channel,
    ) -> Self {
        Self {
            tx_ring: RingBuffer::<usize, TX_BUF_SIZE>::from_iter(0..TX_BUF_SIZE),
            tx_bufs,
            tx_chan,
            rx_ring: RingBuffer::<RxReadyMsg, RX_BUF_SIZE>::empty(),
            rx_bufs,
            rx_chan,
        }
    }
}

pub struct TxToken {
    index: usize,
    buf: MemoryRegion<&'static mut Buf, ReadWrite>,
    chan: Channel
}

impl phy::TxToken for TxToken {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, length: usize, f: F) -> R {
        let res = f(&mut self.buf.extract_inner()[..]);

        self.chan.pp_call(MessageInfo::send(
            ClientTag::TxReady,
            TxReadyMsg {
                index: self.index,
                length,
            },
        ));

        res
    }
}

pub struct RxToken {
    index: usize,
    length: usize,
    buf: MemoryRegion<&'static mut Buf, ReadWrite>,
    chan: Channel
}

impl phy::RxToken for RxToken {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, f: F) -> R {
        let res = f(&mut self.buf.extract_inner()[0..self.length]);

        self.chan.pp_call(MessageInfo::send(
            ClientTag::RxDone,
            RxDoneMsg {
                index: self.index,
            },
        ));

        res
    }
}

impl phy::Device for EthDevice {
    type TxToken<'a> = TxToken;
    type RxToken<'a> = RxToken;

    fn receive(&mut self, timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let rx_ready = self.rx_ring.take()?;
        let tx_index = self.tx_ring.take()?; // XXX Handle the case where rx is non-empty, buf tx
                                             // is full
        Some((
            Self::RxToken {
                index: rx_ready.index,
                length: rx_ready.length,
                buf: self.rx_bufs.index_mut(rx_ready.index),
                chan: self.rx_chan,
            },
            Self::TxToken {
                index: tx_index,
                buf: self.tx_bufs.index_mut(tx_index),
                chan: self.tx_chan,
            },
        ))
    }

    fn transmit(&mut self, timestamp: Instant) -> Option<Self::TxToken<'_>> {
        let index = self.tx_ring.take()?;

        Some(Self::TxToken {
            index,
            buf: self.tx_bufs.index_mut(index),
            chan: self.tx_chan,
        })
    }

    fn capabilities(&self) -> phy::DeviceCapabilities {
        // Assuming no checksums and a fixed (standard) MTU for now.
        // TODO Revisit these capabilities once we know what hardware we're using.
        let mut caps = phy::DeviceCapabilities::default();
        caps.medium = phy::Medium::Ethernet;
        caps.max_transmission_unit = MTU;
        caps.max_burst_size = None;
        caps.checksum.ipv4 = phy::Checksum::None;
        caps.checksum.udp = phy::Checksum::None;
        caps.checksum.tcp = phy::Checksum::None;
        caps.checksum.icmpv4 = phy::Checksum::None;

        caps
    }
}

#[derive(Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
pub enum ClientTag {
    TxReady,
    RxDone,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, AsBytes, FromBytes)]
#[repr(C)]
pub struct TxReadyMsg {
    pub index: usize,
    pub length: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, AsBytes, FromBytes)]
#[repr(C)]
pub struct RxDoneMsg {
    pub index: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
pub enum ServerTag {
    RxReady,
    TxDone,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, AsBytes, FromBytes)]
#[repr(C)]
pub struct RxReadyMsg {
    pub index: usize,
    pub length: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, AsBytes, FromBytes)]
#[repr(C)]
pub struct TxDoneMsg {
    pub index: usize,
}
