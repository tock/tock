use core::cell::Cell;
use process::{AppId, Callback, AppSlice, Shared};
use hil::Driver;
use hil::flash::{Error, Client, FlashController};

pub struct Flash<'a, F: FlashController + 'a> {
    controller: &'a mut F,
    page_size: u32,
    flash_size: u32,
    busy: Cell<bool>
}


impl<'a, F: FlashController> Flash <'a, F> {
    pub fn new(controller: &'a mut F) -> Flash<'a, F> {
        Flash {
            controller: controller,
            page_size: 0,
            flash_size: 0,
            busy: Cell::new(true)
        }
    }

    pub fn configure(&mut self){ 
        self.controller.configure(); // do any configurations necessary for the
                                     // specific flash driver (i.e. check clocks, etc.)
        self.page_size = self.controller.get_page_size();
        self.flash_size = self.controller.get_flash_size();
        self.busy.set(false); // become available after configuration
    } 
}

//client implementation
impl<'a, F: FlashController> Client for Flash<'a, F> {
    fn command_complete(&self) {
        unimplemented!()
    }        
}

//driver implementation
impl<'a, F: FlashController> Driver for Flash<'a, F> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        unimplemented!()    
    }

    fn command(&self, command_num: usize, data: usize, _: AppId) -> isize {
        match command_num {
            0 /* read_page  */ => {
                1             
            },
            1 /* write_page */  => {
                1
            },
            2 /* erase_page */ => {
                1
            },
            3 /* current_page */ => {
                1
            },
            _ => { -1 }
        }
    }

    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> isize {
        unimplemented!()    
    }
    
}
