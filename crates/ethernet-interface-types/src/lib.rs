#![no_std]

use zerocopy::{AsBytes, FromBytes};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sel4cp::{Channel, message::MessageInfo, memory_region::{ExternallySharedRef, ExternallySharedPtr, ReadOnly, ReadWrite}};
use smoltcp::{phy, time::Instant};

mod ringbuffer;
use crate::ringbuffer::*;

// Assuming a fixed (standard) MTU for now.
// TODO Revisit once we know more about hardware.
const MTU: usize = 1500;

/// Number of buffers available for transmitting frames. Set to an arbitrary value for now.
const TX_BUF_SIZE: usize = 8;
/// Number of buffers available for receiving frames. Set to an arbitrary value for now.
const RX_BUF_SIZE: usize = 8;

pub type Buf = [u8; MTU];
pub type Bufs = [Buf];

pub struct EthDevice {
    tx_ring: RingBuffer<usize, TX_BUF_SIZE>,
    tx_bufs: ExternallySharedRef<'static, Bufs, ReadWrite>,
    tx_chan: Channel,
    rx_ring: RingBuffer<RxReadyMsg, RX_BUF_SIZE>,
    rx_bufs: ExternallySharedRef<'static, Bufs, ReadWrite>,
    rx_chan: Channel,
}

impl EthDevice {
    /// Constructor requiring pointers to the respective buffers. These should be constructed
    /// using
    ///
    /// ```
    /// let tx_bufs_ptr = memory_region_symbol!(my_tx_buf_symbol: *mut [Buf], n = TX_BUF_SIZE);
    /// let rx_bufs_ptr = memory_region_symbol!(my_rx_buf_symbol: *mut [Buf], n = RX_BUF_SIZE);
    /// ```
    ///
    /// A couple of things to note:
    ///     * It's necessary to use [Buf], rather than the Bufs type alias, due to how
    ///       memory_region_symbol is defined
    ///     * The region pointed to by `my_tx_buf_symbol` should be TX_BUF_SIZE * MTU bytes (resp.
    ///       `my_rx_buf_symbol`)
    pub fn new(
        tx_bufs_ptr: core::ptr::NonNull<Bufs>,
        tx_chan: Channel,
        rx_bufs_ptr: core::ptr::NonNull<Bufs>,
        rx_chan: Channel,
    ) -> Self {
        let tx_bufs = unsafe { ExternallySharedRef::<'static, Bufs>::new(tx_bufs_ptr) };

        let rx_bufs = unsafe { ExternallySharedRef::<'static, Bufs>::new(rx_bufs_ptr) };

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

pub struct TxToken<'a> {
    index: usize,
    buf: ExternallySharedPtr<'a, Buf, ReadWrite>,
    chan: Channel
}

impl<'a> phy::TxToken for TxToken<'a> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, length: usize, f: F) -> R {
        debug_assert!(length <= MTU);

        let mut buf = [0; MTU];
        let res = f(&mut buf);
        self.buf.write(buf);

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

pub struct RxToken<'a> {
    index: usize,
    length: usize,
    buf: ExternallySharedPtr<'a, Buf, ReadOnly>,
    chan: Channel
}

impl<'a> phy::RxToken for RxToken<'a> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, f: F) -> R {
        let mut buf = self.buf.read();
        let res = f(&mut buf[0..self.length]);

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
    type TxToken<'a> = TxToken<'a>;
    type RxToken<'a> = RxToken<'a>;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let rx_ready = self.rx_ring.take()?;
        let tx_index = self.tx_ring.take()?; // XXX Handle the case where rx is non-empty, but tx
                                             // is full; not important now, since RX_BUF_SIZE =
                                             // TX_BUF_SIZE

        let rx_buf = self.rx_bufs.as_ptr().index(rx_ready.index);
        let tx_buf = self.tx_bufs.as_mut_ptr().index(tx_index);

        Some((
            Self::RxToken {
                index: rx_ready.index,
                length: rx_ready.length,
                buf: rx_buf,
                chan: self.rx_chan,
            },
            Self::TxToken {
                index: tx_index,
                buf: tx_buf,
                chan: self.tx_chan,
            },
        ))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        let index = self.tx_ring.take()?;

        let tx_buf = self.tx_bufs.as_mut_ptr().index(index);

        Some(Self::TxToken {
            index,
            buf: tx_buf,
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
