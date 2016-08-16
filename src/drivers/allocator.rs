/// This is a memory allocator written in Rust using the binary buddy system.
/// As this uses the binary buddy system, it must be given a size segment that 
/// is a power of two. 
///
/// This memory allocator is designed to not require the std crate that's typically
/// automatically imported in rust files. For this reason I used a freelist
/// of a statically allocated size of 31. Thus if the minimum size block was 1 byte
/// we'd have enough space in are freelist to store sizes up to 2^30 ( we can't have
/// larger sizes in this implementation since the MSB in headers are used to tell
/// whether a block of memory is free or allocated). Furthermore, because there is
/// no std, and I can't generate an array at runtime whose size can change depending on
/// the given amount of memory) I used the free_list_size variable as a comprise 
/// to track the valid number of positions in the free_list array.
///
/// There's a price paid of 4 bytes for block headers. Furthermore, as this is a
/// binary buddy system, every allocated block is a power of two, so there will
/// probably be padding to your blocks. 
///
/// If alignment is of interest to your use, then continue reading below.
/// If you want alignment to say, powers of 2 in your allocated blocks (say for 
/// example memory protection worked at a granduality of powers of two) then
/// whether this will be all blocks would be aligned to that constraint is subject
/// to the base address.
///
///     If the base address is 0x0, then your blocks will always be aligned to powers
///     of two.
///     
///     If the base address is a power of two, then your blocks up to a size of 
///     your base address will be aligned. This is a property from all larger powers
///     of two have have all lower powers of two as factors. 
///     
///     To make this more concrete: if we have 0x10 as our base address, all blocks
///     up to the size 0x10 will be aligned to powers of 2. But if we had a request for
///     a block of size 32, it couldn't be guarenteed to be aligned to a power of two.
///     As the address 0x30 is not a power of two ( 0x10 + 32 ).
///     
///     Similarly, all other addresses can't guarentee alignment.
///     
///
/// Author: Kevin Baichoo <kbaichoo@cs.stanford.edu>
///

use core::option;
use core::mem;

pub struct Allocator {
    start_address : usize,
    size: usize, // size the allocator is given in bytes -- this must be a power of 2. 
    free_list: [Option<*mut BlockHeader>; 31], // freelist of freed blocks 
    free_list_size: u32, // how many positions in the freelist segment is valid.
    smallest_block_size: usize // This is the smallest allocatable block size. Will
                               // have a payload of smallest_block_size - 4 however
                               // as 4 bytes are needed for the header.
}

///
/// The BlockHeader is used to track memory blocks for the allocator. The header
/// field of a block is always valid. The next field will be valid if the block
/// is free, and invalid if the block is in use ( as then it's payload ). None is
/// used to signify that there is no next block.
///
/// The MSB for the header field is a bit which to track whether the block is free
/// or allocated. The remaining 31 bits are the blocks size.
///
#[derive(Copy, Clone)]
struct BlockHeader {
    header: u32, 
    next:   Option<*mut BlockHeader>
}


impl BlockHeader {
    fn is_free(&self) -> bool {
        (self.header & (1 << 31)) != 0 
    }

    fn get_size(&self) -> u32 {
        self.header & !(1 << 31) 
    }

    fn mark_free(&mut self, free : bool) {
        if free {
            // mark free
            self.header = self.header | (1 << 31);
        } else {
            // mark allocated
            self.header &= !(1 << 31);
        }
    }

    fn set_size(&mut self, size : u32) {
        assert!(size < (1 << 31)); // we wouldn't be able to use 
                                   // the msb as a alloc bit if the blocks size
                                   // is this large.
        self.header = (self.header & (1 << 31)) | size;
    }
   
    fn get_next(&self) -> Option<*mut BlockHeader> {
        self.next
    }

    fn set_next(&mut self, next : Option<*mut BlockHeader>) {
        self.next = next;
    }
}

// Expects num to be a power of 2. It then returns which power of two it is.
fn power_of_two( num : usize) -> u32 {
    let max_pos = (mem::size_of::<usize>() * 8) - 1;
    for i in (0..max_pos).rev() {
        if (1 << i) & num != 0 {
            return i as u32
        }
    }
    assert!(false); // should never reach here
    0
}


// Gets the next power of two (for 32 bit). If the input is already a power
// of two then that number is returned.
fn next_power_of_two(mut size : u32) -> u32 {
    size = size - 1;
    size |= size >> 1;
    size |= size >> 2;
    size |= size >> 4;
    size |= size >> 8;
    size |= size >> 16;
    size = size + 1;
    size
}



impl Allocator {
    
