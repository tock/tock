use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::common::StaticRef;
use kernel::hil::uart;
use kernel::ReturnCode;

use crate::event_manager::LiteXEventManager;
use crate::litex_registers::{
    register_bitfields, LiteXSoCRegisterConfiguration, Read, ReadRegWrapper, Write, WriteRegWrapper,
};

const EVENT_MANAGER_INDEX_TX: usize = 0;
const EVENT_MANAGER_INDEX_RX: usize = 1;

type LiteXUartEV<'a, R> = LiteXEventManager<
    'a,
    u8,
    <R as LiteXSoCRegisterConfiguration>::ReadOnly8,
    <R as LiteXSoCRegisterConfiguration>::ReadWrite8,
    <R as LiteXSoCRegisterConfiguration>::ReadWrite8,
>;

/// LiteX UART PHY registers
///
/// This is a separate register set, as it is not necessarily present
/// on every LiteX SoC with UART (e.g. a verilated simulation)
#[repr(C)]
pub struct LiteXUartPhyRegisters<R: LiteXSoCRegisterConfiguration> {
    /// Tuning word (UART baudrate)
    tuning_word: R::ReadWrite32,
}

/// LiteX UART registers
#[repr(C)]
pub struct LiteXUartRegisters<R: LiteXSoCRegisterConfiguration> {
    /// receive & transmit register
    rxtx: R::ReadWrite8,
    /// transmit buffer full
    txfull: R::ReadOnly8,
    /// receive buffer empty
    rxempty: R::ReadOnly8,
    /// LiteX EventManager status register
    ev_status: R::ReadOnly8,
    /// LiteX EventManager pending register
    ev_pending: R::ReadWrite8,
    /// LiteX EventManager pending register
    ev_enable: R::ReadWrite8,
    /// transmit buffer empty
    txempty: R::ReadOnly8,
    /// receive buffer full
    rxfull: R::ReadOnly8,
}

impl<R: LiteXSoCRegisterConfiguration> LiteXUartRegisters<R> {
    /// Create an event manager instance for the UART events
    fn ev<'a>(&'a self) -> LiteXUartEV<'a, R> {
        LiteXUartEV::<R>::new(&self.ev_status, &self.ev_pending, &self.ev_enable)
    }
}

register_bitfields![u8,
    rxtx [
        data OFFSET(0) NUMBITS(8) []
    ],
    txfull [
        full OFFSET(0) NUMBITS(1) []
    ],
    rxempty [
        empty OFFSET(0) NUMBITS(1) []
    ],
    txempty [
        empty OFFSET(0) NUMBITS(1) []
    ],
    rxfull [
        full OFFSET(0) NUMBITS(1) []
    ]
];

pub struct LiteXUart<'a, R: LiteXSoCRegisterConfiguration> {
    uart_regs: StaticRef<LiteXUartRegisters<R>>,
    phy: Option<(StaticRef<LiteXUartPhyRegisters<R>>, u32)>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_progress: Cell<usize>,
    tx_aborted: Cell<bool>,
    tx_deferred_call: Cell<bool>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_progress: Cell<usize>,
    rx_aborted: Cell<bool>,
    rx_deferred_call: Cell<bool>,
    deferred_caller: &'static DynamicDeferredCall,
    deferred_handle: OptionalCell<DeferredCallHandle>,
}

