//! A dummy flash client to test flashcalw functionality at the platform level.
//! It can be used to 'kick-off' flashcalw tests by issuing only ONE command that
//! will generate an interrupt.

use core::cell::Cell;
use sam4l::flashcalw;
use sam4l::flashcalw::{Error, Client};

// ======================================
//  Test the flash controller (using interrupts).
//  This is essentially a state-machine.
//  Note: This assumes that all the buses in the config function is on for the
//  entire time.
// ======================================

#[allow(dead_code)]
#[derive(Copy,Clone,PartialEq)]
enum FlashClientState {
    ClearPageBuffer,
    WritePageBuffer,
    Writing,
    Reading,
    Erasing,
    EWRCycleStart, // Start of the Erase, Write, Read Cycle
}

struct FlashClient {
    state: Cell<FlashClientState>,
    page: Cell<i32>,
    num_cycle_per_page: u32,
    val_data: Cell<u8>,
    cycles_finished: Cell<u32>,
}

static mut FLASH_CLIENT: FlashClient = FlashClient {
    state: Cell::new(FlashClientState::EWRCycleStart),
    page: Cell::new(53), // Page to start
    num_cycle_per_page: 2, // How many times to repeat a Erase/Write/Read cycle on a page
    val_data: Cell::new(2), // Data to write to the page.
    cycles_finished: Cell::new(0),
};

const MAX_PAGE_NUM: i32 = 80;   // Page to go up to

impl Client for FlashClient {
    fn command_complete(&self, _error: Error) {

        print!("Client Notified that job done in state {}",
               self.state.get() as u32);

        let dev = unsafe { &mut flashcalw::flash_controller };

        match self.state.get() {
            FlashClientState::Writing => {
                println!("\tWriting page {}", self.page.get());
                let data: [u8; 512] = [self.val_data.get(); 512];
                dev.write_page(self.page.get(), &data);
                self.state.set(FlashClientState::Reading);
            }
            FlashClientState::Reading => {
                //  The callback from completing the write will lead to this being
                //  called.
                let mut pass = true;

                //  Prints out any differences in the flash page.
                println!("\treading page {}", self.page.get());
                let mut data: [u8; 512] = [0; 512];
                dev.read(self.page.get() as usize * 512, 512, &mut data);

                // verify what we expect
                for i in 0..512 {
                    if data[i] != self.val_data.get() {
                        pass = false;
                        println!("\t\t======bit:{} expected {}, got {}========",
                                 i,
                                 self.val_data.get(),
                                 data[i]);

                    }
                }

                //  If there's any discrepancies read the page 3x more and output
                //  differences.
                if !pass {
                    for _ in 0..3 {
                        println!("\treading page {}", self.page.get());
                        let mut data: [u8; 512] = [0; 512];
                        dev.read(self.page.get() as usize * 512, 512, &mut data);
                        // verify what we expect
                        for i in 0..512 {
                            if data[i] != self.val_data.get() {
                                println!("\t\t\t======bit:{} expected {}, got {}========",
                                         i,
                                         self.val_data.get(),
                                         data[i]);

                            }
                        }
                    }
                }

                // start cycle again
                self.state.set(FlashClientState::EWRCycleStart);
                // call self as reading isn't a callback...
                self.command_complete(Error::CommandComplete);
            }
            FlashClientState::Erasing => {
                println!("\tErasing page {}", self.page.get());
                dev.erase_page(self.page.get());
                self.state.set(FlashClientState::Writing);
            }
            FlashClientState::EWRCycleStart => {
                if self.page.get() <= MAX_PAGE_NUM {
                    if self.cycles_finished.get() >= self.num_cycle_per_page {
                        // reset count
                        self.cycles_finished.set(1);
                        // increment pg num
                        self.page.set(self.page.get() + 1);
                        // increment val_data
                        self.val_data.set(self.val_data.get() + 1);
                        // start new cycle
                        if self.page.get() <= MAX_PAGE_NUM {
                            self.state.set(FlashClientState::Erasing);
                            println!("==============Starting work on page {} \
                                =================",
                                     self.page.get());
                            self.command_complete(Error::CommandComplete);
                        }
                    } else {
                        println!("\t Still Cycling page {}", self.page.get());
                        // increment cycle count
                        self.cycles_finished.set(self.cycles_finished.get() + 1);
                        // increment val_data
                        self.val_data.set(self.val_data.get() + 1);
                        // continue cycle
                        self.state.set(FlashClientState::Erasing);
                        self.command_complete(Error::CommandComplete);
                    }
                }
            }
            _ => {
                panic!("Should never reach here!");
            }

        }
    }
}

// Sets up the testing for the flash driver.
pub fn set_read_write_test() {
    let flash_client = unsafe { &mut FLASH_CLIENT };
    let dev = unsafe { &mut flashcalw::flash_controller };

    dev.set_client(flash_client);
    print!("Calling configure...");
    dev.configure();
    println!("Is the picocache on? {}",
             if dev.pico_enabled() { "yes" } else { "no" });

    // generates an interrupt which will cause a callback after initalization.
    dev.lock_page_region(0, false);

}

/// This function primarily tests meta information for the chip on the
/// the FireStorm - ATSAM4LC8C. For other ATSAM4L chips, calculations using the
/// flash size asserts might fail (as they might not have the same flash size).
#[allow(unused_unsafe)]
pub unsafe fn meta_test() {
    println!("Testing Meta Info...");
    assert_eq!(flashcalw::flash_controller.get_page_size(), 512);
    assert_eq!(flashcalw::flash_controller.get_flash_size(), 512 << 10);
    assert_eq!(flashcalw::flash_controller.get_number_pages(), 1024);
    assert_eq!(flashcalw::flash_controller.get_page_count_per_region(), 64);
    let mut pg_num = 0u32;
    for i in 0..16 {
        assert_eq!(flashcalw::flash_controller.get_page_region(pg_num as i32),
                   i);
        assert_eq!(flashcalw::flash_controller.get_region_first_page_number(i),
                   pg_num);
        pg_num += 64;
    }
    println!("Passed Meta Info...");
}
