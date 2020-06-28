//! Interface for touch input devices

use crate::ReturnCode;

#[derive(Debug, Copy, Clone)]
pub enum TouchStatus {
    Pressed,
    Released,
}

#[derive(Debug, Copy, Clone)]
pub enum GestureEvent {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    ZoomIn,
    ZoomOut,
}

#[derive(Copy, Clone)]
pub struct TouchEvent {
    pub status: TouchStatus,
    // touch (x, y) position
    pub x: usize,
    pub y: usize,

    // touch id, value defined by the driver
    pub id: usize,

    // touch area
    pub area: Option<usize>,

    // touch weight
    pub weight: Option<usize>,
}

pub trait Touch {
    // enable touch
    fn enable(&self) -> ReturnCode;

    // disable touch
    fn disable(&self) -> ReturnCode;

    fn set_client(&self, touch_client: &'static dyn TouchClient);
}

pub trait MultiTouch {
    // enable touches
    fn enable(&self) -> ReturnCode;

    // disable touches
    fn disable(&self) -> ReturnCode;

    /// Returns the number of concurently supported touches
    /// This function must be called in the same interrupt
    /// as the event, otherwise data might not be available.
    fn get_num_touches(&self) -> usize;

    /// Returns the touch event at index or None
    /// This function must be called in the same interrupt
    /// as the event, otherwise data might not be available.
    fn get_touch(&self, index: usize) -> Option<TouchEvent>;

    fn set_client(&self, multi_touch_client: &'static dyn MultiTouchClient);
}

pub trait TouchClient {
    fn touch_event(&self, touch_event: TouchEvent);
}

pub trait MultiTouchClient {
    /// num touches represents the number of touches detected
    fn touch_event(&self, num_touches: usize);
}

pub trait Gesture {
    fn set_client(&self, gesture_client: &'static dyn GestureClient);
}

pub trait GestureClient {
    fn gesture_event(&self, gesture_event: GestureEvent);
}
