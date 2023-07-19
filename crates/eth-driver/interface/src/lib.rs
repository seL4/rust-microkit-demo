#![no_std]
#![feature(never_type)]

use sel4cp::{Channel, Handler, debug_print};
use sel4cp::memory_region::{ExternallySharedRef, ExternallySharedPtr, ReadWrite};
use sel4_shared_ring_buffer::{RingBuffers, RingBuffer, Descriptor};
use smoltcp::{phy, time::Instant};

pub use sel4_shared_ring_buffer::RawRingBuffer;
pub use heapless::Vec;

// Assuming a fixed (standard) MTU for now.
// TODO Revisit once we know more about hardware.
pub const MTU: usize = 1500;

/// Number of buffers available for transmitting frames. Set to an arbitrary value for now.
pub const TX_BUF_SIZE: usize = 8;
/// Number of buffers available for receiving frames. Set to an arbitrary value for now.
pub const RX_BUF_SIZE: usize = 8;

pub type Buf = heapless::Vec<u8, MTU>;
pub type Bufs = [Buf];

pub struct EthHandler/*<PhyDevice>*/ {
    channel: Channel,
    //phy_device: PhyDevice,
    tx_ring: RingBuffers<'static, ()>,
    tx_bufs: ExternallySharedRef<'static, Bufs, ReadWrite>,
    rx_ring: RingBuffers<'static, ()>,
    rx_bufs: ExternallySharedRef<'static, Bufs, ReadWrite>,
}

impl/*<PhyDevice: phy::Device>*/ EthHandler/*<PhyDevice>*/ {
    // XXX This has a lot of arguments. Maybe use a builder pattern or a macro?
    pub unsafe fn new(
        channel: Channel,
        //phy_device: PhyDevice, 
        tx_free_ptr: core::ptr::NonNull<RawRingBuffer>,
        tx_used_ptr: core::ptr::NonNull<RawRingBuffer>,
        tx_bufs_ptr: core::ptr::NonNull<Bufs>,
        rx_free_ptr: core::ptr::NonNull<RawRingBuffer>,
        rx_used_ptr: core::ptr::NonNull<RawRingBuffer>,
        rx_bufs_ptr: core::ptr::NonNull<Bufs>,
    ) -> Self {
        let tx_free = unsafe { RingBuffer::from_ptr(tx_free_ptr) };
        let tx_used = unsafe { RingBuffer::from_ptr(tx_used_ptr) };
        let rx_free = unsafe { RingBuffer::from_ptr(rx_free_ptr) };
        let rx_used = unsafe { RingBuffer::from_ptr(rx_used_ptr) };

        let mut tx_ring = RingBuffers::new(tx_free, tx_used, (), true);
        let mut rx_ring = RingBuffers::new(rx_free, rx_used, (), true);

        for i in 0..TX_BUF_SIZE {
            tx_ring.free_mut()
                .enqueue(Descriptor::new(i, MTU as u32, 0))
                .expect("Unable to enqueue to TX free ring");
        }
        for i in 0..RX_BUF_SIZE {
            rx_ring.free_mut()
                .enqueue(Descriptor::new(i, MTU as u32, 0))
                .expect("Unable to enqueue to RX free ring");
        }

        let tx_bufs = unsafe { ExternallySharedRef::<'static, Bufs>::new(tx_bufs_ptr) };
        let rx_bufs = unsafe { ExternallySharedRef::<'static, Bufs>::new(rx_bufs_ptr) };

        for i in 0..TX_BUF_SIZE {
            tx_bufs.as_ptr()
                .index(i)
                .as_raw_ptr()
                .as_mut()
                .clear();
        }
        for i in 0..RX_BUF_SIZE {
            rx_bufs.as_ptr()
                .index(i)
                .as_raw_ptr()
                .as_mut()
                .clear();
        }

        Self {
            channel,
            //phy_device,
            tx_ring,
            tx_bufs,
            rx_ring,
            rx_bufs,
        }
    }
}

// TODO Use underlying PhyDevice for send/recv.
impl/*<PhyDevice: phy::Device>*/ Handler for EthHandler/*<PhyDevice>*/ {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        if channel == self.channel {
            match self.tx_ring.used_mut().dequeue() { // TODO We could send more than one frame here...
                Ok(tx_desc) => {
                    // Try to loop the packet back to the client
                    match self.rx_ring.free_mut().dequeue() {
                        Ok(rx_desc) => {
                            // XXX Can we do this withoug the unsafe?
                            let tx_buf = unsafe {
                                self.tx_bufs
                                    .as_ptr()
                                    .index(tx_desc.encoded_addr())
                                    .as_raw_ptr()
                                    .as_ref()
                                    .clone()
                            };
                            let rx_buf_mut = unsafe {
                                self.rx_bufs
                                    .as_mut_ptr()
                                    .index(rx_desc.encoded_addr())
                                    .as_raw_ptr()
                                    .as_mut()
                            };

                            rx_buf_mut.clear();
                            let _ = rx_buf_mut.extend_from_slice(&tx_buf);

                            let _ = self.rx_ring.used_mut().enqueue(Descriptor::new(rx_desc.encoded_addr(), MTU as u32, 0));
                        }
                        Err(_) => debug_print!("Failed to loop back; RX buffer is full"),
                    }

                    // Free the TX buffer
                    let _ = self.tx_ring.free_mut().enqueue(Descriptor::new(tx_desc.encoded_addr(), MTU as u32, 0));
                }
                Err(_) => debug_print!("Driver notified, but TX buffer is empty"),
            }
        } else {
            unreachable!();
        }

        Ok(())
    }
}

pub struct EthDevice {
    channel: Channel,
    tx_ring: RingBuffers<'static, ()>,
    tx_bufs: ExternallySharedRef<'static, Bufs, ReadWrite>,
    rx_ring: RingBuffers<'static, ()>,
    rx_bufs: ExternallySharedRef<'static, Bufs, ReadWrite>,
}

impl EthDevice {
    /// Constructor requiring pointers to the respective buffers.
    ///
    /// # Examples
    ///
    /// ```
    /// let tx_bufs_ptr = memory_region_symbol!(my_tx_buf_symbol: *mut [Buf], n = TX_BUF_SIZE);
    /// let rx_bufs_ptr = memory_region_symbol!(my_rx_buf_symbol: *mut [Buf], n = RX_BUF_SIZE);
    /// ```
    ///
    /// A couple of things to note:
    ///     * It's necessary to use `[Buf]`, rather than the [`Bufs`] type alias, due to how
    ///       memory_region_symbol is defined
    ///     * The region pointed to by `my_tx_buf_symbol` should be [`TX_BUF_SIZE`]* [`MTU`]
    ///       bytes (resp. `my_rx_buf_symbol`)
    pub fn new(
        channel: Channel,
        tx_free_ptr: core::ptr::NonNull<RawRingBuffer>,
        tx_used_ptr: core::ptr::NonNull<RawRingBuffer>,
        tx_bufs_ptr: core::ptr::NonNull<Bufs>,
        rx_free_ptr: core::ptr::NonNull<RawRingBuffer>,
        rx_used_ptr: core::ptr::NonNull<RawRingBuffer>,
        rx_bufs_ptr: core::ptr::NonNull<Bufs>,
    ) -> Self {
        let tx_free = unsafe { RingBuffer::from_ptr(tx_free_ptr) };
        let tx_used = unsafe { RingBuffer::from_ptr(tx_used_ptr) };
        let rx_free = unsafe { RingBuffer::from_ptr(rx_free_ptr) };
        let rx_used = unsafe { RingBuffer::from_ptr(rx_used_ptr) };

        let tx_ring = RingBuffers::new(tx_free, tx_used, (), false);
        let rx_ring = RingBuffers::new(rx_free, rx_used, (), false);

        let tx_bufs = unsafe { ExternallySharedRef::<'static, Bufs>::new(tx_bufs_ptr) };
        let rx_bufs = unsafe { ExternallySharedRef::<'static, Bufs>::new(rx_bufs_ptr) };

        Self {
            channel,
            tx_ring,
            tx_bufs,
            rx_ring,
            rx_bufs,
        }
    }
}

pub struct TxToken<'a> {
    buf: ExternallySharedPtr<'a, Buf, ReadWrite>,
    channel: Channel
}

// TODO Implement Drop to put the buffer back

impl<'a> phy::TxToken for TxToken<'a> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, length: usize, f: F) -> R {
        let mut buf = Buf::default();

