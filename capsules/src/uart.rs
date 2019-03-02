use kernel::common::cells::{OptionalCell, TakeCell, MapCell};

use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::CONSOLE as usize;

use kernel::ikc::DriverState::{BUSY, IDLE};
use kernel::ikc;

pub type AppRequest = ikc::AppRequest<u8>;

pub fn handle_irq(num: usize, driver: &UartDriver<'a>, clients: Option<&[&'a hil::uart::Client<'a>]>) {
    driver.uart[num].state.map( |state| {
        // pass a copy of state to the HIL's handle interrupt routine
        // it will return completed requests if there are any
        let (tx_complete, rx_complete) = driver.uart[num].handle_interrupt(*state);

        // if we have receive a completed transmit, then we need to handle it
        if let Some(request) = tx_complete {
            // set default state transition to IDLE
            state.tx = IDLE;

            // if there is some app_id, then the it is app tx request
            if let Some(app_id) = driver.uart[num].app_requests.tx_in_progress.take() {
                // put back the driver's app request memory
                driver.uart[num].app_requests.tx.put(request);
                // update the app tx request; if it returns something then
                // same app request still has data
                if let Some(app_id) = driver.uart[num].app_tx_update(app_id){
                    driver.transmit_app_tx_request(num, app_id);
                    // undo IDLE transition, we are in fact busy
                    state.tx = BUSY;
                }
            }
            // otherwise, it is a kernel client request 
            else if let Some(clients) = clients {
                // use client callback
                let client_id = request.client_id;
                clients[client_id].tx_request_complete(num, request);
            }
        }

        // if we have receive a completed receive, then we need to handle it
        if let Some(request) = rx_complete {
            // set default state transition to IDLE
            state.rx = IDLE;
            // give the transaction to the driver level for muxing out received bytes to other buffers
            let request = driver.uart[num].mux_completed_rx_to_others(request);

            // if there is some app_id, then the it is app tx request
            if let Some(_app_id) = driver.uart[num].app_requests.rx_in_progress.take() {
                // put back the driver's app request memory
                driver.uart[num].app_requests.rx.put(request);
                // no need to write out the data since mux_completed does that already
            }
            // otherwise, it is a kernel client request 
            else if let Some(clients) = clients {
                let client_id = request.client_id;
                clients[client_id].rx_request_complete(num, request);

                // if the muxing out completed any other rx'es, return them as well
                while let Some(next_request) = driver.uart[num].get_other_completed_rx() {
                    let client_id = next_request.client_id;
                    clients[client_id].rx_request_complete(num, next_request);
                }
            }
        }

        // // Dispatch new requests only after both TX/RX completed have been handled
        // TX'es are dispatched one by one, so only take it if we are ready for another one
        if state.tx == IDLE {
            // Check if Kernel clients have app request
            if let Some(clients) = clients {
                if dispatch_next_tx_request(num, driver, clients){
                    state.tx = BUSY;
                } 
            }
        }

        // If no kernel clients needed to use UART, 
        // check for pending application requests
        if state.tx == IDLE {
            if let Some(appid) = driver.pending_app_tx_request(num){
                driver.transmit_app_tx_request(num, appid);
                state.tx = BUSY;
            }
        }

        if let Some(clients) = clients {
            // Each client can have one (and only one) pending RX concurrently with other clients
            // so take any new ones that have occured this go-around
            take_new_rx_requests(num, driver, clients);
        }

        // If a request completed, dispatch the shortest pending request (if there is one)
        if state.rx == IDLE {
            if driver.uart[num].dispatch_shortest_rx_request(){
                state.rx = BUSY;
            }
        }
    });

}

fn dispatch_next_tx_request<'a>(
    num: usize,
    driver: &UartDriver<'a>,
    clients: &[&'a hil::uart::Client<'a>],
) -> bool {
    for index in 0..clients.len() {
        let client = clients[index];
        if client.has_tx_request() {
            if let Some(tx) = client.get_tx_request() {
                tx.client_id = index;
                driver.handle_tx_request(num, tx);
                return true;
            }
        }
    }
    false
}

fn take_new_rx_requests<'a>(
    num: usize,
    driver: &UartDriver<'a>,
    clients: &[&'a hil::uart::Client<'a>],
) {
    for index in 0..clients.len() {
        let client = clients[index];
        if client.has_rx_request() {
            if let Some(rx) = client.get_rx_request() {
                rx.client_id = index;
                driver.uart[num].stash_rx_request(rx);
            }
        }
    }
}