    /// constructs a new buddy allocator
    pub fn new( start_addr : usize, sz : usize, smallest_block_size : usize) -> Allocator {
        assert_eq!(sz & (sz - 1), 0); // assert size is a power of two
        let mut num_freelists = power_of_two(sz);
        
        // check that the number of freelist isn't too much 
        // (we're handling at most 2^30 bytes)
        assert!( num_freelists < 31);
        
        // include the smallest block
        num_freelists = num_freelists - power_of_two(smallest_block_size) + 1; 
        
        let mut alloc = Allocator {
            start_address: start_addr,
            size: sz,
            free_list: [None; 31],
            free_list_size: num_freelists,
            smallest_block_size: smallest_block_size
        };
       
        // Add the initial memory block into the freelist.
        let curr_header : &mut BlockHeader = unsafe { mem::transmute(start_addr) };
        curr_header.mark_free(true);
        curr_header.set_size(sz as u32);
        curr_header.set_next(None);

        alloc.free_list[num_freelists as usize - 1] =  
            Some(curr_header as *mut BlockHeader);

        alloc
    }
    
    ///
    /// Tries to allocate a block of size 'size'. It returns the blocks address
    /// on success, and none on failure.
    /// As of now, it rejects small requests ( this might instead be change to
    /// heavily pad those requests).
    ///
    pub fn alloc(&mut self, mut size: usize) -> Option<usize> {
        
        // return on useless request
        if size + mem::size_of::<u32>() < self.smallest_block_size {
            return None
        }
        
        // Add header size and pad size
        size += mem::size_of::<u32>();
        let padded_size = next_power_of_two(size as u32);
        
        // get the index to begin the search
        let mut index = self.get_freelist_index(padded_size as usize);
        let mut block : Option<usize> = None;

        while block.is_none() && index < self.free_list_size as usize {
            if self.free_list[index].is_none() {
                index = index + 1; // this free_list index is empty, continue...
            } else {
                // get the candidate block
                let candidate_block : &mut BlockHeader = unsafe {
                    mem::transmute(self.free_list[index].unwrap())
                };
                
                // take it out of its freelist
                self.free_list[index] = candidate_block.next;
               
                // break up the candidate block until we get the tightest 
                // power of two fit.
                while candidate_block.get_size() != padded_size {
                    self.split_block(candidate_block); 
                }

                // mark block as allocated
                candidate_block.mark_free(false);

                // Adjust the address to point to the beginning where payload is
                // expected. (After the header)
                block = Some(candidate_block as *mut BlockHeader as usize + mem::size_of::<u32>());
            }
        }
      
        // return the appropriate sized block
        block
    }
  

    // Splits the block, add the other half to the freelists and updates both
    // other their headers.
    fn split_block(&mut self, block : &mut BlockHeader) {
        let new_size = block.get_size() / 2;
        let buddy_address = (block as *mut BlockHeader as usize) + new_size as usize;
        let buddy_block : &mut BlockHeader = unsafe { mem::transmute(buddy_address) };
        buddy_block.set_size(new_size);
        buddy_block.mark_free(true);
        block.set_size(new_size);

        // Place in block will mark buddy as free and put in cooresponding list
        self.place_block_in_list(buddy_block);
    }

    // Takes a marked free block and places it in it's cooresponding list
    fn place_block_in_list(&mut self, block: &mut BlockHeader) {
        let index = self.get_freelist_index(block.get_size() as usize);
        
        let mut current_block = self.free_list[index];
        
        if current_block.is_none() {
            self.free_list[index] = Some(block as *mut BlockHeader);
            block.next = None; // make the next none since this was the first block
                               // in the list.
        } else {
            // at least one block in the free list
            let mut is_placed = false;
            let mut prev : Option<*mut BlockHeader> = None;
            while !is_placed {

                let curr_block = current_block.unwrap();
                
                // The free_lists are organized in ascending addresses. So lower
                // addressed blocks always come first. 
                if curr_block > block as *mut BlockHeader {
                    block.next = current_block;
                    if curr_block == self.free_list[index].unwrap() {
                        // simply prepend to the list. 
                        self.free_list[index] = Some(block as *mut BlockHeader);
                    } else {
                        // prev should be something now as this isn't the first block
                        assert!(!prev.is_none());
                        (unsafe { &mut *(prev.unwrap()) }).next = 
                            Some(block as *mut BlockHeader);
                    }
                    is_placed = true;
                } else if (unsafe {&mut *curr_block}).next.is_none() {
                    // if the next is none, put it after
                    (unsafe {&mut *curr_block}).next = Some(block as *mut BlockHeader);
                    block.next = None;
                    is_placed = true;
                } else {
                    // assert that we're not self looping.
                    assert!(block as *mut BlockHeader != curr_block);
                    
                    // update curr_block and continue trying to place the block.
                    prev = current_block; 
                    current_block = (unsafe{&mut *curr_block}).next;
                }
            }
        }
    }

