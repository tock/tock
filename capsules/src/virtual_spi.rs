//! Virtualize a Spi Master bus to enable multiple users of the Spi bus.

use core::cell::Cell;
use kernel::common::{List, ListLink, ListNode};
use kernel::common::take_cell::TakeCell;
use kernel::hil;

/// The Mux struct manages multiple Spi clients. Each client may have
/// at most one outstanding Spi request.
pub struct MuxSpiMaster<'a, Spi: hil::spi::SpiMaster + 'a> {
    spi: &'a Spi,
    devices: List<'a, SpiMasterDevice<'a, Spi>>,
    inflight: TakeCell<&'a SpiMasterDevice<'a, Spi>>,
}

impl<'a, Spi: hil::spi::SpiMaster> hil::spi::SpiMasterClient for MuxSpiMaster<'a, Spi> {
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

impl<'a, Spi: hil::spi::SpiMaster> MuxSpiMaster<'a, Spi> {
    pub const fn new(spi: &'a Spi) -> MuxSpiMaster<'a, Spi> {
        MuxSpiMaster {
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
                    Op::SetPolarity(pol) => {
                        self.spi.set_clock(pol);
                    }
                    Op::SetPhase(pal) => {
                        self.spi.set_phase(pal);
                    }
                    Op::SetRate(rate) => {
                        self.spi.set_rate(rate);
                    }
                    Op::SetChipSelect(cs) => {
                        self.spi.specify_chip_select(cs);
                    }
                    Op::Idle => {} // Can't get here...
                }
                node.operation.set(Op::Idle);
            });
        }
    }
}

#[derive(Copy, Clone)]
enum Op<CS> {
    Idle,
    Configure(hil::spi::ClockPolarity, hil::spi::ClockPhase, u32),
    ReadWriteBytes(usize),
    SetPolarity(hil::spi::ClockPolarity),
    SetPhase(hil::spi::ClockPhase),
    SetRate(u32),
    SetChipSelect(CS),
}

impl<CS> PartialEq for Op<CS> {
    fn eq(&self, other: &Op<CS>) -> bool {
        match (self, other) {
            (&Op::Idle, &Op::Idle) => true,
             _ => false,
        }
    }
}

pub struct SpiMasterDevice<'a, Spi: hil::spi::SpiMaster + 'a> {
    mux: &'a MuxSpiMaster<'a, Spi>,
    chip_select: Cell<Spi::ChipSelect>,
    txbuffer: TakeCell<&'static mut [u8]>,
    rxbuffer: TakeCell<Option<&'static mut [u8]>>,
    operation: Cell<Op<Spi::ChipSelect>>,
    next: ListLink<'a, SpiMasterDevice<'a, Spi>>,
    client: Cell<Option<&'a hil::spi::SpiMasterClient>>,
}

impl<'a, Spi: hil::spi::SpiMaster> SpiMasterDevice<'a, Spi> {
    pub const fn new(mux: &'a MuxSpiMaster<'a, Spi>,
                     chip_select: Spi::ChipSelect)
                     -> SpiMasterDevice<'a, Spi> {
        SpiMasterDevice {
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

    // Binding virtualization and configuration causes problems,
    // because it implies that a virtualized client never wants to
    // use more than one chip select line. Counter-example is the
    // system call driver. It's good to have a default, but
    // we also need to be able to reconfigure.
    pub fn set_chip_select(&'a self, cs: Spi::ChipSelect) {
        self.chip_select.set(cs);
    }
}

impl<'a, Spi: hil::spi::SpiMaster> hil::spi::SpiMasterClient for SpiMasterDevice<'a, Spi> {
    fn read_write_done(&self,
                       write_buffer: &'static mut [u8],
                       read_buffer: Option<&'static mut [u8]>,
                       len: usize) {
        self.client.get().map(move |client| {
            client.read_write_done(write_buffer, read_buffer, len);
        });
    }
}

impl<'a, Spi: hil::spi::SpiMaster> ListNode<'a, SpiMasterDevice<'a, Spi>>
    for SpiMasterDevice<'a, Spi> {
    fn next(&'a self) -> &'a ListLink<'a, SpiMasterDevice<'a, Spi>> {
        &self.next
    }
}

impl<'a, Spi: hil::spi::SpiMaster> hil::spi::SpiMasterDevice for SpiMasterDevice<'a, Spi> {
    type ChipSelect = Spi::ChipSelect;

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

    fn set_polarity(&self, cpol: hil::spi::ClockPolarity) {
        self.operation.set(Op::SetPolarity(cpol));
        self.mux.do_next_op();
    }

    fn set_phase(&self, cpal: hil::spi::ClockPhase) {
        self.operation.set(Op::SetPhase(cpal));
        self.mux.do_next_op();
    }

    fn set_rate(&self, rate: u32) {
        self.operation.set(Op::SetRate(rate));
        self.mux.do_next_op();
    }

    fn get_polarity(&self) -> hil::spi::ClockPolarity {
        hil::spi::ClockPolarity::IdleLow
    }

    fn get_phase(&self) -> hil::spi::ClockPhase {
        hil::spi::ClockPhase::SampleLeading
    }

    fn get_rate(&self) -> u32 {
        0
    }

    fn set_chip_select(&self, cs: Self::ChipSelect) {
        self.operation.set(Op::SetChipSelect(cs));
        self.mux.do_next_op();
    }
}