#[derive(Default)]
pub struct App {
    tx: AppRequest,
    rx: AppRequest,
}

pub struct Uart<'a> {
    // uart peripheral that this item is responsible for
    uart: &'a hil::uart::UartPeripheral<'a>,
    state: MapCell<hil::uart::PeripheralState>,
    // slots of each intrakernel client
    rx_requests: Option<&'a [TakeCell<'a, hil::uart::RxRequest<'a>>]>,
    // space for copying requests from Apps before dispatching to UART HIL
    app_requests: AppRequestsInProgress<'a>,
    // app grant providing space fo app clients
    apps: Grant<App>,
}

pub struct AppRequestsInProgress<'a> {
    tx_in_progress: OptionalCell<AppId>,
    tx: MapCell<&'a mut hil::uart::TxRequest<'a>>,
    rx_in_progress: OptionalCell<AppId>,
    rx: MapCell<&'a mut hil::uart::RxRequest<'a>>,
}

impl<'a> AppRequestsInProgress<'a> {
    pub fn space() -> (
        [u8; 8],
        hil::uart::TxRequest<'a>,
        [u8; 8],
        hil::uart::RxRequest<'a>,
    ) {
        (
            [0; 8],
            hil::uart::TxRequest::new(),
            [0; 8],
            hil::uart::RxRequest::new(),
        )
    }

    pub fn new_with_default_space(
        space: &'a mut (
            [u8; 8],
            hil::uart::TxRequest<'a>,
            [u8; 8],
            hil::uart::RxRequest<'a>,
        ),
    ) -> AppRequestsInProgress<'a> {
        let (tx_request_buffer, tx_request, rx_request_buffer, rx_request) = space;

        Self::new(tx_request_buffer, tx_request, rx_request_buffer, rx_request)
    }

    pub fn new(
        tx_request_buffer: &'a mut [u8],
        tx_request: &'a mut kernel::ikc::TxRequest<'a, u8>,
        rx_request_buffer: &'a mut [u8],
        rx_request: &'a mut kernel::ikc::RxRequest<'a, u8>,
    ) -> AppRequestsInProgress<'a> {
        tx_request.set_with_mut_ref(tx_request_buffer);
        rx_request.set_buf(rx_request_buffer);

        AppRequestsInProgress {
            tx_in_progress: OptionalCell::empty(),
            tx: MapCell::new(tx_request),
            rx_in_progress: OptionalCell::empty(),
            rx: MapCell::new(rx_request),
        }
    }
}

pub struct UartDriver<'a> {
    pub uart: &'a [&'a Uart<'a>],
}

impl<'a> UartDriver<'a> {
    pub fn new(uarts: &'a [&'a Uart<'a>]) -> UartDriver<'a> {
        UartDriver { uart: uarts }
    }

    fn handle_tx_request(&self, uart_num: usize, tx: &'a mut hil::uart::TxRequest<'a>) {
        self.uart[uart_num].uart.transmit_buffer(tx);
    }

    fn pending_app_tx_request(&self, uart_num: usize) -> Option<AppId> {
        for app in self.uart[uart_num].apps.iter() {
            if let Some(app_id) = app.enter(|app, _| {
                if app.tx.remaining!=0 {
                    Some(app.appid())
                } else {
                    None
                }
            }){
                return Some(app_id)
            }
        }
        None
    }

    fn transmit_app_tx_request(&self, uart_num: usize, app_id: AppId) -> ReturnCode {
        if let Some(request) = self.uart[uart_num].app_requests.tx.take(){
            if let Err(_err) = self.uart[uart_num].apps.enter(app_id, |app, _| {
                request.reset();
                request.copy_from_app_request(&mut app.tx);
            }){ return  ReturnCode::FAIL }

            self.uart[uart_num].app_requests.tx_in_progress.set(app_id);
            self.uart[uart_num].uart.transmit_buffer(request)

        }
        else{
            //transmit_app_request invoked but no request_tx buffer available
            ReturnCode::FAIL
        }
    }
}

static DEFAULT_PARAMS: hil::uart::Parameters = hil::uart::Parameters {
    baud_rate: 115200, // baud rate in bit/s
    width: hil::uart::Width::Eight,
    parity: hil::uart::Parity::None,
    stop_bits: hil::uart::StopBits::One,
    hw_flow_control: false,
};

impl<'a> Uart<'a> {
    pub fn new(
        uart: &'a hil::uart::UartPeripheral<'a>,
        rx_requests: Option<&'a [TakeCell<'a, hil::uart::RxRequest<'a>>]>,
        app_requests: AppRequestsInProgress<'a>,
        grant: Grant<App>) -> Uart<'a> {