    /// This function trusts that 'addr' is actually a valid addr that was returned
    /// from an alloc().
    pub fn free(&mut self, addr : usize) {
        // assert that address is in the allocators space!
        assert!(addr >= self.start_address && addr < self.start_address + self.size);
        let block_header : &mut BlockHeader = unsafe { 
                mem::transmute(addr - mem::size_of::<u32>())
        };
        block_header.mark_free(true); 
        
        if !self.coalesce(block_header) {
            // if we couldn't coalecse just add the block to the freelist
            self.place_block_in_list(block_header);
        }
    }

    // Tries to Coalesce 'block' with it's buddy block to form the orignal block
    // that they split from. The buddy block is a very specific block (not any
    // abitarary block of the same size.) See get_buddy for more information.
    fn coalesce(&mut self, block: &mut BlockHeader) -> bool {
        let buddy_block_ptr = self.get_buddy(block);
        
        let mut buddy_block = unsafe { &mut *buddy_block_ptr };

        // if the buddy block is not free, or it's not coalesced itself we can't
        // coalesce!
        if !buddy_block.is_free() || buddy_block.get_size() != block.get_size() {
            return false
        } 

        // buddy is ready to coalesce!
        
        let my_ptr = block as *mut BlockHeader;
       
        
        // remove buddy block
        self.remove_block_from_list(buddy_block);
        
        // Change the header of the lowest addressed block in the set and try to
        // coalesce some more :).
        if my_ptr < buddy_block_ptr {
            block.set_size(buddy_block.get_size() * 2);
            
            // Don't go out of bounds if we've full coalesce back to the original block.
            if block.get_size() as usize == self.size {
                self.place_block_in_list(block);
            } else if !self.coalesce(block) {
                self.place_block_in_list(block);
            }
        } else {
            // change the header of the buddy block, as it comes before
            buddy_block.set_size(block.get_size() * 2);
            
            // Don't go out of bounds!
            if buddy_block.get_size() as usize == self.size {
                self.place_block_in_list(buddy_block);
            } else if !self.coalesce(buddy_block) {
                self.place_block_in_list(buddy_block);
            }
        }
        // we merged.
        true
    }

    // Removes 'block' from it's free_list.
    fn remove_block_from_list(&mut self, block: &mut BlockHeader) {
        let index = self.get_freelist_index(block.get_size() as usize);
        let mut removed = false; // nothing removed yet...
        let target = block as *mut BlockHeader;

        // Assert that there's at least one block in the free_list. There should be
        // if we're going to remove a block from that index.
        assert!(!self.free_list[index].is_none());
        
        let mut current = self.free_list[index].unwrap(); 
        if current == target {
            // Removing the first block in the list.
            self.free_list[index] = (unsafe { &mut *current }).next;
        } else {
            let mut previous = current;
            while !removed {
                current = (unsafe { &mut *current }).next.unwrap();
                if current == target {
                    // remove!
                    (unsafe {&mut *previous}).next = (unsafe { &mut *current}).next;
                    removed = true;
                } 
                previous = current;
            }
        }
    }

    // Calculates the address of the 'blocks' buddy.
    // The buddy of a block with size s at address x is either at s + x or s - x.
    // Because buddies come in pairs, we can use the parity of the block (whether it's
    // even or odd) to identity which block it is and then find it's buddy.
    fn get_buddy(&self, block: &BlockHeader) -> *mut BlockHeader { 
        let buddy_parity = (block as *const BlockHeader as usize - self.start_address) 
                                / block.get_size() as usize;
        if buddy_parity % 2 == 0 {
            // it's the first in the set, so the buddy is after it!
            ((block as *const BlockHeader as usize) + (block.get_size() as usize)) 
                as *mut BlockHeader
        } else {
            // it's the second in the set, so it's buddy is before it~
            ((block as *const BlockHeader as usize) - (block.get_size() as usize)) 
                as *mut BlockHeader
        }
    }   

    // Returns the index in the free_list for the given 'size'.
#[inline(always)]
    fn get_freelist_index(&self, size: usize) -> usize {
        (power_of_two(size) - power_of_two(self.smallest_block_size)) as usize
    }

// Verifies that the linked list structure is correctly ordered (i.e. lowest address
// first, only free blocks in the list, and that the sizes are correct).
#[cfg(test)]
    pub fn verify_lists(&self) {
        for i in 0..(self.free_list_size as usize) {
            let mut current_entry = self.free_list[i];
            while !current_entry.is_none() {
                 let pointer = current_entry.unwrap();
                 let block : &mut BlockHeader = unsafe {
                    mem::transmute(pointer)
                 };

                 // assert block is free and it's size is correct.
                 assert!(block.is_free());
                 assert_eq!(block.get_size(), (self.smallest_block_size as u32) << i);
                
                 if !block.next.is_none() {
                    assert!(pointer < block.next.unwrap()); // assert current pointer
                                                            // is lower-addressed.
                 } 
                 current_entry = block.next; // try next entry;
            }
        }
    }
}

