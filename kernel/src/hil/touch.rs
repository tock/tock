//! Interface for touch input devices

use crate::ReturnCode;

pub trait Touch {
    /// Subscribe to one of the touches
    fn subscribe_to_touch(id: usize) -> ReturnCode;

    /// Subscribe to all touches
    fn subscribe_to_all() -> ReturnCode;

    /// Retruns the number of concurently supported touches
    fn get_num_touches() -> ReturnCode;
}

pub trait TouchClient {
    fn touch_down(id: usize, x: usize, y: usize);
    fn touch_move(id: usize, x: usize, y: usize);
    fn touch_up(id: usize, x: usize, y: usize);
}

pub trait Gesture {}
