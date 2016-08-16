use core::slice;
use core::option::Option;
use core::ops::Index;
use main::{Driver};
use common::{RingBuffer, Queue};
use allocator::{Allocator};
// TODO: think of a good way to import.
//use chips::sam4l::flashcalw::{FLASHCALW, flash_controller };
// TODO: import buddy alloc and flash...

/*
    TODO( in the future)
    Have my storage 'walk memory' it's declared on, on bootup so it can know
    what's allocated, and what's not already.
*/

// todo: FIGURE out storage issues...

const NUM_FILE_DESCRIPTORS : usize = 5;

    
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
#[derive(Copy, Clone)]
pub struct Callback {
    id: usize, // which index in the table does this relate to...
    offset: u32, // starting position of writing
}

//TODO implement right
//static buffer : [Block<'a>; 5] = [ Block { slice : &[] }; 5];


pub struct Storage <'a> {
    // todo: might modify and wrap the block up maybe with a client id / app id?
    block_table: [Option<*mut Block<'a>>; NUM_FILE_DESCRIPTORS],
    // TODO: should really be a callback...
    write_queue: RingBuffer<'a, Callback>,
    // TODO add buddy alloc( in the correct place!) / using the dependcy system when
    // it's good.
    allocator: Allocator,
    last_fd: i32, // last used 'index' into block table. Remember to flush if
                  // a close or free is called!
    // todo change to a trait ( for flash driver)
}


impl<'a> Storage<'a> {
    // todo change to take in anything with allocator trait, and anything with
    // flash trait?
    /*pub fn new() -> 'a Storage {
        Storage {
            block_table: [&[]; 5],
            write_queue: RingBuffer::new(
        }
    }*/

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
        if id == -1 || !self.write_queue.enqueue(Callback { 
            id: id as usize, offset : offset})  {
            ErrorCode::failure
        } else { // the request has been successfully enqueued.
            ErrorCode::success
        }
    }

    
}
