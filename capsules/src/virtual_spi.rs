use core::cell::Cell;
use kernel::common::{List, ListLink, ListNode};
use kernel::common::take_cell::TakeCell;
use kernel::hil;

/// The Mux struct manages multiple SPI clients. Each client may have
/// at most one outstanding SPI request.
pub struct MuxSPIMaster<'a, SPI: hil::spi::SpiMaster + 'a> {
    spi: &'a SPI,
    devices: List<'a, SPIMasterDevice<'a, SPI>>,
    inflight: TakeCell<&'a SPIMasterDevice<'a, SPI>>,
}

impl<'a, SPI: hil::spi::SpiMaster> hil::spi::SpiMasterClient for MuxSPIMaster<'a, SPI> {
    fn read_write_done(&self,
                       write_buffer: &'static mut [u8],
                       read_buffer: Option<&'static mut [u8]>,
                       len: usize) {
        self.inflight.take().map(move |device| {
            device.read_write_done(write_buffer, read_buffer, len);
        });
        self.do_next_op();
    }
}

impl<'a, SPI: hil::spi::SpiMaster> MuxSPIMaster<'a, SPI> {
    pub const fn new(spi: &'a SPI) -> MuxSPIMaster<'a, SPI> {
        MuxSPIMaster {
            spi: spi,
            devices: List::new(),
            inflight: TakeCell::empty(),
        }
    }

    fn do_next_op(&self) {
        if self.inflight.is_none() {
            let mnode = self.devices.iter().find(|node| node.operation.get() != Op::Idle);
            mnode.map(|node| {

                match node.operation.get() {
                    Op::Configure(cpol, cpal, rate) => {

                        // The `chip_select` type will be correct based on
                        // what implemented `SpiMaster`.
                        self.spi.specify_chip_select(node.chip_select.get());

                        self.spi.set_clock(cpol);
                        self.spi.set_phase(cpal);
                        self.spi.set_rate(rate);
                    }
                    Op::ReadWriteBytes(len) => {

                        node.txbuffer.take().map(|txbuffer| {
                            node.rxbuffer.take().map(move |rxbuffer| {
                                self.spi.read_write_bytes(txbuffer, rxbuffer, len);
                            });
                        });

                        // Only async operations want to block by setting the devices
                        // as inflight.
                        self.inflight.replace(node);
                    }
                    Op::Idle => {} // Can't get here...
                }
                node.operation.set(Op::Idle);
            });
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Op {
    Idle,
    Configure(hil::spi::ClockPolarity, hil::spi::ClockPhase, u32),
    ReadWriteBytes(usize),
}

pub struct SPIMasterDevice<'a, SPI: hil::spi::SpiMaster + 'a> {
    mux: &'a MuxSPIMaster<'a, SPI>,
    chip_select: Cell<SPI::ChipSelect>,
    txbuffer: TakeCell<&'static mut [u8]>,
    rxbuffer: TakeCell<Option<&'static mut [u8]>>,
    operation: Cell<Op>,
    next: ListLink<'a, SPIMasterDevice<'a, SPI>>,
    client: Cell<Option<&'a hil::spi::SpiMasterClient>>,
}

impl<'a, SPI: hil::spi::SpiMaster> SPIMasterDevice<'a, SPI> {
    pub const fn new(mux: &'a MuxSPIMaster<'a, SPI>,
                     chip_select: SPI::ChipSelect)
                     -> SPIMasterDevice<'a, SPI> {
        SPIMasterDevice {
            mux: mux,
            chip_select: Cell::new(chip_select),
            txbuffer: TakeCell::empty(),
            rxbuffer: TakeCell::empty(),
            operation: Cell::new(Op::Idle),
            next: ListLink::empty(),
            client: Cell::new(None),
        }
    }

    pub fn set_client(&'a self, client: &'a hil::spi::SpiMasterClient) {
        self.mux.devices.push_head(self);
        self.client.set(Some(client));
    }
}

impl<'a, SPI: hil::spi::SpiMaster> hil::spi::SpiMasterClient for SPIMasterDevice<'a, SPI> {
    fn read_write_done(&self,
                       write_buffer: &'static mut [u8],
                       read_buffer: Option<&'static mut [u8]>,
                       len: usize) {
        self.client.get().map(move |client| {
            client.read_write_done(write_buffer, read_buffer, len);
        });
    }
}

impl<'a, SPI: hil::spi::SpiMaster> ListNode<'a, SPIMasterDevice<'a, SPI>>
    for SPIMasterDevice<'a, SPI> {
    fn next(&'a self) -> &'a ListLink<'a, SPIMasterDevice<'a, SPI>> {
        &self.next
    }
}

impl<'a, SPI: hil::spi::SpiMaster> hil::spi::SPIMasterDevice for SPIMasterDevice<'a, SPI> {
    fn configure(&self, cpol: hil::spi::ClockPolarity, cpal: hil::spi::ClockPhase, rate: u32) {
        self.operation.set(Op::Configure(cpol, cpal, rate));
        self.mux.do_next_op();
    }

    fn read_write_bytes(&self,
                        write_buffer: &'static mut [u8],
                        read_buffer: Option<&'static mut [u8]>,
                        len: usize)
                        -> bool {
        self.txbuffer.replace(write_buffer);
        self.rxbuffer.replace(read_buffer);
        self.operation.set(Op::ReadWriteBytes(len));
        self.mux.do_next_op();

        true
    }
}
