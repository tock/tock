use kernel::common::{List, ListLink, ListNode};
use kernel::common::take_cell::TakeCell;
use core::cell::Cell;
use kernel::hil;

/// The Mux struct manages multiple SPI clients. Each client may have
/// at most one outstanding SPI request.
pub struct MuxSPIMaster<'a> {
    spi: &'a hil::spi::SpiMaster,
    devices: List<'a, SPIMasterDevice<'a>>,
    inflight: TakeCell<&'a SPIMasterDevice<'a>>,
}

impl<'a> hil::spi::SpiMasterClient for MuxSPIMaster<'a> {
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

impl<'a> MuxSPIMaster<'a> {
    pub const fn new(spi: &'a hil::spi::SpiMaster) -> MuxSPIMaster<'a> {
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

                        match node.chip_select {
                            Some(x) => {
                                self.spi.set_chip_select(x);
                            }
                            None => {}
                        }

                        // In theory, the SPI interface should support
                        // using a GPIO in lieu of a hardware CS line.
                        // This is particularly important for the SAM4L
                        // if using a USART, but might be relevant
                        // for other platforms as well.
                        // TODO: make this do something if given GPIO pin
                        // match node.chip_select_gpio {
                        //     Some() => { },
                        //     None => {}
                        // }

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

pub struct SPIMasterDevice<'a> {
    mux: &'a MuxSPIMaster<'a>,
    chip_select: Option<u8>,
    chip_select_gpio: Option<&'static hil::gpio::GPIOPin>,
    txbuffer: TakeCell<&'static mut [u8]>,
    rxbuffer: TakeCell<Option<&'static mut [u8]>>,
    operation: Cell<Op>,
    next: ListLink<'a, SPIMasterDevice<'a>>,
    client: Cell<Option<&'a hil::spi::SpiMasterClient>>,
}

impl<'a> SPIMasterDevice<'a> {
    pub const fn new(mux: &'a MuxSPIMaster<'a>,
                     chip_select: Option<u8>,
                     chip_select_gpio: Option<&'static hil::gpio::GPIOPin>)
                     -> SPIMasterDevice<'a> {
        SPIMasterDevice {
            mux: mux,
            chip_select: chip_select,
            chip_select_gpio: chip_select_gpio,
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

impl<'a> hil::spi::SpiMasterClient for SPIMasterDevice<'a> {
    fn read_write_done(&self,
                       write_buffer: &'static mut [u8],
                       read_buffer: Option<&'static mut [u8]>,
                       len: usize) {
        self.client.get().map(move |client| {
            client.read_write_done(write_buffer, read_buffer, len);
        });
    }
}

impl<'a> ListNode<'a, SPIMasterDevice<'a>> for SPIMasterDevice<'a> {
    fn next(&'a self) -> &'a ListLink<'a, SPIMasterDevice<'a>> {
        &self.next
    }
}

impl<'a> hil::spi::SPIMasterDevice for SPIMasterDevice<'a> {
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
