/// Hardware Interface Layer for flash drivers
use core::fmt::{Display, Formatter, Result};

/// ERROR codes
pub enum Error {
    CommandComplete,
    LockE,
    ProgE,
    LockProgE,
    ECC,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Command {
    Write,
    Read,
    Erase,
    None
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        let display_str = match *self {
            Error::LockE => "Flash Lock Error",
            Error::ProgE => "Flash Programming Error",
            Error::LockProgE => "Flash Lock and Programming Error",
            Error::CommandComplete => "Flash Command Completed",
            Error::ECC => "Flash ECC Error"
        };
        write!(fmt, "{}", display_str)
    }
}

pub trait FlashController {
    //  meta functions
    fn configure(&mut self);

    //  in bytes
    fn get_page_size(&self) -> u32;
    
    //  in # of pages
    fn get_number_pages(&self) -> u32;
    
    //  register's a particular client as the user of the controller.
    fn set_client(&self, &'static Client);
    
    //  Commands

    //  Read_page actually doesn't take a while => will do it synchronously.
    //  This call will fail if size and the buffer's size is different. 
    fn read(&self, address : usize, size: usize, buffer: &mut [u8]) -> i32;
       
    // Write fails if data isn't of size page size.
    fn write_page(&self, page_num : i32, data: & [u8]) -> i32;
    fn erase_page(&self, page_num: i32) -> i32;

}

pub trait Client {
    //  Called upon a completed call
    fn command_complete(&self, err: Error);     
}

