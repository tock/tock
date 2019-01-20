//! UDP port table implementation for enforcing port binding. This table is
//! checked when packets are sent/received by a UDPSendStruct/UDPReceiver.
//! UdpPortBinding provides an opaque descriptor object that allows the holder
//! interact with the bound port table. Only the holder of the UdpPortBinding
//! object can interact with its own corresponding location in the bound port
//! table. In order to bind to a particular port as sending/receiving, one must
//! obtain the corresponding sender/receiving binding from UdpPortBinding.
use tock_cells::take_cell::TakeCell;
use core::cell::Cell;
//use capabilities;
use crate::returncode::ReturnCode;
//#![allow(dead_code)]
const MAX_NUM_BOUND_PORTS: usize = 16;
static mut port_table: [Option<u16>; MAX_NUM_BOUND_PORTS] = [None; MAX_NUM_BOUND_PORTS];

// An opaque descriptor object that gives the holder of the object access to
// a particular location (at index idx) of the bound port table.
pub struct UdpPortBinding {
    receive_allocated: Cell<bool>,
    send_allocated: Cell<bool>,
    idx: usize,
}

// An opaque descriptor that allows the holder to obtain a binding on a port
// for receiving UDP packets.
pub struct UdpReceiverBinding {
    idx: usize,
}

// An opaque descriptor that allows the holder to obtain a binding on a port
// for sending UDP packets.
pub struct UdpSenderBinding {
    idx: usize,
}



pub struct UdpPortTable {
    udpid_to_port: TakeCell<'static, [Option<u16>]>,
    max_counter: Cell<usize>,
}

impl UdpPortBinding {
    pub fn new(idx: usize) -> UdpPortBinding {
        UdpPortBinding {receive_allocated: Cell::new(false),
                        send_allocated: Cell::new(false),
                        idx: idx} // TODO: initialize to what?
    }

    // TODO: should probably change error type to ReturnCode
    pub fn get_receiver(&self) -> Result<UdpReceiverBinding, ()> {
        // What if self.send_allocated?
        if self.receive_allocated.get() {
           Err(())
        } else {
            self.receive_allocated.set(true);
            Ok(UdpReceiverBinding { idx: self.idx })
        }
    }

    pub fn put_receiver(&self, recv_binding: UdpReceiverBinding)
        -> Result<(), ()> {
        if recv_binding.idx == self.idx {
            self.receive_allocated.set(false);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn get_sender(&self) -> Result<UdpSenderBinding, ()> {
        if self.send_allocated.get() {
            Err(())
        } else {
            self.send_allocated.set(true);
            Ok(UdpSenderBinding {idx: self.idx })
        }
    }

    pub fn put_sender(&self, send_binding: UdpSenderBinding) -> Result<(), ()> {
        if send_binding.idx == self.idx {
            self.send_allocated.set(false);
            Ok(())
        } else {
            Err(())
        }
    }
}



impl UdpPortTable {
    // TODO: update constructor to accept a reference to the port_table.
    pub fn new() -> UdpPortTable {
        unsafe {
            UdpPortTable {
                udpid_to_port: TakeCell::new(&mut port_table),
                max_counter: Cell::new(0),
            }
        }
    }

    pub fn create_binding(&self) -> Result<UdpPortBinding, ReturnCode> {
        let ret = self.max_counter.get();
        if ret < MAX_NUM_BOUND_PORTS {
            self.max_counter.set(ret + 1);
            Ok(UdpPortBinding::new(ret))
        } else {
            Err(ReturnCode::FAIL)
        }
        // Code below doesn't work => would need a separate array for managing
        // which indices are allocated already. This seems to imply that we have
        // the bindings as valid only when they are "in use"
        // self.udpid_to_port.map(|table| {
        //     let mut result: Result<UdpPortBinding, ReturnCode> = Err(ReturnCode::FAIL);
        //     for i in 0..MAX_NUM_BOUND_PORTS {
        //         match table[i] {
        //             None => {
        //                 result = Ok(UdpPortBinding::new(i));
        //                 break;
        //             },
        //             _ => (),
        //         }
        //     };
        //     result
        // }).unwrap()
    }

    // TODO: double check that this works
    pub fn destroy_binding(&self, binding: UdpPortBinding) {
        self.udpid_to_port.map(|table| {
            table[binding.idx] = None;
        });
    }

    // TODO: should binding be a shared ref? (&), should ref to self be mutable?
    pub fn bind(&self, binding: &UdpPortBinding, port: u16,
                /*cap: &capabilities::UDPBindCapability*/) -> ReturnCode {
        self.udpid_to_port.map(|table| {
            let mut port_exists = false;
            for i in 0..MAX_NUM_BOUND_PORTS {
                match table[i] {
                    Some(port) => {port_exists = true;},
                    _ => (),
                }
            };
            if port_exists {
                ReturnCode::FAIL
            } else {
                table[binding.idx] = Some(port);
                ReturnCode::SUCCESS
            }
        }).unwrap()
    }

    // TODO: implement after testing basic features. Do we need to maintain
    // a free list?



    // Disassociate the port from the given binding.
    // TODO: what would a return value here convey? How can there possibly be
    // failure?
    pub fn unbind(&self, binding: &UdpPortBinding,
        /*cap: &capabilities::UDPBindCapability*/) {
        self.udpid_to_port.map(|table| {
            table[binding.idx] = None;
        });
    }

    // Returns SUCCESS if the table indicates that the sender binding
    // corresponds to the given port.
    pub fn can_send(&self, binding: &UdpSenderBinding, port: u16)
        -> ReturnCode {
        self.udpid_to_port.map(|table| {
            match table[binding.idx] {
                Some(port) => ReturnCode::SUCCESS,
                _ => ReturnCode::FAIL,
            }
        }).unwrap()
    }

    // Returns SUCCESS if the table indicates that the receiver binding
    // corresponds to the given port.
    pub fn can_recv(&self, binding: &UdpReceiverBinding, port: u16)
        -> ReturnCode {
        self.udpid_to_port.map(|table| {
            match table[binding.idx] {
                Some(port) => ReturnCode::SUCCESS,
                _ => ReturnCode::FAIL,
            }
        }).unwrap()
    }


}