        uart.configure(DEFAULT_PARAMS);

        Uart {
            uart,
            state: MapCell::new(hil::uart::PeripheralState::new()),
            rx_requests,
            app_requests,
            apps: grant,
        }
    }
    
    fn app_tx_update(&self, app_id: AppId) -> Option<AppId>{
        self.apps.enter(app_id, |app, _| {
            // if app tx request has no data left
            if app.tx.remaining == 0 {
                // Enqueue the application callback
                let written = app.tx.len();
                app.tx.callback.map(|mut cb| {
                    cb.schedule(written, 0, 0);
                });
                None
            } else {
                // Otherwise, return app_id
                Some(app_id)
            }
        }).unwrap_or_else(|_err| None)
    }

    fn handle_interrupt(&self, state: hil::uart::PeripheralState) 
        -> (Option<&mut hil::uart::TxRequest<'a>>, Option<&mut hil::uart::RxRequest<'a>>) {
        self.uart.handle_interrupt(state)
    }

    fn stash_rx_request(&self, rx: &'a mut hil::uart::RxRequest<'a>){
        let index = rx.client_id;
        if let Some(requests_stash) = self.rx_requests {
            if let Some(_existing_request) = requests_stash[index].take() {
                panic!("Client #{} should not be making new request when request is already pending!", index)
            }
            else {
                requests_stash[index].put(Some(rx));
            }
        }
        else {
            panic!("UART has not been provisioned with space to store any client requests!")
        }
    }

    fn mux_completed_rx_to_others(&self, completed_rx: &'a mut hil::uart::RxRequest<'a>) -> &'a mut hil::uart::RxRequest<'a> {

        if let Some(requests_stash) = self.rx_requests {
            match &completed_rx.buf {
                ikc::RxBuf::MUT(buf) => {
                    // for every item in the compeleted_rx
                    for i in 0..completed_rx.items_pushed() {
                        let item = buf[i];
                        // copy it into any existing requests in the requests_stash
                        for j in 0..requests_stash.len() {
                            if let Some(request) = requests_stash[j].take() {
                                if request.has_room(){
                                    request.push(item);
                                }
                                requests_stash[j].put(Some(request));
                            }
                        }

                        // copy it into any app requests
                        for app in self.apps.iter() {
                            app.enter(|app, _| {
                                // push item if request is pending
                                if app.rx.remaining!=0 {
                                    app.rx.push(item);
                                }
                                // //enqueue callback if it's completed requested
                                if app.rx.remaining == 0 {
                                    // Enqueue the application callback
                                    let read = app.rx.len();
                                    app.rx.callback.map(|mut cb| {
                                        cb.schedule(From::from(ReturnCode::SUCCESS), read, 0);
                                    });
                                }
                            });
                        }
                    }


                },
                _ => panic!("A null buffer has become a completed request? It should have never been dispatched in the first place! Shame on console/uart.rs"),
            }
        }

        completed_rx
    }

    fn get_other_completed_rx(&self) -> Option<&'a mut hil::uart::RxRequest<'a>> {
        if let Some(requests_stash) = self.rx_requests {
            for i in 0..requests_stash.len() {
                if let Some(request) = requests_stash[i].take() {
                    if request.request_completed(){
                        return Some(request)
                    }
                    else{
                        requests_stash[i].put(Some(request));
                    }
                }
                else {

                }
            }
        }
        // no more completed rx
        None
    }