impl<'a, R: LiteXSoCRegisterConfiguration> LiteXUart<'a, R> {
    pub const fn new(
        uart_base: StaticRef<LiteXUartRegisters<R>>,
        phy_args: Option<(StaticRef<LiteXUartPhyRegisters<R>>, u32)>,
        deferred_caller: &'static DynamicDeferredCall,
    ) -> LiteXUart<'a, R> {
        LiteXUart {
            uart_regs: uart_base,
            phy: phy_args,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_progress: Cell::new(0),
            tx_aborted: Cell::new(false),
            tx_deferred_call: Cell::new(false),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_progress: Cell::new(0),
            rx_aborted: Cell::new(false),
            rx_deferred_call: Cell::new(false),
            deferred_caller,
            deferred_handle: OptionalCell::empty(),
        }
    }

    pub fn initialize(&self, deferred_call_handle: DeferredCallHandle) {
        self.uart_regs.ev().disable_all();
        self.deferred_handle.set(deferred_call_handle);
    }

    pub fn transmit_sync(&self, bytes: &[u8]) {
        // We need to make sure that we're not modifying interrupt
        // pending and enabled bits here!
        let regs = self.uart_regs;
        let ev = regs.ev();

        // Store whether there was a pending interrupt before and
        // whether interrupts were enabled, and if we cause one, clear
        // it after waiting until the buffer has space again.
        let interrupt_pending = ev.event_pending(EVENT_MANAGER_INDEX_TX);
        let interrupt_enabled = ev.event_enabled(EVENT_MANAGER_INDEX_TX);
        ev.disable_event(EVENT_MANAGER_INDEX_TX);

        for b in bytes.iter() {
            while ReadRegWrapper::wrap(&regs.txfull).is_set(txfull::full) {}
            WriteRegWrapper::wrap(&regs.rxtx).write(rxtx::data.val(*b));
        }

        // Wait until there is space for at least one byte
        while ReadRegWrapper::wrap(&regs.txfull).is_set(txfull::full) {}

        // Check if we generated an additional event and clear it
        if !interrupt_pending && ev.event_pending(EVENT_MANAGER_INDEX_TX) {
            ev.clear_event(EVENT_MANAGER_INDEX_TX);
        }

        // Check if interrupts were previously enabled and reenable in that case
        if interrupt_enabled {
            ev.enable_event(EVENT_MANAGER_INDEX_TX);
        }
    }

    pub fn service_interrupt(&self) {
        let ev = self.uart_regs.ev();

        // We need to mask the events depending on their "enable"
        // state to avoid falsely calling an interrupt handler which
        // shouldn't fire

        if ev.event_enabled(EVENT_MANAGER_INDEX_RX) && ev.event_pending(EVENT_MANAGER_INDEX_RX) {
            self.rx_data();
        }

        if ev.event_enabled(EVENT_MANAGER_INDEX_TX) && ev.event_pending(EVENT_MANAGER_INDEX_TX) {
            ev.clear_event(EVENT_MANAGER_INDEX_TX);
            self.resume_tx();
        }
    }

    fn deferred_rx_abort(&self) {
        // The RX event has already been disabled
        // Just return the buffer to the client
        let buffer = self.rx_buffer.take().expect("no rx buffer");
        let progress = self.rx_progress.get();

        self.rx_client.map(move |client| {
            client.received_buffer(buffer, progress, ReturnCode::ECANCEL, uart::Error::None)
        });
    }

    fn rx_data(&self) {
        // New data is available for reception
        let ev = self.uart_regs.ev();
        let buffer = self.rx_buffer.take().expect("no rx buffer");
        let len = self.rx_len.get();
        let mut progress = self.rx_progress.get();

        // Read all available data, until we've reached the length limit
        while {
            !ReadRegWrapper::wrap(&self.uart_regs.rxempty).is_set(rxempty::empty) && progress < len
        } {
            buffer[progress] = ReadRegWrapper::wrap(&self.uart_regs.rxtx).read(rxtx::data);
            progress += 1;

            // Mark the byte as read by acknowledging the event
            ev.clear_event(EVENT_MANAGER_INDEX_RX);
        }

        // Check whether we've reached the length limit and call to
        // the client respectively
        if progress == len {
            // Disable RX events
            self.uart_regs.ev().disable_event(EVENT_MANAGER_INDEX_RX);
            self.rx_client.map(move |client| {
                client.received_buffer(buffer, len, ReturnCode::SUCCESS, uart::Error::None)
            });
        } else {
            self.rx_buffer.replace(buffer);
            self.rx_progress.set(progress);
        }
    }

    fn deferred_tx_abort(&self) {
        // The TX event has already been disabled
        // Just return the buffer to the client
        let buffer = self.tx_buffer.take().expect("no tx buffer");
        let progress = self.tx_progress.get();

        self.tx_client
            .map(move |client| client.transmitted_buffer(buffer, progress, ReturnCode::ECANCEL));
    }

    // This is either called as a deferred call or by a
    // hardware-generated interrupt, hence it is guaranteed to be an
    // upcall
    fn resume_tx(&self) {
        let len = self.tx_len.get();
        let mut progress = self.tx_progress.get();
        let buffer = self.tx_buffer.take().expect("no tx buffer");

        // Try to transmit any remaining data

        let mut fifo_full: bool; // Store this to check whether we will get another interrupt
        while {
            fifo_full = ReadRegWrapper::wrap(&self.uart_regs.txfull).is_set(txfull::full);
            !fifo_full && progress < len
        } {
            WriteRegWrapper::wrap(&self.uart_regs.rxtx).write(rxtx::data.val(buffer[progress]));
            progress += 1;
        }

        if progress < len {
            // If we haven't transmitted all data, we _must_ have
            // reached the fifo-limit
            assert!(fifo_full == true);

            // Place all information and buffers back for the next
            // call to `resume_tx`
            self.tx_progress.set(progress);
            self.tx_buffer.replace(buffer);
        } else if fifo_full {
            // All data is transmitted, but an interrupt will still be
            // generated, for which we wait

            // Place all information and buffers back for the next
            // call to `resume_tx`
            self.tx_progress.set(progress);
            self.tx_buffer.replace(buffer);
        } else {
            // All data is transmitted and we will get no further
            // interrupt
            //
            // Disable TX events until the next transmission and call back to the client
            self.uart_regs.ev().disable_event(EVENT_MANAGER_INDEX_TX);
            self.tx_client
                .map(move |client| client.transmitted_buffer(buffer, len, ReturnCode::SUCCESS));
        }
    }
}

