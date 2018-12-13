use common::cells::TakeCell;
use core::cell::Cell;
use capabilities;
use ReturnCode;
const MAX_NUM_BOUND_PORTS: usize = 16;
//#![allow(dead_code)]
static mut port_table: [Option<u16>; MAX_NUM_BOUND_PORTS] = [None; MAX_NUM_BOUND_PORTS];
// Opaque way to pass id into function (private struct)
// Separate struct for send id and receive id.
pub struct UDPID {
    id: usize,
}


pub struct UDPPortTable {
    udpid_to_port: TakeCell<'static, [Option<u16>]>,
    max_counter: Cell<usize>,
}


impl UDPPortTable {
    // TODO: update constructor to accept a reference to the port_table.
    pub fn new() -> UDPPortTable {
        unsafe {
            UDPPortTable {
                udpid_to_port: TakeCell::new(&mut port_table),
                max_counter: Cell::new(0),
            }
        }
    }

    // return UDPID so it's more opaque
    pub fn add_new_client(&self) -> Option<UDPID> { // TODO: add return code
        // use result instead of option
        let ret = self.max_counter.get();
        if ret < MAX_NUM_BOUND_PORTS {
            self.max_counter.set(ret + 1);
            Some(UDPID {id: ret})
        } else {
            None
        }
    }


    pub fn get_port_at_id(&self, id_desc: &UDPID) -> Option<u16> {
        let mut port = None;
        self.udpid_to_port.map(|table| {
            port = table[id_desc.id].clone(); // is clone needed here?
        });
        port
    }

    // TODO: should this also return some opaque object?
    pub fn get_id_with_port(&self, port_number: u16) -> Option<usize> {
        for i in 0..MAX_NUM_BOUND_PORTS {
            match self.get_port_at_id(&(UDPID {id:i})) {
                None => (),
                Some(port_num) => {
                    return Some(i);
                },
            };
        }
        None
    }

    // instead of a UDP ID, want some object/struct wrapper that doesn't allow
    // caller to know what the underlying 
    // upd_id that is passed in needs to be opaque, use struct

    // TODO: how to let go of a port?
    // Returns true if successful, false if not successful
    pub fn bind_port_to_id(&self, port_number: u16, udp_id: UDPID,
                        cap: &capabilities::UDPBindCapability) -> ReturnCode {
        // calling twice in a row
        match self.get_id_with_port(port_number) {
            None => (),
            Some(idx) => {
                if idx != udp_id.id {
                    return ReturnCode::FAIL;
                }
            },
        };
        self.udpid_to_port.map(|table| {
            table[udp_id.id] = Some(port_number);
        });
        ReturnCode::SUCCESS
    }

    pub fn unbind(&mut self, udp_id: UDPID, cap: &capabilities::UDPBindCapability) {
        self.udpid_to_port.map(|table| {
            table[udp_id.id] = None;
        });
    }
}