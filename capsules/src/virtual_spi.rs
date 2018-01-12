//! Virtualize a SPI master bus to enable multiple users of the SPI bus.

use core::cell::Cell;
use kernel::ReturnCode;
use kernel::common::{List, ListLink, ListNode};
use kernel::common::take_cell::TakeCell;
use kernel::hil;

/// The Mux struct manages multiple Spi clients. Each client may have
/// at most one outstanding Spi request.
pub struct MuxSpiMaster<'a, Spi: hil::spi::SpiMaster + 'a> {
    spi: &'a Spi,
    devices: List<'a, VirtualSpiMasterDevice<'a, Spi>>,
    inflight: Cell<Option<&'a VirtualSpiMasterDevice<'a, Spi>>>,
}

impl<'a, Spi: hil::spi::SpiMaster> hil::spi::SpiMasterClient for MuxSpiMaster<'a, Spi> {
    fn read_write_done(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) {
        self.inflight.get().map(move |device| {
            self.inflight.set(None);
            self.do_next_op();
            device.read_write_done(write_buffer, read_buffer, len);
        });
    }
}

impl<'a, Spi: hil::spi::SpiMaster> MuxSpiMaster<'a, Spi> {
    pub const fn new(spi: &'a Spi) -> MuxSpiMaster<'a, Spi> {
        MuxSpiMaster {
            spi: spi,
            devices: List::new(),
            inflight: Cell::new(None),
        }
    }

    fn do_next_op(&self) {
        if self.inflight.get().is_none() {
            let mnode = self.devices
                .iter()
                .find(|node| node.operation.get() != Op::Idle);
            mnode.map(|node| {
                self.spi.specify_chip_select(node.chip_select.get());
                let op = node.operation.get();
                // Need to set idle here in case callback changes state
                node.operation.set(Op::Idle);
                match op {
                    Op::Configure(cpol, cpal, rate) => {
                        // The `chip_select` type will be correct based on
                        // what implemented `SpiMaster`.
                        self.spi.set_clock(cpol);
                        self.spi.set_phase(cpal);
                        self.spi.set_rate(rate);
                    }
                    Op::ReadWriteBytes(len) => {
                        // Only async operations want to block by setting
                        // the devices as inflight.
                        self.inflight.set(Some(node));
                        node.txbuffer.take().map(|txbuffer| {
                            let rxbuffer = node.rxbuffer.take();
                            self.spi.read_write_bytes(txbuffer, rxbuffer, len);
                        });
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
                    Op::Idle => {} // Can't get here...
                }
            });
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Op {
    Idle,
    Configure(hil::spi::ClockPolarity, hil::spi::ClockPhase, u32),
    ReadWriteBytes(usize),
    SetPolarity(hil::spi::ClockPolarity),
    SetPhase(hil::spi::ClockPhase),
    SetRate(u32),
}

pub struct VirtualSpiMasterDevice<'a, Spi: hil::spi::SpiMaster + 'a> {
    mux: &'a MuxSpiMaster<'a, Spi>,
    chip_select: Cell<Spi::ChipSelect>,
    txbuffer: TakeCell<'static, [u8]>,
    rxbuffer: TakeCell<'static, [u8]>,
    operation: Cell<Op>,
    next: ListLink<'a, VirtualSpiMasterDevice<'a, Spi>>,
    client: Cell<Option<&'a hil::spi::SpiMasterClient>>,
}

impl<'a, Spi: hil::spi::SpiMaster> VirtualSpiMasterDevice<'a, Spi> {
    pub const fn new(
        mux: &'a MuxSpiMaster<'a, Spi>,
        chip_select: Spi::ChipSelect,
    ) -> VirtualSpiMasterDevice<'a, Spi> {
        VirtualSpiMasterDevice {
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

impl<'a, Spi: hil::spi::SpiMaster> hil::spi::SpiMasterClient for VirtualSpiMasterDevice<'a, Spi> {
    fn read_write_done(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) {
        self.client.get().map(move |client| {
            client.read_write_done(write_buffer, read_buffer, len);
        });
    }
}

impl<'a, Spi: hil::spi::SpiMaster> ListNode<'a, VirtualSpiMasterDevice<'a, Spi>>
    for VirtualSpiMasterDevice<'a, Spi> {
    fn next(&'a self) -> &'a ListLink<'a, VirtualSpiMasterDevice<'a, Spi>> {
        &self.next
    }
}

impl<'a, Spi: hil::spi::SpiMaster> hil::spi::SpiMasterDevice for VirtualSpiMasterDevice<'a, Spi> {
    fn configure(&self, cpol: hil::spi::ClockPolarity, cpal: hil::spi::ClockPhase, rate: u32) {
        self.operation.set(Op::Configure(cpol, cpal, rate));
        self.mux.do_next_op();
    }

    fn read_write_bytes(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> ReturnCode {
        self.txbuffer.replace(write_buffer);
        self.rxbuffer.put(read_buffer);
        self.operation.set(Op::ReadWriteBytes(len));
        self.mux.do_next_op();
        ReturnCode::SUCCESS
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
        self.mux.spi.get_clock()
    }

    fn get_phase(&self) -> hil::spi::ClockPhase {
        self.mux.spi.get_phase()
    }

    fn get_rate(&self) -> u32 {
        self.mux.spi.get_rate()
    }
}

pub struct VirtualSpiSlaveDevice<'a, Spi: hil::spi::SpiSlave + 'a> {
    spi: &'a Spi,
    client: Cell<Option<&'a hil::spi::SpiSlaveClient>>,
}

impl<'a, Spi: hil::spi::SpiSlave> VirtualSpiSlaveDevice<'a, Spi> {
    pub const fn new(spi: &'a Spi) -> VirtualSpiSlaveDevice<'a, Spi> {
        VirtualSpiSlaveDevice {
            spi: spi,
            client: Cell::new(None),
        }
    }

    pub fn set_client(&'a self, client: &'a hil::spi::SpiSlaveClient) {
        self.client.set(Some(client));
    }
}

impl<'a, Spi: hil::spi::SpiSlave> hil::spi::SpiSlaveClient for VirtualSpiSlaveDevice<'a, Spi> {
    fn read_write_done(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) {
        self.client.get().map(move |client| {
            client.read_write_done(write_buffer, read_buffer, len);
        });
    }

    fn chip_selected(&self) {
        self.client.get().map(move |client| {
            client.chip_selected();
        });
    }
}

impl<'a, Spi: hil::spi::SpiSlave> hil::spi::SpiSlaveDevice for VirtualSpiSlaveDevice<'a, Spi> {
    fn configure(&self, cpol: hil::spi::ClockPolarity, cpal: hil::spi::ClockPhase) {
        self.spi.set_clock(cpol);
        self.spi.set_phase(cpal);
    }

    fn read_write_bytes(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> ReturnCode {
        self.spi.read_write_bytes(write_buffer, read_buffer, len)
    }

    fn set_polarity(&self, cpol: hil::spi::ClockPolarity) {
        self.spi.set_clock(cpol);
    }

    fn set_phase(&self, cpal: hil::spi::ClockPhase) {
        self.spi.set_phase(cpal);
    }

    fn get_polarity(&self) -> hil::spi::ClockPolarity {
        self.spi.get_clock()
    }

    fn get_phase(&self) -> hil::spi::ClockPhase {
        self.spi.get_phase()
    }
}