impl<R: LiteXSoCRegisterConfiguration> uart::Configure for LiteXUart<'_, R> {
    fn configure(&self, params: uart::Parameters) -> ReturnCode {
        // LiteX UART supports only
        // - a fixed with of 8 bits
        // - no parity
        // - 1 stop bit
        // - no hardware flow control(?)
        if let Some((ref phy_regs, system_clock)) = self.phy {
            if params.width != uart::Width::Eight
                || params.parity != uart::Parity::None
                || params.stop_bits != uart::StopBits::One
                || params.hw_flow_control == true
            {
                ReturnCode::ENOSUPPORT
            } else if params.baud_rate == 0 || params.baud_rate > system_clock {
                ReturnCode::EINVAL
            } else {
                let tuning_word = if params.baud_rate == system_clock {
                    u32::MAX
                } else {
                    (((params.baud_rate as u64) * (1 << 32)) / (system_clock as u64)) as u32
                };
                phy_regs.tuning_word.set(tuning_word);

                ReturnCode::SUCCESS
            }
        } else {
            ReturnCode::ENOSUPPORT
        }
    }
}

impl<'a, R: LiteXSoCRegisterConfiguration> uart::Transmit<'a> for LiteXUart<'a, R> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        // Make sure the UART is initialized
        assert!(self.deferred_handle.is_some());

        if tx_buffer.len() < tx_len {
            return (ReturnCode::ESIZE, Some(tx_buffer));
        }

        if self.tx_buffer.is_some() {
            return (ReturnCode::EBUSY, Some(tx_buffer));
        }

        // Enable TX events (interrupts)
        self.uart_regs.ev().clear_event(EVENT_MANAGER_INDEX_TX);
        self.uart_regs.ev().enable_event(EVENT_MANAGER_INDEX_TX);

        // Try to send the buffer
        //
        // If it does not fill the FIFO, an
        // interrupt will _not_ be generated and hence we have to
        // perform the upcall using a deferred call.
        //
        // If we fill up the FIFO, an interrupt _will_ be
        // generated. We can transmit the rest using `resume_tx` and
        // directly call the callback there, as we are guaranteed to
        // be in an upcall.
        let mut fifo_full: bool;
        let mut progress: usize = 0;
        while {
            fifo_full = ReadRegWrapper::wrap(&self.uart_regs.txfull).is_set(txfull::full);
            (progress < tx_len) && !fifo_full
        } {
            WriteRegWrapper::wrap(&self.uart_regs.rxtx).write(rxtx::data.val(tx_buffer[progress]));
            progress += 1;
        }

        // Store the respective values (implicitly setting the device as busy)
        self.tx_progress.set(progress);
        self.tx_len.set(tx_len);
        self.tx_buffer.replace(tx_buffer);
        self.tx_aborted.set(false);

        // If we did not reach the fifo-limit, the entire buffer
        // _must_ have been written to the device
        //
        // In this case, we must request a deferred call for the
        // upcall
        if !fifo_full {
            assert!(progress == tx_len);

            self.tx_deferred_call.set(true);
            self.deferred_handle
                .map(|handle| self.deferred_caller.set(*handle));
        }

        // If fifo_full == true, we will get an interrupt

        (ReturnCode::SUCCESS, None)
    }

    fn transmit_word(&self, _word: u32) -> ReturnCode {
        // Make sure the UART is initialized
        assert!(self.deferred_handle.is_some());

        ReturnCode::FAIL
    }

    fn transmit_abort(&self) -> ReturnCode {
        // Disable TX events
        //
        // A deferred call might still be pending from the started
        // transmission, however that will be routed to
        // `deferred_tx_abort` if `tx_aborted` is set

        // Make sure the UART is initialized
        assert!(self.deferred_handle.is_some());

        self.uart_regs.ev().disable_event(EVENT_MANAGER_INDEX_TX);

        if self.tx_buffer.is_some() {
            self.tx_aborted.set(true);
            self.tx_deferred_call.set(true);
            self.deferred_handle
                .map(|handle| self.deferred_caller.set(*handle));

            ReturnCode::EBUSY
        } else {
            ReturnCode::SUCCESS
        }
    }
}