        buf.extend(core::iter::repeat(0).take(length));
        let res = f(&mut buf);

        // XXX Can we do this withoug the unsafe?
        let buf_mut = unsafe { self.buf.as_raw_ptr().as_mut() };
        buf_mut.clear();
        let _ = buf_mut.extend_from_slice(&buf);
        let _ = buf_mut.resize(length, 0);

        self.channel.notify();

        res
    }
}

pub struct RxToken {
    buf: Buf,
}

// TODO Implement Drop to put the buffer back

impl phy::RxToken for RxToken {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(mut self, f: F) -> R {
        f(&mut self.buf)
    }
}

impl phy::Device for EthDevice {
    type TxToken<'a> = TxToken<'a>;
    type RxToken<'a> = RxToken;

    fn receive(&mut self, timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        // Ensure we don't take a TX buffer if there's nothing to receive
        if self.rx_ring.used().is_empty() {
            return None;
        }

        let rx_desc = self.rx_ring.used_mut().dequeue().ok()?; // Should succeed; we just checked

        // Copy the buffer and put it back. This avoids needing to notify on read.
        // XXX Can we do this without copying? Without protected calls?
        // XXX Can we do this withoug the unsafe?
        let rx_buf = unsafe {
            self.rx_bufs
                .as_ptr()
                .index(rx_desc.encoded_addr())
                .as_raw_ptr()
                .as_ref()
                .clone()
        };
        let _ = self.rx_ring.free_mut().enqueue(Descriptor::new(rx_desc.encoded_addr(), MTU as u32, 0));

        let tx_token = self.transmit(timestamp)?;

        Some((
            Self::RxToken {
                buf: rx_buf,
            },
            tx_token,
        ))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        let desc = self.tx_ring.free_mut().dequeue().ok()?;
        let buf = self.tx_bufs.as_mut_ptr().index(desc.encoded_addr());

        // Place the buffer to be written into the used ring; the driver won't be notified
        // until we call `consume`.
        // XXX How can we set the length?
        let _ = self.tx_ring.used_mut().enqueue(Descriptor::new(desc.encoded_addr(), MTU as u32, 0));

        Some(Self::TxToken {
            buf,
            channel: self.channel,
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
