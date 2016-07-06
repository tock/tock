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
    // meta functions
    
    fn configure(&mut self);
    // in bytes
    fn get_page_size(&self) -> u32;
    // in # of pages
    fn get_number_pages(&self) -> u32;
    
    //commands
    
    // The three functions below will need to be used with a subscribe CB
    // as they might take a while...
    fn read_page(&self, addr : usize, mut buffer: &mut [usize]);
    fn write_page(&self, addr : usize, data: & [u8]);
    fn erase_page(&self, page_num: i32);
}

pub trait Client {
    fn command_complete(&self);     
}

