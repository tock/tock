/// A dummy flash client to test flashcalw functionality at the platofrm level (so board/chip specific)
use sam4l::flashcalw;
use hil::flash::{FlashController, Client};
use core::mem;
use core::cell::Cell;

// put any 'let me try this' type of test code here...
pub unsafe fn scratch_test() {
    //testing with QPR
    assert_eq!(flashcalw::flash_controller.quick_page_read(40), false);
    assert_eq!(flashcalw::flash_controller.quick_page_read(41), false);
    assert_eq!(flashcalw::flash_controller.quick_page_read(42), false);

    println!("\tTesting erase, quick read and read");
    //write_and_read(50, 12);
    /*let x = write_and_read(50, 4);*/

    let mut x = [0usize; 128];
    flashcalw::flash_controller.read_page(50, &mut x);
    let trans : [u8; 512] = mem::transmute(x);
    for i in 0..512 {
        println!("Val of {}th trans is:{}",i, trans[i]);
    }

    test_erase(50);

    flashcalw::flash_controller.read_page(50, &mut x);
    let trans : [u8; 512] = mem::transmute(x);
    for i in 0..512 {
        println!("Val of {}th trans is:{}",i, trans[i]);
    }
    //test_erase(41);


    
}

// ======================================
//  Test the flash controller (using interrupts).
// ======================================

#[derive(Copy,Clone,PartialEq)]
enum FlashClientState {
    Enabling,
    Writing,
    Reading,
    Erasing
}

struct FlashClient { state : Cell<FlashClientState>, page: Cell<u32> }

static mut FLASH_CLIENT : FlashClient = 
    FlashClient { state: Cell::new(FlashClientState::Enabling), page: Cell::new(40) };

impl Client for FlashClient {
    fn command_complete(&self) {
        println!("Client Notified that job done...");
    }

    fn is_configuring(&self) -> bool {
        /*self.state.map(|value| {
            value == FlashClientState::Enabling
        });*/
        self.state.get() == FlashClientState::Enabling
    }
}

pub fn set_read_write_test() {
    let flashClient = unsafe { &mut FLASH_CLIENT };
    let dev = unsafe { &mut flashcalw::flash_controller };

    dev.set_client(flashClient);
    println!("Calling configure...");
    dev.configure();

}












// tests the flash driver for the flashcalw...
pub unsafe fn flash_dummy_test() {
    
    println!("Flashcalw Sam4L testing beginning...");
    
    println!("Ready interrupt is on? {}", 
        flashcalw::flash_controller.is_ready_int_enabled());
    println!("Enabling Ready Interrupt...");
    flashcalw::flash_controller.enable_ready_int(true);
    println!("Configuring...");
    flashcalw::flash_controller.configure();
    println!("Configured!");
    println!("Setting Wait State...");
    flashcalw::flash_controller.set_flash_waitstate_and_readmode(48000000, 0, false);
    println!("Wait State Set");
    println!("And the controller is available right? {}", flashcalw::flash_controller.is_ready());
    //flashcalw::flash_controller.set_wait_state(1);

    //unlock any locks on the flash to all writing...
    flashcalw::flash_controller.lock_all_regions(false);
    

    //println!("Disabling PicoCache....");
    //flashcalw::enable_picocache(false);
    println!("Testing PicoCache...{}", flashcalw::pico_enabled());

    //test_meta_info();

    println!("Testing Read, Write and Erase");
    
    println!("\tTesting basic r/w to a page ONCE!");
    for i in 40u8..61 {
        test_read_write(i as i32, i - 40);
    }
    //println!("\tPassed basic r/w ONCE!");
    
    //test_erase(60);
    write_page_std(60, 4);
    //test_read_write(60, 4);
    //test_erase_read(40);

    //scratch_test();
    /* 
    println!("\tTesting erase & read");
    for i in 60u8..39 {
        test_erase_read(i as i32);
    }
    
    println!("\tTesting basic r/w to a page ONCE!");
    for i in 40u8..61 {
        test_read_write(i as i32, i - 40);
    }
    println!("\tPassed basic r/w ONCE!");
    println!("\tPassed erase & read!");
    
    println!("All literate! Passed Read, Write and Erase!");

    println!("Done testing Sam4L Flashcalw.");
    */
}


/// This function primarily tests meta information for the chip on the 
/// the FireStorm - ATSAM4LC8C. For other ATSAM4L chips, calculations using the 
/// flash size asserts might fail (as they might not have the same flash size).
pub unsafe fn meta_test() {
    println!("Testing Meta Info...");
    assert_eq!(flashcalw::flash_controller.get_page_size(), 512);
    assert_eq!(flashcalw::flash_controller.get_flash_size(), 512 << 10);
    assert_eq!(flashcalw::flash_controller.get_number_pages(), 1024);
    assert_eq!(flashcalw::flash_controller.get_page_count_per_region(), 64);
    let mut pg_num = 0u32;
    for i in 0..16 {
        assert_eq!(flashcalw::flash_controller.get_page_region(pg_num as i32), i);
        assert_eq!(flashcalw::flash_controller.get_region_first_page_number(i),
            pg_num);
        pg_num += 64;
    }
    println!("Passed Meta Info...");
}

