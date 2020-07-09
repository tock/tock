//! Interface for touch input devices

use crate::ReturnCode;

/// Touch Event Status
#[derive(Debug, Copy, Clone)]
pub enum TouchStatus {
    Pressed,
    Released,
}

/// Gesture event
#[derive(Debug, Copy, Clone)]
pub enum GestureEvent {
    SwipeUp,
    SwipeDown,
    SwipeLeft,
    SwipeRight,
    ZoomIn,
    ZoomOut,
}

/// A single touch event's data
#[derive(Copy, Clone)]
pub struct TouchEvent {
    pub status: TouchStatus,
    /// touch (x, y) position
    pub x: usize,
    pub y: usize,

    /// touch id, value defined by the driver
    pub id: usize,

    /// touch area
    pub area: Option<usize>,

    /// touch weight
    pub weight: Option<usize>,
}

/// Single touch panels should implement this
pub trait Touch {
    /// Enable the touche panel
    fn enable(&self) -> ReturnCode;

    /// Disable the touch panel
    fn disable(&self) -> ReturnCode;

    /// Set the touch client
    fn set_client(&self, touch_client: &'static dyn TouchClient);
}

/// Multi-touch panels should implement this
pub trait MultiTouch {
    /// Enable the touche panel
    fn enable(&self) -> ReturnCode;

    /// Disable the touch panel
    fn disable(&self) -> ReturnCode;

    /// Returns the number of concurently supported touches
    /// This function must be called in the same interrupt
    /// as the event, otherwise data might not be available.
    fn get_num_touches(&self) -> usize;

    /// Returns the touch event at index or `None`.
    ///
    /// This function must be called in the same interrupt
    /// as the event, otherwise data might not be available.
    fn get_touch(&self, index: usize) -> Option<TouchEvent>;

    /// Set the multi-touch client
    fn set_client(&self, multi_touch_client: &'static dyn MultiTouchClient);
}

/// The single touch client
pub trait TouchClient {
    /// Report a touch event
    fn touch_event(&self, touch_event: TouchEvent);
}

/// The multi touch client
pub trait MultiTouchClient {
    /// Report a multi touch event
    /// num touches represents the number of touches detected
    fn touch_events(&self, touch_events: &[TouchEvent], len: usize);
}

/// Touch panels that support gestures
pub trait Gesture {
    /// Set the gesture client
    fn set_client(&self, gesture_client: &'static dyn GestureClient);
}

/// The gesture client
pub trait GestureClient {
    fn gesture_event(&self, gesture_event: GestureEvent);
}