// Tests below.
#[cfg(test)]
mod tests {
    extern crate std;
    extern crate core;
    extern crate rand;
    
    use super::*;
    use core::mem;
    use std::vec;

    #[test]
    fn correct_power_of_twos() {
        assert_eq!(2, super::power_of_two(4));
        assert_eq!(16, super::power_of_two(65536));
    }

    #[test]
    #[should_panic]
    fn incorrect_power_of_two() {
        assert_eq!(0, super::power_of_two(0));
    }

    #[test]
    fn next_power_of_two_test() {
        assert_eq!(4, super::next_power_of_two(3)); // non-power of two
        assert_eq!(2, super::next_power_of_two(2)); // power of two (shouldn't change)
        assert_eq!((1 << 31), super::next_power_of_two((1 << 30) + 1));
    }
    
    // An ode to CS107's Heap Allocator.
#[test]
    fn cs107() {
        let memory : [u8; 65536] = [0; 65536];
        let mut myAllocator = Allocator::new(
            unsafe { mem::transmute(&memory[0]) }, 4096, 1024);
        
        alloc_and_test(&mut myAllocator, 420, true); // should fail ( block size too small)
        alloc_and_test(&mut myAllocator, 4200, true); // should fail ( block size too large)
        
        alloc_and_test(&mut myAllocator, 2036, false);
        alloc_and_test(&mut myAllocator, 2036, false);
        alloc_and_test(&mut myAllocator, 2036, true); // should fail now (no space left)


    }


    // uses the entire memory location
#[test]
    fn full_load() {
        let memory : [u8; 65536] = [0; 65536];
        let mut myAllocator = super::Allocator::new(
            unsafe { mem::transmute(&memory[0]) }, 16384, 1024);
        let mut address_vec : vec::Vec<usize> =  vec::Vec::new();
        let mut curr_req_size = 1024;
        
        println!("Allocating memory...");
        while curr_req_size < 16384 {
            println!("Allocating chunk of size {}", curr_req_size);
            address_vec.push(alloc_and_test(&mut myAllocator, curr_req_size - 4, false).unwrap());
            curr_req_size = curr_req_size << 1;
        }

        // make an extra 1024 request
        address_vec.push(alloc_and_test(&mut myAllocator, 1020, false).unwrap());
        
        // this should fail as all the memroy should be taken...
        alloc_and_test(&mut myAllocator, 1024, true);
        address_vec.reverse();

        println!("Freeing memory");

        // free all starting from the back.
        for addr in address_vec {
            free_and_test(&mut myAllocator, addr);
        }
    }

#[test]
    fn small_blocks() {
        // allocs the whole memory in small blocks...
        let memory : [u8; 65536] = [0; 65536];
        let mut myAllocator = Allocator::new(
            unsafe { mem::transmute(&memory[0]) }, 16384, 512);
        let mut address_vec : vec::Vec<usize> = vec::Vec::new();
        let mut size_left = 16384;
        
        let start_address : usize = unsafe { mem::transmute(&memory[0])};

        println!("Allocating Blocks.... starting at address {}",  start_address);

        while size_left > 0 {
            address_vec.push(alloc_and_test(&mut myAllocator, 508, false).unwrap());
            size_left -= 512;
        }
        
        println!("Freeing blocks..."); 

        while !address_vec.is_empty() {
            let index = (address_vec.len() as f64 * rand::random::<f64>()) as usize;
            let address = address_vec.remove(index);
            println!("Removing block at address {}", address);
            free_and_test(&mut myAllocator, address);
        }

        alloc_and_test(&mut myAllocator, 14201, false); // the entire segement should be
                                                        // consolidated now, so this should
                                                        // work!

    }



    fn free_and_test(allocator: &mut Allocator, address: usize) {
        allocator.free(address);
        allocator.verify_lists();
        let block : &mut super::BlockHeader = unsafe { 
                mem::transmute(address - 4) 
        };
        assert!(block.is_free());
    }

    fn alloc_and_test(allocator: &mut Allocator, size : usize, 
        expected_fail : bool) -> Option<usize> {
        let results = allocator.alloc(size);
        allocator.verify_lists();
        if(expected_fail) {
            assert!(results.is_none());
        } else {
            // verify size and that it's not marked as free.
            let block : &mut super::BlockHeader = unsafe { 
                    mem::transmute(results.unwrap() - 4) 
            };
            assert_eq!(block.get_size(), super::next_power_of_two((size + 4) as u32));

            assert!(!block.is_free());
        }
        results
    }
}