impl<'a, R: LiteXSoCRegisterConfiguration> uart::Receive<'a> for LiteXUart<'a, R> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        // Make sure the UART is initialized
        assert!(self.deferred_handle.is_some());

        if rx_len > rx_buffer.len() {
            return (ReturnCode::ESIZE, Some(rx_buffer));
        }

        if self.rx_buffer.is_some() {
            return (ReturnCode::EBUSY, Some(rx_buffer));
        }

        // Store the slice and length for receiving, set the progress
        // to 0
        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);
        self.rx_progress.set(0);
        self.rx_aborted.set(false);

        // If there is already data in the FIFO but the event is not
        // pending (has been cleared), request a deferred call,
        // otherwise rely on the interrupts
        //
        // This is required as the EventSourceProcess only triggers on
        // a falling edge, which will not happen if the FIFO had valid
        // data left over from the previous transaction.
        if !ReadRegWrapper::wrap(&self.uart_regs.rxempty).is_set(rxempty::empty)
            && !self.uart_regs.ev().event_pending(EVENT_MANAGER_INDEX_RX)
        {
            // We do not enable interrupts just yet, but rely on a
            // deferred call for the bytes left over from a previous
            // transaction in the FIFO
            //
            // Enable the event interrupt in the deferred callback
            // instead! Otherwise we risk double-delivery of the
            // interrupt _and_ the deferred call
            self.rx_deferred_call.set(true);
            self.deferred_handle
                .map(|handle| self.deferred_caller.set(*handle));
        } else {
            // We do _not_ clear any pending data in the FIFO by
            // acknowledging previous events
            self.uart_regs.ev().enable_event(EVENT_MANAGER_INDEX_RX);
        }

        (ReturnCode::SUCCESS, None)
    }

    fn receive_word(&self) -> ReturnCode {
        // Make sure the UART is initialized
        assert!(self.deferred_handle.is_some());
        ReturnCode::FAIL
    }

    fn receive_abort(&self) -> ReturnCode {
        // Make sure the UART is initialized
        assert!(self.deferred_handle.is_some());

        // Disable RX events
        self.uart_regs.ev().disable_event(EVENT_MANAGER_INDEX_RX);

        if self.rx_buffer.is_some() {
            // Set the UART transmission to aborted and request a deferred
            // call
            self.rx_aborted.set(true);
            self.rx_deferred_call.set(true);
            self.deferred_handle
                .map(|handle| self.deferred_caller.set(*handle));

            ReturnCode::EBUSY
        } else {
            ReturnCode::SUCCESS
        }
    }
}

impl<'a, R: LiteXSoCRegisterConfiguration> uart::Uart<'a> for LiteXUart<'a, R> {}
impl<'a, R: LiteXSoCRegisterConfiguration> uart::UartData<'a> for LiteXUart<'a, R> {}

impl<'a, R: LiteXSoCRegisterConfiguration> DynamicDeferredCallClient for LiteXUart<'a, R> {
    fn call(&self, _handle: DeferredCallHandle) {
        // Are we currently in a TX or RX transaction?
        if self.tx_deferred_call.get() {
            self.tx_deferred_call.set(false);
            // Has the transmission been aborted?
            if self.tx_aborted.get() {
                self.deferred_tx_abort();
            } else {
                // The buffer has been completely transmitted in the initial
                // `transmit_buffer` call, finish the operation
                self.resume_tx();
            }
        }

        if self.rx_deferred_call.get() {
            self.rx_deferred_call.set(false);
            // Has the reception been aborted?
            if self.rx_aborted.get() {
                self.deferred_rx_abort();
            } else {
                // The deferred call is used as there is some leftover
                // data in the FIFO from a previous transaction, which
                // won't trigger the falling-edge based
                // EventSourceProcess
                //
                // We need to instead enable interrupts here (can't be
                // done in the original receive_buffer method, as that
                // would risk double-delivery of interrupts and
                // deferred calls)
                self.uart_regs.ev().enable_event(EVENT_MANAGER_INDEX_RX);
                self.rx_data();
            }
        }
    }
}
