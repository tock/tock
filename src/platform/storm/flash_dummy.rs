//! A dummy flash client to test flashcalw functionality at the platform level.
/// It can be used to 'kick-off' flashcalw tests by issuing only ONE command that
/// will generate an interrupt.

use sam4l::flashcalw;
use hil::flash::{FlashController, Client, Error};
use core::mem;
use core::cell::Cell;

// ======================================
//  Test the flash controller (using interrupts).
//  This is essentially a state-machine.
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
    page: Cell::new(53), // Page to start
    region_unlocked: Cell::new(0),
    num_cycle_per_page: 2,  // How many times to repeat a Erase/Write/Read cycle on a page
    val_data: Cell::new(2), // Data to write to the page.
    cycles_finished: Cell::new(0)
};

const MAX_PAGE_NUM: i32 = 80;   // Page to go up to

impl Client for FlashClient {

    fn command_complete(&self, error : Error) {
        
        print!("Client Notified that job done in state {}", self.state.get() as u32);
        
        let dev = unsafe { &mut flashcalw::flash_controller };
        
        match self.state.get() {
            FlashClientState::Enabling => {
                if self.region_unlocked.get() == 16 {
                    self.state.set(FlashClientState::EWRCycleStart);
                    println!("\t All Regions unlocked");
                    println!("===========Transitioning \
                        to Erasing/Writing/Reading========");
                    dev.enable_ws1_read_opt(true);
                    // This enabled High Speed Mode using a command
                    // which generates the interrupt for the next stage.
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
                //  Again like WritePageBuffer, this isn't a command. But should be
                //  triggered after the write (hopefully).
                let mut pass = true;
                
                //  Prints out any differences in the flash page. 
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
                self.command_complete(Error::CommandComplete); // we need to call this here as 
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
                            self.command_complete(Error::CommandComplete);
                        }
                    } else {
                        println!("\t Still Cycling page {}", self.page.get());
                        //increment cycle count
                        self.cycles_finished.set(self.cycles_finished.get() + 1);
                        //increment val_data
                        self.val_data.set(self.val_data.get() + 1);
                        //continue cycle
                        self.state.set(FlashClientState::Erasing);
                        self.command_complete(Error::CommandComplete);
                    }
                }
            }

        }
    }

}

// Sets up the testing for the flash driver.
pub fn set_read_write_test() {
    let flashClient = unsafe { &mut FLASH_CLIENT };
    let dev = unsafe { &mut flashcalw::flash_controller };

    dev.set_client(flashClient);
    print!("Calling configure...");
    dev.configure();
    
    //  By default the picocache ( a cache only for the flash) is turned off.
    //  However the bootloader turns it on. I will explicitly turn it on here.
    //  So if the bootloader changes, nothing breaks.
    println!("Is the picocache on? {}", flashcalw::pico_enabled());
    print!("Turning it on then...");
    flashcalw::enable_picocache(true);
    println!("It's on? {}", flashcalw::pico_enabled());
    
    //kicks off the interrupts
    dev.lock_page_region(0, false);

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
