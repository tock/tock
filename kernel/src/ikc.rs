//! These are some primitive generics for Intra-Kernel Communication
use super::{AppSlice, Shared, Callback};
use core::cmp;

#[derive(Default)]
pub struct AppRequest <T: Copy> {
    pub slice: Option<AppSlice<Shared, T>>,
    pub callback: Option<Callback>,
    length: usize ,
    pub remaining: usize,
}

impl<T: Copy> AppRequest <T> {
    pub fn set_len(&mut self, len: usize){
        let mut length = len;
        if let Some(ref buf) = self.slice {
            length = cmp::min(length, buf.len())
        }
        self.length = length;
        self.remaining = length;
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn push<'a>(&mut self, element: T) {
        if let Some(ref mut buf) = self.slice {
            let offset = self.length - self.remaining;
            buf.as_mut()[offset] = element;
            self.remaining -= 1;
        }
    }
}

pub enum TxBuf<'a, T: Copy> {
    None,
    CONST(&'a [T]),
    MUT(&'a mut [T]),
}

pub enum RxBuf<'a, T: Copy> {
    None,
    MUT(&'a mut [T]),
}

impl<'a, T: Copy> Default for RxBuf<'a, T> {
    fn default() -> Self { RxBuf::None }
}

impl<'a, T: Copy> Default for TxBuf<'a, T> {
    fn default() -> Self { TxBuf::None }
}

#[derive(Default)]
pub struct TxRequest<'a, T: Copy> {
    buf: TxBuf<'a, T>,
    // The total amount of data written in
    pushed: usize,
    // The total amount of data read out
    popped: usize,
    // The total size of the request
    requested: usize,
    // Identifier to route response to owner
    pub client_id: usize,
}

#[derive(Default)]
pub struct RxRequest<'a, T: Copy> {
    pub buf: RxBuf<'a, T>,
    // The total amount of data written in
    pushed: usize,
    // The total amount of data read out
    popped: usize,
    // The total size of the request
    requested: usize,
    // Identifier to route response to owner
    pub client_id: usize,
}

pub enum Request<'a, T: Copy> {
    TX(&'a mut TxRequest<'a, T>),
    RX(&'a mut RxRequest<'a, T>),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DriverState {
    BUSY,
    IDLE,
}

impl<'a, T: Copy> TxRequest<'a, T> {
    pub fn pop(&mut self) -> Option<T> {
        let ret = match &self.buf {
            TxBuf::CONST(s) => Some(s[self.popped]),
            TxBuf::MUT(ref s) => Some(s[self.popped]),
            TxBuf::None => None,
        };
        self.popped += 1;
        ret
    }

    // this interface is for a Client that wants to prepare a Request
    // as such, APP_SLICE and CONST types are not treated
    pub fn push(&mut self, element: T) {
        match &mut self.buf {
            TxBuf::MUT(buf) => {
                buf[self.pushed] = element;
            }
            TxBuf::CONST(_buf) => panic!("Should not be pushing data into constant TxRequest!"),
            TxBuf::None => panic!("Should not be pushing data into TxRequest with no TxBuf!")
        }

        // increment both the pushed and requested amount
        self.pushed += 1;
        self.requested += 1;
    }

    pub fn copy_from_app_request(&mut self, app_request: &mut AppRequest<T>) {
        match &mut self.buf {
            TxBuf::MUT(ref mut buf) => {
                let num_elements = cmp::min(buf.len(), app_request.remaining);
                let offset = app_request.length - app_request.remaining;

                if let Some(ref slice) = app_request.slice {
                    for i in 0..num_elements {
                        buf[i] = slice.as_ref()[i + offset];
                    }
                    self.popped = 0;
                    self.pushed = num_elements;
                    self.requested = num_elements;
                    app_request.remaining -= num_elements;
                } else {
                    app_request.remaining = 0;
                }
            },
            _ => panic!("Can only copy_from_app_slice if self is TxBuf::MUT"),
        };
    }

    pub fn has_some(&self) -> bool {
       self.popped < self.pushed
    }

    pub fn requested_length(&self) -> usize {
        self.requested
    }

    pub fn remaining_request(&self) -> usize {
        self.requested - self.popped
    }

    pub fn has_request_remaining(&self) -> bool {
        self.popped < self.requested
    }

    pub fn request_completed(&self) -> bool {
        self.popped >= self.requested
    }

    pub fn has_room(&self) -> bool {
        match &self.buf {
            TxBuf::MUT(buf) => self.pushed < buf.len(),
            _ => false,
        }
    }

    pub fn room_available(&self) -> usize {
        match &self.buf {
            TxBuf::MUT(buf) => buf.len() - self.pushed,
            _ => 0,
        }
    }

    pub fn reset(&mut self) {
        self.pushed = 0;
        self.popped = 0;
        match &self.buf { 
            TxBuf::MUT(_buf) => self.requested = 0,
            TxBuf::CONST(buf) => self.requested = buf.len(),
            TxBuf::None => {},
        }
    }

