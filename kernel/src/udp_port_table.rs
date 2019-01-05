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
// Opaque way to pass id into function (private struct)
// Separate struct for send id and receive id.
// pub struct UDPID {
//     id: usize,
// }

// An opaque descriptor object that gives the holder of the object access to
// a particular location (at index idx) of the bound port table.
pub struct UdpPortBinding {
    receive_allocated: Cell<bool>,
    send_allocated: Cell<bool>,
    idx: usize,
    port_num: Cell<Option<u16>>,
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
                        idx: idx,
                        port_num: Cell::new(Some(0))} // TODO: initialize to what?
    }

    // TODO: verify impl
    pub fn bound(&self) -> bool {
        true // TODO: implement
    }

    // TODO: verify impl
    pub fn port(&self) -> Option<u16> { // Look up in table or store in object?
        self.port_num.get()
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
        // TODO: check here, update self.idx?
        // if self.receive_allocated {
        //     Err(())
        // } else {
        //     self.receive_allocated = true;
        //     self.idx = recv_binding.idx;
        //     Ok(())
        // }
        // TODO: mutability here? do we have to use Cells?
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

    // return UDPID so it's more opaque
    // pub fn add_new_client(&self) -> Option<UDPID> { // TODO: add return code
    //     // use result instead of option
    //     let ret = self.max_counter.get().unwrap();
    //     if ret < MAX_NUM_BOUND_PORTS {
    //         self.max_counter.set(ret + 1);
    //         Some(UDPID {id: ret})
    //     } else {
    //         None
    //     }
    // }

    pub fn create_binding(&self) -> Result<UdpPortBinding, ReturnCode> {
        let ret = self.max_counter.get();
        if ret < MAX_NUM_BOUND_PORTS {
            self.max_counter.set(ret + 1);
            Ok(UdpPortBinding::new(ret))
        } else {
            Err(ReturnCode::FAIL)
        }
    }


    // pub fn get_port_at_id(&self, id_desc: &UDPID) -> Option<u16> {
    //     let mut port = None;
    //     self.udpid_to_port.map(|table| {
    //         port = table[id_desc.id].clone(); // is clone needed here?
    //     });
    //     port
    // }

    // TODO: should this also return some opaque object?
    // pub fn get_id_with_port(&self, port_number: u16) -> Option<usize> {
    //     for i in 0..MAX_NUM_BOUND_PORTS {
    //         match self.get_port_at_id(&(UDPID {id:i})) {
    //             None => (),
    //             Some(port_num) => {
    //                 return Some(i);
    //             },
    //         };
    //     }
    //     None
    // }

    // instead of a UDP ID, want some object/struct wrapper that doesn't allow
    // caller to know what the underlying 
    // upd_id that is passed in needs to be opaque, use struct

    // TODO: how to let go of a port?
    // Returns true if successful, false if not successful
    // pub fn bind_port_to_id(&self, port_number: u16, udp_id: UDPID,
    //                     cap: &capabilities::UDPBindCapability) -> ReturnCode {
    //     // calling twice in a row
    //     match self.get_id_with_port(port_number) {
    //         None => (),
    //         Some(idx) => {
    //             if idx != udp_id.id {
    //                 return ReturnCode::FAIL;
    //             }
    //         },
    //     };
    //     self.udpid_to_port.map(|table| {
    //         table[udp_id.id] = Some(port_number);
    //     });
    //     ReturnCode::SUCCESS
    // }

    // TODO: should binding be a shared ref? (&), should ref to self be mutable?
    pub fn bind(&self, binding: &UdpPortBinding, port: u16,
                /*cap: &capabilities::UDPBindCapability*/) -> ReturnCode {
        // TODO: at this point in the code, do we care about sender vs. recv?
        // for i in 0..MAX_NUM_BOUND_PORTS {
        //     self.udpid_to_port.map(|table| {
        //         match table[i] {
        //             Some(port) => {
        //                 ReturnCode::FAIL;
        //             },
        //             _ => ()
        //         }
        //     });
        // }
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
    //pub fn destroy_binding(binding: UdpPortBinding);


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
            // if table[binding.idx] == port {
            //     ReturnCode::SUCCESS
            // } else {
            //     ReturnCode::FAIL
            // }
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