use core::slice;
use core::option::Option;
use core::ops::Index;
use core::cell::Cell;

use main::{Driver, AppId};
use common::take_cell::TakeCell;
use common::{List, ListLink, ListNode, Queue};
use common::allocator::{Allocator};
use hil::storage_controller::{StorageController, Client, Error};

// TODO: import buddy alloc and flash...
/*
    TODO( in the future)
    Have my storage 'walk memory' it's declared on, on bootup so it can know
    what's allocated, and what's not already.
*/

// TODO:
// Move the allocator stuff in 'platform' and allow allocator to have access to
// flash in some way ( that way it can do merging and so forth when there are frees / 
// coalescing)

const NUM_CLIENTS : usize = 5;
// This will depend on the system...
const ALLOCATOR_START_ADDR : usize = 0x40000;
const ALLOCATOR_SIZE : usize = 0x40000;
const ALLOCATOR_SMALLEST_BLOCK_SIZE : usize = 1024;

    
pub enum ErrorCode {
    success,
    failure,
}

pub enum RequestResponse {
    success, // success!
    alloc_fail, // size not available.
    clients_full, // Too many clients, no room for you now...
    current_pending_init // try again soon. One is currently pending to init..
}

#[derive(Clone, Copy)]
pub struct Block <'a> {
    slice: &'a[u8]
}

impl<'a> Block <'a> {
    pub fn read(&self, index : usize) -> u8 {
        self.slice[index]
    }
}

impl<'a> Index<usize> for Block <'a> {
    type Output = u8;

    fn index(&self, index : usize) -> &u8 {
        &self.slice[index] 
    }
}

// This is enqueued...

pub struct Callback <'a>{
    id: usize, // which index in the table does this relate to...
    offset: u32, // starting position of writing
    next: ListLink<'a, Callback<'a>>
}


impl <'a> PartialEq for Callback<'a> {
    fn eq(&self, other: &Callback<'a>) -> bool {
        self as *const Callback<'a> == other as *const Callback<'a>
    }
}


impl <'a> ListNode<'a, Callback<'a>> for Callback<'a> {
    fn next(&self) -> &'a ListLink<Callback<'a>> {
        &self.next
    }
}

struct ClientInfo {
    client_id : TakeCell<AppId>, // id of client ( None if there's no client )
    queued_write: Cell<bool>, // whether there's a write pending for this client
    offset: Cell<u32>, // the offset of the current request
    inited: Cell<bool>, // whether this client has been initalized...
    addr: Cell<usize>,
    size: Cell<usize>,
}


// Trait that any client of storage needs to implement.
pub trait StorageClient {
    fn write(arr : &[u8], size : usize);
}

pub struct Storage <'a, S: StorageController + 'a> {
    controller: &'a mut S,
    pending_requests: Cell<u32>, // number pending requests to write...
    pending_init: Cell<bool>, // number of inits ( need to write after opening) 
    clients: [ClientInfo; NUM_CLIENTS],
    allocator: TakeCell<Allocator>,
    buffer: TakeCell<[u8; 512]>,
    last_index: Cell<usize> // last index that was given a chance to write
}


impl<'a, S: StorageController> Storage<'a,S> {
    