    // for TxRequest with const reference, pushed = requested = buffer length
    pub fn set_with_const_ref(&mut self, buf: &'a [T]) {
        self.pushed = buf.len();
        self.requested = buf.len();
        self.buf = TxBuf::CONST(buf);
        self.popped = 0;
    }

    // for TxRequest with mutable reference
    // it is assumed empty so client will fill before dispatching
    pub fn set_with_mut_ref(&mut self, buf: &'a mut [T]) {
        self.buf = TxBuf::MUT(buf);
        self.pushed = 0;
        self.popped = 0;
        self.requested = 0;
    }

    // initializes space expect for the TxItem, which must be allocated elsewhere
    pub fn new() -> TxRequest<'a, T> {
        TxRequest {
            buf: TxBuf::None,
            pushed: 0,
            popped: 0,
            requested: 0,
            client_id: 0xFF,
        }
    }

    pub fn new_with_const_ref(buf: &'a [T]) -> TxRequest<'a, T> {
        let length = buf.len();
        Self::new_with_ref_set_len(TxBuf::CONST(buf), length)
    }

    pub fn new_with_mut_ref(buf: &'a mut [T]) -> TxRequest<'a, T> {
        let length = buf.len();
        Self::new_with_ref_set_len(TxBuf::MUT(buf), length)
    }

    // allow user to set request length, but don't let it exceed buffer/slice size
    pub fn set_request_len(&mut self, length: usize) {
        match &self.buf { 
            TxBuf::MUT(buf) => self.requested = cmp::min(length, buf.len()),
            TxBuf::CONST(buf) => self.requested = cmp::min(length, buf.len()),
            TxBuf::None => {},
        }
    }

    pub fn new_with_ref_set_len(buf: TxBuf<'a, T>, length: usize) -> TxRequest<'a, T> {
        match buf {
            TxBuf::CONST(b) => TxRequest {
                buf: TxBuf::CONST(b),
                pushed: length,
                requested: length,
                popped: 0,
                client_id: 0xFF,
            },
            TxBuf::MUT(b) => TxRequest {
                buf: TxBuf::MUT(b),
                pushed: 0,
                popped: 0,
                requested: 0,
                client_id: 0xFF,
            },
            TxBuf::None => TxRequest {
                buf: TxBuf::None,
                pushed: 0,
                popped: 0,
                requested: 0,
                client_id: 0xFF,
            },
        }
    }

}

impl<'a, T: Copy> RxRequest<'a, T> {
    pub fn new() -> RxRequest<'a, T> {
        RxRequest {
            buf: RxBuf::None,
            pushed: 0,
            popped: 0,
            requested: 0,
            client_id: 0xFF,
        }
    }

    pub fn initialize_from_app_request(&mut self, app_request: &mut AppRequest<T>) {
        match &mut self.buf {
            RxBuf::MUT(ref mut buf) => {
                let num_elements = cmp::min(buf.len(), app_request.remaining);
                if let Some(ref _slice) = app_request.slice {
                    self.requested = num_elements;
                }
            },
            _ => panic!("Can only copy_from_app_slice if self is RxBuf::MUT"),
        };
    }

    pub fn requested_length(&self) -> usize {
       self.requested
    }

    pub fn new_with_mut_ref(buf: &'a mut [T]) -> RxRequest<'a, T> {
        RxRequest {
            requested: buf.len(),
            buf: RxBuf::MUT(buf),
            pushed: 0,
            popped: 0,
            client_id: 0xFF,
        }
    }

    // RxRequest is assumed empty and we assume client wants host to fill buffer
    pub fn set_buf(&mut self, buf: &'a mut [T]) {
        self.requested = buf.len();
        self.buf = RxBuf::MUT(buf);
        self.pushed = 0;
        self.popped = 0;
    }

    // Reset pushed/popped values and assume client wants host to fill buffer
    pub fn reset(&mut self) {
        self.pushed = 0;
        self.popped = 0;
        match &self.buf {
            RxBuf::MUT(buf) => self.requested = buf.len(),
            RxBuf::None => self.requested = 0,
        }
    }

    // Host has pushed enough data to fulfill the request
    pub fn request_completed(&self) -> bool {
        self.pushed >= self.requested
    }

    // How much data has been pushed
    pub fn items_pushed(&self) -> usize {
        self.pushed
    }

    pub fn request_remaining(&self) -> usize {
        self.requested - self.pushed
    }

    pub fn has_room(&self) -> bool {
        match &self.buf {
            RxBuf::MUT(buf) => self.pushed < buf.len(),
            RxBuf::None => false,
        }
    }

    pub fn push(&mut self, element: T) {
        match &mut self.buf {
            RxBuf::MUT(buf) => {
                buf[self.pushed] = element;
            }
            RxBuf::None => {}
        }
        self.pushed += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        let ret = match &self.buf {
            RxBuf::MUT(s) => Some(s[self.popped]),
            RxBuf::None => None,
        };
        self.popped += 1;
        ret
    }
}