use core::slice;
use core::option::Option;
use core::ops::Index;
use main::{Driver};
use common::{List, ListLink, ListNode, Queue};
//use common::{RingBuffer, Queue};
use allocator::{Allocator};
// TODO: think of a good way to import.
//use chips::sam4l::flashcalw::{FLASHCALW, flash_controller };
// TODO: import buddy alloc and flash...
// TODO: import buddy allocator using cargo / crates..
/*
    TODO( in the future)
    Have my storage 'walk memory' it's declared on, on bootup so it can know
    what's allocated, and what's not already.
*/

// todo: FIGURE out storage issues...

const NUM_FILE_DESCRIPTORS : usize = 5;
// This will depend on the system...
const ALLOCATOR_START_ADDR : usize = 0x40000;
const ALLOCATOR_SIZE : usize = 0x40000;
const ALLOCATOR_SMALLEST_BLOCK_SIZE : usize = 1024;

    
pub enum ErrorCode {
    success,
    failure,
}

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
        // TODO: think slice itself would handle Out of Bounds.
    }
}

// This is enqueued...
pub struct Callback <'a>{
    id: usize, // which index in the table does this relate to...
    offset: u32, // starting position of writing
    next: ListLink<'a, Callback<'a>>
}

impl <'a> ListNode<'a, Callback<'a>> for Callback<'a> {
    fn next(&self) -> &'a ListLink<Callback<'a>> {
        &self.next
    }
}

pub struct Storage <'a> {
    // todo: might modify and wrap the block up maybe with a client id / app id?
    block_table: [Option<*mut Block<'a>>; NUM_FILE_DESCRIPTORS],
    queued_list: List<'a, Callback<'a>>,
    allocator: Allocator,
    last_fd: i32, // last used 'index' into block table. Remember to flush if
                  // a close or free is called!
    // todo change to a trait ( for flash driver)
}


impl<'a> Storage<'a> {
    // todo change to take in anything with allocator trait, and anything with
    // flash trait?
    pub fn new() -> Storage<'a> {
        Storage {
            block_table: [None; 5],
            queued_list: List::new(),
            allocator: Allocator::new(ALLOCATOR_START_ADDR, ALLOCATOR_SIZE, 
                ALLOCATOR_SMALLEST_BLOCK_SIZE),
            last_fd: -1,
        }
    }

    // TODO: this needs to be able to fail ( could give an option to say why fail
    // i.e. alloc out of memory or block table full
    pub fn request(&mut self, size : usize) -> Option<Block> {
        let mut index : i32 = -1;
        
        // If either the block table  or the allocator don't have space, then fail.
        for i in 0..NUM_FILE_DESCRIPTORS {
            if self.block_table[i].is_none() {
                index = i as i32;
                break;
            }
        }

        if index == -1 {
            return None
        }

        let space = self.allocator.alloc(size);
        if space.is_none() {
            return None
        }
    
        // Make the Block, and update the block_table index.
        self.block_table[index as usize] = Some(space.unwrap() as *mut Block<'a>);
        Some(Block {
            slice: unsafe { slice::from_raw_parts(space.unwrap() as *mut u8,size) }
        })
    }

    // closes the block from being accessable.
    pub fn close(&mut self, block : Block) -> Option<Block> {
       unimplemented!();     
    }

    // closes the block, and also deallocates it!
    pub fn free(&mut self, block : Block) {
        //TODO: check address / code logic
        let address = block.slice[0] as *mut u8 as usize;
        self.close(block);
        self.allocator.free(address);
    }

   // pub fn 

// TODO: the client will have an interface some trait / function that they have to 
// implement in order to use the storage and that's where I send the CB to.

    pub fn initiate_write<F>(&mut self, block : &mut Block, offset: u32) -> ErrorCode {
        // Find block in block table...
        let mut id : i32 = -1;
        for i in 0..NUM_FILE_DESCRIPTORS {
            if !self.block_table[i].is_none() && self.block_table[i].unwrap() 
                == block.slice[0] as *mut Block<'a> {
                id = i as i32;
                break;
            }
        }
        
        // Error out if this block doesn't exist or we don't have room to queue a write.
        // TODO: uncomment and use instead.
        if id == -1 || !self.write_queue.as_mut().unwrap().enqueue(Callback { 
            id: id as usize, offset : offset})  {
            ErrorCode::failure
        } else { // the request has been successfully enqueued.
            ErrorCode::success
        }
    }

    
}