    #[allow(unused_assignments)]
    fn dispatch_shortest_rx_request(&self) -> bool {
        if let Some(requests_stash) = self.rx_requests {

            let mut min: Option<usize> = None;
            let mut appid: Option<AppId> = None;
            let mut min_index: usize = 0;

            for i in 0..requests_stash.len() {
                if let Some(request) = requests_stash[i].take() {
                    let request_remaining = request.request_remaining();

                    // if there is a minimum already, compare to see if this is shorter
                    if let Some(mut min) = min {
                        if request_remaining <  min {
                            min = request_remaining;
                            min_index  = i;
                        }
                    }
                    // otherwise, this is the min so far
                    else{
                        min = Some(request_remaining);
                        min_index  = i;
                    }
                    requests_stash[i].put(Some(request));
                }
            }

            // copy it into any app requests
            for app in self.apps.iter() {
                app.enter(|app, _| {

                    if app.rx.remaining > 0 {
                        // if there is a minimum already, compare to see if this is shorter
                        if let Some(mut min) = min {
                            if app.rx.remaining <  min {
                                min = app.rx.remaining;
                                appid  = Some(app.appid());
                            }
                        }
                        // otherwise, this is the min so far
                        else{
                            min = Some(app.rx.remaining);
                            appid  = Some(app.appid());
                        }
                    }
                });
            }

            // if there was a request found,dispatch it
            if let Some(_min) = min {
                if let Some(appid) = appid {
                    self.transmit_app_rx_request(appid);
                } else if let Some(request) = requests_stash[min_index].take() {
                    self.uart.receive_buffer(request);
                }
                return true;
            }

        }

        false
    }

    fn transmit_app_rx_request(&self, app_id: AppId) -> ReturnCode {
        if let Some(request) = self.app_requests.rx.take(){
            if let Err(_err) = self.apps.enter(app_id, |app, _| {
                request.reset();
                request.initialize_from_app_request(&mut app.rx);
            }){ return  ReturnCode::FAIL };

            self.app_requests.rx_in_progress.set(app_id);
            self.uart.receive_buffer(request)
        }
        else{
            // uart transmit_app_request invoked but no request_tx buffer available
            ReturnCode::FAIL
        }
    }
}

impl Driver for UartDriver<'a> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `1`: Writeable buffer for write buffer
    /// - `2`: Writeable buffer for read buffer
    fn allow(&self, appid: AppId, arg2: usize, slice: Option<AppSlice<Shared, u8>>) -> ReturnCode {
        let allow_num = arg2 as u16;
        let uart_num =  (arg2 >> 16) as usize;
        match allow_num {
            1 => self.uart[uart_num]
                .apps
                .enter(appid, |app, _| {
                    app.tx.slice = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            2 => self.uart[uart_num]
                .apps
                .enter(appid, |app, _| {
                    app.rx.slice = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `1`: Write buffer completed callback
    fn subscribe(&self, arg1: usize, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        let subscribe_num = arg1 as u16;
        let uart_num =  (arg1 >> 16) as usize;
        match subscribe_num {
            1 /* putstr/write_done */ => {
                self.uart[uart_num].apps.enter(app_id, |app, _| {
                    app.tx.callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            2 /* getnstr done */ => {
                self.uart[uart_num].apps.enter(app_id, |app, _| {
                    app.rx.callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }

    /// Initiate serial transfers
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Transmits a buffer passed via `allow`, up to the length
    ///        passed in `arg1`
    /// - `2`: Receives into a buffer passed via `allow`, up to the length
    ///        passed in `arg1`
    /// - `3`: Cancel any in progress receives and return (via callback)
    ///        what has been received so far.
    fn command(&self, arg0: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        let cmd_num = arg0 as u16;
        let uart_num =  (arg0 >> 16) as usize;
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 /* transmit request */ => { 
                // update the request with length
                if let Err(_err) = self.uart[uart_num].apps.enter(appid, |app, _| {
                    let len = arg1;
                    app.tx.set_len(len);
                }){ return  ReturnCode::FAIL }

                self.uart[uart_num].state.map_or(ReturnCode::ENOSUPPORT, 
                    |state| {
                    if state.tx == IDLE {
                        state.tx = BUSY;
                        self.transmit_app_tx_request(uart_num, appid)
                    }
                    else {
                        ReturnCode::SUCCESS
                    }
                })

            },
            2 /* getnstr */ => { 
                // update the request with length
                if let Err(_err) = self.uart[uart_num].apps.enter(appid, |app, _| {
                    let len = arg1;
                    app.rx.set_len(len);
                }){ return  ReturnCode::FAIL }

                self.uart[uart_num].state.map_or(ReturnCode::ENOSUPPORT, 
                    |state| {
                    if state.rx == IDLE {
                        state.rx = BUSY;
                        self.uart[uart_num].transmit_app_rx_request(appid)
                    }
                    else {
                        ReturnCode::SUCCESS
                    }
                })
            },
            3 /* abort rx */ => {
                self.uart[uart_num].uart.receive_abort();
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT
        }
    }
}