/// Erases page page_num and test whether both QPR and Read confirm it's been erased...
pub unsafe fn test_erase(page_num : i32) {
    flashcalw::flash_controller.enable_ahb();
    let qpr_read = flashcalw::flash_controller.flashcalw_erase_page(page_num, true);
    if !qpr_read { assert!(false); }
    
    println!("Erased page {} with error code:{}.", page_num, 
        flashcalw::flash_controller.debug_error_status());

    let mut read_buffer = [0usize; 128];
    let expected_bit_pattern = [255u8; 512];
    flashcalw::flash_controller.read_page(page_num as usize, &mut read_buffer);
    
    let output_bits : [u8;512]  = mem::transmute(read_buffer);

    //compare raw bits
    for i in 0..512 {
        assert_eq!(expected_bit_pattern[i], output_bits[i]);
    }
}

/// Erases a page and tests whether the page is now 'erased'.
pub unsafe fn test_erase_read(page_num : i32) {
    flashcalw::flash_controller.erase_page(page_num);
    
    let mut read_buffer = [0usize; 128];
    let expected_bit_pattern = [255u8; 512];
    flashcalw::flash_controller.read_page(page_num as usize, &mut read_buffer);
    
    let output_bits : [u8;512]  = mem::transmute(read_buffer);

    //compare raw bits
    for i in 0..512 {
        assert_eq!(expected_bit_pattern[i], output_bits[i]);
    }
}

/// Writes some u8 value repeatedly to a page, and see tests whether
/// the same pattern is read.
pub unsafe fn test_read_write(page_num : i32, value : u8) {
    let expected_bit_pattern = [value; 512];
    let read_val = write_and_read(page_num, value);
    let output_bits : [u8;512]  = mem::transmute(read_val);

    //compare raw bits
    for i in 0..512 {
        //println!("Testing bit {}", i);
        assert_eq!(expected_bit_pattern[i], output_bits[i]);
    }
}

/// Writes a particular value to some page and returns what is read from that
/// value.
pub unsafe fn write_and_read(page_num : i32, value : u8) -> [usize;128] {
    let buff = [value; 512];
    let mut read_buffer = [0usize; 128];
    //write to the page...
    flashcalw::flash_controller.write_page(page_num as usize, &buff); 
    flashcalw::flash_controller.read_page(page_num as usize, &mut read_buffer);
    read_buffer
}

pub unsafe fn write_page(page_num : i32, value : u8) { 
    let data = [value; 512];
    
    //enable clock incase it's off
    flashcalw::flash_controller.enable_ahb();

    //erase page
    flashcalw::flash_controller.erase_page(page_num);
    println!("Erased page {}, with error:{}", page_num,
        flashcalw::flash_controller.debug_error_status());
    
    flashcalw::flash_controller.clear_page_buffer();
    
    //write to page buffer @ 0x0
    flashcalw::flash_controller.write_to_page_buffer(&data, page_num as usize * 512);

    //TODO addr is being treted as pgnum here...

    //issue write command to write the page buffer to some specific page!
    flashcalw::flash_controller.flashcalw_write_page( page_num); 
    
    println!("Written to page {}, with error:{}", page_num,
        flashcalw::flash_controller.debug_error_status());
        
    let mut read_buffer = [0usize; 128];

    //computationally intensive thing...lol or sleep tbh..
    /*for i in 0..10000 {
        println!("wasting time to show the race!");
        do_fib(80);
    }*/
    //do_fib(50);

    flashcalw::flash_controller.read_page(page_num as usize, &mut read_buffer);
   
    let output_bits : [u8;512]  = mem::transmute(read_buffer);

    //compare raw bits
    for i in 0..512 {
        println!("expected:{}, got bits:{}", data[i], output_bits[i]);
        //assert_eq!(data[i], output_bits[i]);
    }
    
}

// More or less write_page but uses write pg from library..
pub unsafe fn write_page_std(page_num : i32, value : u8) { 
    let expected : [u8; 512] = [value; 512];
    //computationally intensive thing...lol or sleep tbh..
    /*for i in 0..10000 {
        println!("wasting time to show the race!");
        do_fib(80);
    }*/
    //do_fib(50);

    let mut read_buffer = write_and_read(page_num, value);

    let output_bits : [u8;512]  = mem::transmute(read_buffer);

    //compare raw bits
    for i in 0..512 {
        println!("expected:{}, got bits:{}", expected[i], output_bits[i]);
        //assert_eq!(data[i], output_bits[i]);
    }
    
}
