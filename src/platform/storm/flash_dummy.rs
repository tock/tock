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
//  Note: This assumes that all the buses in the config function is on for the
//  entire time.
// ======================================

#[derive(Copy,Clone,PartialEq)]
enum FlashClientState {
    Enabling,
    ClearPageBuffer,
    WritePageBuffer,
    Writing,
    Reading,
    Erasing,
    EWRCycleStart   /* Start of the Erase, Write, Read Cycle */
}

struct FlashClient { 
    state : Cell<FlashClientState>,
    page: Cell<i32>, 
    region_unlocked: Cell<u32>,
    num_cycle_per_page: u32,
    val_data: Cell<u8>,
    cycles_finished: Cell<u32>
}

static mut FLASH_CLIENT : FlashClient = FlashClient { 
    state: Cell::new(FlashClientState::Enabling),
    page: Cell::new(53),
    region_unlocked: Cell::new(0),
    num_cycle_per_page: 2,
    val_data: Cell::new(2),
    cycles_finished: Cell::new(0)
};

const MAX_PAGE_NUM: i32 = 80;

impl Client for FlashClient {

    fn command_complete(&self) {
        
        print!("Client Notified that job done in state {}", self.state.get() as u32);
        
        let dev = unsafe { &mut flashcalw::flash_controller };
        
        match self.state.get() {
            FlashClientState::Enabling => {
                if self.region_unlocked.get() == 16 {
                    self.state.set(FlashClientState::EWRCycleStart);
                    println!("\t All Regions unlocked");
                    println!("===========Transitioning \
                        to Erasing/Writing/Reading========");
                    //self.command_complete();
                    dev.set_flash_waitstate_and_readmode(48000000, 0, false);
                } else {
                    dev.lock_region(self.region_unlocked.get(), false);
                    println!("\t Unlocking Region {}", self.region_unlocked.get()); 
                    self.region_unlocked.set(self.region_unlocked.get() + 1);
                }
            },
            FlashClientState::Writing => {
                println!("\tWriting page {}", self.page.get());
                dev.flashcalw_write_page(self.page.get());
                self.state.set(FlashClientState::Reading);
            },
            FlashClientState::Reading => {
               /* use sam4l::ast::AST;
                use hil::alarm::Alarm;
                unsafe {
                    //implementing a 'wait' essentially...
                    let now = AST.now();
                    let delta = 16000 * 40; // wait 480k cycles ticks from the AST.
                    while( AST.now() - delta < now) {}
                }*/
                use support;
                for i in 0..24_000_000 {
                    //NOP SPIN!
                    support::nop();
                }

                //  Again like WritePageBuffer, this isn't a command. But should be
                //  triggered after the write (hopefully).
                let mut pass = true;
                
                
                println!("\treading page {}", self.page.get());
                let mut data : [u8; 512] = [0;512];
                dev.read_page_raw(self.page.get(), &mut data);
                //verify what we expect
                for i in 0..512 {
                    if( data[i] != self.val_data.get()) {
                        pass = false;
                        println!("\t\t======bit:{} expected {}, got {}========", i, 
                           self.val_data.get(), data[i]);
                        
                    }
                    //assert_eq!(self.val_data.get(), data[i]);
                }
                
                if(!pass) {
                    for j in 0..3 {
                        println!("\treading page {}", self.page.get());
                        let mut data : [u8; 512] = [0;512];
                        dev.read_page_raw(self.page.get(), &mut data);
                        //verify what we expect
                        for i in 0..512 {
                            if data[i] != self.val_data.get() {
                            println!("\t\t\t======bit:{} expected {}, got {}========", i, 
                               self.val_data.get(), data[i]);
                            
                            }
                        }
                    }
                }
                    
                //start cycle again
                self.state.set(FlashClientState::EWRCycleStart);
            },
            FlashClientState::Erasing => {
                println!("\tErasing page {}", self.page.get());
                dev.flashcalw_erase_page(self.page.get(), true);
                self.state.set(FlashClientState::ClearPageBuffer);
            },
            FlashClientState::ClearPageBuffer => { 
                println!("\tClearing page buffer");
                dev.clear_page_buffer();
                self.state.set(FlashClientState::WritePageBuffer);
            },
            FlashClientState::WritePageBuffer => {
                println!("\tWriting to page buffer");
                let data : [u8;512] = [self.val_data.get(); 512];
                dev.write_to_page_buffer(&data, (self.page.get() * 512) as usize);
                self.state.set(FlashClientState::Writing);
                self.command_complete(); // we need to call this here as 
                                         // write_to_page_buffer isn't really a
                                         // command (thus no interrupt generated).
            },
            FlashClientState::EWRCycleStart => {
                if self.page.get() <= MAX_PAGE_NUM {
                    if self.cycles_finished.get() >= self.num_cycle_per_page {
                        //reset count
                        self.cycles_finished.set(1);
                        //increment pg num
                        self.page.set(self.page.get() + 1);
                        //increment val_data
                        self.val_data.set(self.val_data.get() + 1);
                        //start new cycle
                        if self.page.get() <= MAX_PAGE_NUM {
                            self.state.set(FlashClientState::Erasing);
                            println!("==============Starting work on page {} \
                                =================", self.page.get());
                            self.command_complete();
                        }
                    } else {
                        println!("\t Still Cycling page {}", self.page.get());
                        //increment cycle count
                        self.cycles_finished.set(self.cycles_finished.get() + 1);
                        //increment val_data
                        self.val_data.set(self.val_data.get() + 1);
                        //continue cycle
                        self.state.set(FlashClientState::Erasing);
                        self.command_complete();
                    }
                }
            }

        }
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
    print!("Calling configure...");
    dev.configure();
    println!("Is the picocache on? {}", flashcalw::pico_enabled());
    dev.lock_page_region(0, false);

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
    
    println!("\t Testing basic r/w to a page ONCE!");
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
    println!("\tTesting Writing and Reading...");
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
    println!("Writing to the page");
    flashcalw::flash_controller.write_page(page_num as usize, &buff); 
    print!("Going to read buffer?");
    flashcalw::flash_controller.read_page(page_num as usize, &mut read_buffer);
    println!("READ BUFFER");
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
