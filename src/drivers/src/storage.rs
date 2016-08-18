use core::slice;
use core::option::Option;
use core::ops::Index;
use main::{Driver};
use common::{List, ListLink, ListNode, Queue};
//use common::{RingBuffer, Queue};
use common::allocator::{Allocator};
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
        // TODO : think of a safer way to compare.
        self as *const Callback<'a> == other as *const Callback<'a>
    }
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
        /* TODO: reword...
        Some(Block {
            slice: unsafe { slice::from_raw_parts(space.unwrap() as *mut u8,size) }
        })
        */
        None
    }

    // closes the block from being accessable if there's no writes left...
    // Returns None if the block is closed, or the block if there are pending
    // writes / it's not found in the table...
    pub fn close<'b>(&'b mut self, mut block : Block<'b>) -> Option<Block> {
        let idx = self.find_block_in_table(&mut block);
        if idx == -1 {
            return Some(block) // error block not found in table... 
        }
        
        // check to make sure no more queued up writes
        let mut iter = self.queued_list.iter();
        let mut curr = iter.next();
        
        while !curr.is_none() {
            if curr.unwrap().id == (idx as usize) {
                return Some(block)
            }
        }

        // No more queued up writes, so lets close it.
        self.block_table[idx as usize] = None;
        None
    }

    // closes the block, and also deallocates it!
    pub fn free<'b>(&'b mut self, mut block : Block<'b>) -> Option<Block> {
        //TODO: check address / code logic
        let address = block.slice[0] as *mut u8 as usize;
        let results: Option<Block> = self.close(block);
        // Why is it borrowing self.allocator as mut?
        if results.is_none() {
            self.allocator.free(address);
            None
        } else {
            results
        }
    }

    // returns the index in the block_table of a block ( or -1 on failure)
    fn find_block_in_table(&self, block: &mut Block) -> i32 {
        for i in 0..NUM_FILE_DESCRIPTORS {
            if !self.block_table[i].is_none() && self.block_table[i].unwrap() 
                == block.slice[0] as *mut Block<'a> {
                return i as i32
            }
        }
        -1 // not found in table
    }

// TODO: the client will have an interface some trait / function that they have to 
// implement in order to use the storage and that's where I send the CB to.

    pub fn initiate_write<F>(&mut self, block : &mut Block, offset: u32) -> ErrorCode {
        // Find block in block table...
        let id = self.find_block_in_table(block);
        // Error out if this block doesn't exist.
        if id == -1 { 
            ErrorCode::failure
        } else { 
            // the request has been successfully enqueued.
            // TODO: insert into list!
            ErrorCode::success
        }
    }

    
}