    pub fn new(storage_controller: &'a mut S) -> Storage<'a, S> {
        Storage {
            allocator: TakeCell::new(Allocator::new(ALLOCATOR_START_ADDR, ALLOCATOR_SIZE, 
                ALLOCATOR_SMALLEST_BLOCK_SIZE)),
            // TODO: just take the reference from the storage_controller?
            buffer: TakeCell::new([255; 512]),
            // TODO: defn a macro for these
            clients: [ClientInfo {
                client_id: TakeCell::empty(),
                queued_write: Cell::new(false),
                offset: Cell::new(0),
                inited: Cell::new(false),
                addr: Cell::new(0),
                size: Cell::new(0)
                },
                ClientInfo {
                client_id: TakeCell::empty(),
                queued_write: Cell::new(false),
                offset: Cell::new(0),
                inited: Cell::new(false),
                addr: Cell::new(0),
                size: Cell::new(0)
                },
                ClientInfo {
                client_id: TakeCell::empty(),
                queued_write: Cell::new(false),
                offset: Cell::new(0),
                inited: Cell::new(false),
                addr: Cell::new(0),
                size: Cell::new(0)
                },
                ClientInfo {
                client_id: TakeCell::empty(),
                queued_write: Cell::new(false),
                offset: Cell::new(0),
                inited: Cell::new(false),
                addr: Cell::new(0),
                size: Cell::new(0)
                },
                ClientInfo {
                client_id: TakeCell::empty(),
                queued_write: Cell::new(false),
                offset: Cell::new(0),
                inited: Cell::new(false),
                addr: Cell::new(0),
                size: Cell::new(0)
                }],
            controller: storage_controller,
            pending_requests: Cell::new(0),
            pending_init: Cell::new(false),
        }
    }
   
   
    pub fn request(&self, size : usize, id: AppId) -> RequestResponse {
        if self.pending_init.get() {
            return RequestResponse::current_pending_init
        }
        let mut index = -1;

        for i in 0..NUM_CLIENTS {
            if self.clients[i].client_id.is_none() {
                index = i as i32;
                break;
            }
        }
        
        if index == -1 {
            return RequestResponse::clients_full
        }

        let base_addr = self.allocator.map(|value| {
            value.alloc(size)
        }).unwrap();
        
        if base_addr.is_none() {
            return RequestResponse::alloc_fail
        }
        
        // config the clients information
        self.clients[index as usize].size.set(size);
        self.clients[index as usize].addr.set(base_addr.unwrap());
        self.clients[index as usize].inited.set(false);
        self.clients[index as usize].client_id.replace(id);
        self.clients[index as usize].queued_write.set(true);
        
        self.pending_init.set(true); // we have an init pending...
        RequestResponse::success
    }

    // index on success (AppId found in client info)
    // -1 on failure
    fn get_index_position(&self, id: AppId) -> i32 {
        for i in 0..NUM_CLIENTS {
            if !self.clients[i].client_id.is_none() {
                if self.clients[i].client_id.map(|index_id| { index_id == id}) {
                    return i as i32
                }
            }
        }
        -1 
    }

    // -1 on failure, index of client to close otherwise
    fn can_close(&self, id: AppId) -> i32 {
        let index = self.get_index_position(id); 
        // fail if AppId not found, or if there is a pending write from the client.
        if index == -1 || self.clients[index as usize].queued_write.get() {
            return -1
        }
        index
    }
    
    pub fn close(&self, id: AppId) -> bool {
        let idx = self.can_close(id);

        if idx != -1 {
            self.clients[idx as usize].client_id.take(); // remove client id
            true
        } else {
            false
        }
    }
   
    // TODO: will need to change ( when allocator has access to flash because it needs for
    // underlying writes) will need to do in order for this to work
    pub fn free(&self, id: AppId) -> bool {
        let idx = self.can_close(id);

        if idx != -1 {
            self.clients[idx as usize].client_id.take();
            self.allocator.map(|value| {
                value.free(self.clients[idx as usize].addr.get())
            });
            true
            // TODO: give allocator flash driver and free flash area
        } else {
            false
        }
    }

    // true if write is initiated false if it fails ( i.e. you have a pending write or
    // you don't have write access...
    pub fn initiate_write(&self, id: AppId, offset: u32) -> bool {
        let idx = self.get_index_position(id);
        // if not found or write already queued fail
        if idx == -1 || self.clients[idx as usize].queued_write.get() {
            return false
        }
        
        // set the client writes to true, increment my counter for number_write.
        self.clients[idx as usize].queued_write.set(true);
        self.pending_requests.set(self.pending_requests.get() + 1);
        
        // if no other writes pending start a write... TODO (probably).. in order
        // to start a chain of writes with commonad_complete...

        true
    }

    // runs a particular application thats next up in use of the flash. 
    fn run(&self) {
        if !self.controller.storage_ready() {
            // Do Nothing the storage isn't ready.
        } else if pending_init.get() {
            // Flush write... Note this is janky and might not work depending on where
            // flash does its writes..
            // Should pass to the allocator  itself and let it do the work and call me
            // back ( and hand back flash :) ).
        } else if self.pending_requests.get() != 0 {
            //write to buffer...
            let buffer = self.buffer.take().unwrap();
            write_to_memory(buffer, );
        }
    }


}

impl<'a, S: StorageController> Client for Storage<'a, S>{
    // This is the function to call on the Storage in order to process any work 
    // queued up.
    fn command_complete(&self, err: Error) {
/*        if self.queued_list.head().is_none() || !self.controller.storage_ready() {
            // do nothing
        } else {
            // lets handle a request to write ( for now just handling stupidly 
            // flushing all writes but could use last_fd and last_offset to lazily
            // write.
            let buffer = self.buffer.take().unwrap();
            //write_to_memory();
            
        } */
    }
}
