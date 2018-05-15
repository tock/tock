use common::list::*;
use core::cell::Cell;

pub trait ClockClient<'a> {
    /// This function will by called by ClockManager's register function 
    ///     Indicates the peripheral should turn on clock management
    fn enable_cm(&self);
    /// The ClockManager will call this function to report a clock change
    fn clock_updated(&self, clock_changed: bool);
    fn get_params(&self) -> Option<&ClockParams>;
    fn next_link(&'a self) -> &'a ListLink<'a, ClockClient<'a> + 'a>;
}

impl<'a> ListNode<'a, ClockClient<'a> + 'a> for ClockClient<'a> + 'a {
   ///  
    fn next(&'a self) -> &'a ListLink<'a, ClockClient<'a> + 'a> {
        &self.next_link()
    }
}

pub struct ClockParams {
    /// clocklist: bitmask of clocks the client can operate with
    pub clocklist: Cell<u32>, 
    /// min_freq: minimum operational frequency
    pub min_frequency: Cell<u32>, 
    /// max_freq: maximum operational frequency
    pub max_frequency: Cell<u32>, 
    // thresh_freq: frequency above which increasing the clock does not help
    //pub thresh_frequency: Cell<u32>,
}

impl ClockParams {
    pub const fn new(clocklist: u32, min_frequency: u32, 
        max_frequency: u32) -> ClockParams{
        ClockParams{
            clocklist: Cell::new(clocklist),
            min_frequency: Cell::new(min_frequency),
            max_frequency: Cell::new(max_frequency),
            //thresh_frequency: Cell::new(thresh_frequency),
        }
    }
}

pub trait ClockManager<'a> {
    /// Clients should call this function to update the ClockManager
    /// on which clocks they can tolerate
    ///
    fn register(&mut self, c:&'a ClockClient<'a>);
    fn lock(&mut self)->bool;
    fn unlock(&mut self);
    fn need_clock_change(&self, params:&ClockParams)->bool;
    fn clock_change(&mut self);
}
