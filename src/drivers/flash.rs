use core::cell::Cell;
use process::{AppId, Callback, AppSlice, Shared};
use hil::Driver;
use hil::flash::{Error, Client, FlashController};

pub struct Flash<'a, F: FlashController + 'a> {
    controller: &'a mut F,
    page_size: u32,
    num_pages: u32,
    busy: Cell<bool>
}


impl<'a, F: FlashController> Flash <'a, F> {
    pub fn new(controller: &'a mut F) -> Flash<'a, F> {
        Flash {
            controller: controller,
            page_size: 0,
            num_pages: 0,
            busy: Cell::new(true)
        }
    }

    pub fn configure(&mut self){ 
        self.controller.configure(); // do any configurations necessary for the
                                     // specific flash driver (i.e. check clocks, etc.)
        self.page_size = self.controller.get_page_size();
        self.num_pages = self.controller.get_number_pages();
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
                let mut x : [usize;129] = [0;129];
                self.controller.read_page(data, &mut x);
                1             
            },
            1 /* write_page */  => {
                let x : [u8;512] = [0;512];
                self.controller.write_page(data, &x); 
                1
            },
            2 /* erase_page */ => {
                self.controller.erase_page(data as i32);
                1
            },
            _ => { -1 }
        }
    }

    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> isize {
        unimplemented!()    
    }
    
}
