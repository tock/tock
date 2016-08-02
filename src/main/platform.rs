use driver::Driver;

pub trait Platform {
    fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R where
            F: FnOnce(Option<&Driver>) -> R;
}

pub trait Chip {
    type MPU : MPU;
    type SysTick: SysTick;

    fn service_pending_interrupts(&mut self);
    fn has_pending_interrupts(&self) -> bool;
    fn mpu(&self) -> &Self::MPU;
    fn systick(&self) -> &Self::SysTick;
}

pub trait MPU {
    /// Enables MPU, allowing privileged software access to the default memory
    /// map.
    fn enable_mpu(&self);

    /// Sets the base address, size and access attributes of the given MPU
    /// region number.
    ///
    /// `region_num`: an MPU region number 0-7
    /// `start_addr`: the region base address. Lower bits will be masked
    ///               according to the region size. 
    /// `len`       : region size as a function 2^(len + 1)
    /// `execute`   : whether to enable code execution from this region
    /// `ap`        : access permissions as defined in Table 4.47 of the user
    ///               guide.
    fn set_mpu(&self, region_num: u32, start_addr: u32, len: u32,
               execute: bool, ap: u32);
}

/// Noop implementation of MPU trait
impl MPU for () {
    fn enable_mpu(&self) {}

    fn set_mpu(&self, _: u32, _: u32, _: u32, _: bool, _: u32) {}
}

pub trait SysTick {
    /// Sets the timer as close as possible to the given interval in
    /// microseconds.  The clock is 24-bits wide and specific timing is
    /// dependent on the driving clock. Increments of 10ms are most accurate
    /// and, in practice 466ms is the approximate maximum.
    fn set_timer(&self, us: u32);

    /// Returns the time left in approximate microseconds
    fn value(&self) -> u32;


    fn overflowed(&self) -> bool;

    fn reset(&self);

    fn enable(&self, with_interrupt: bool);

    fn overflow_fired() -> bool;
}

impl SysTick for () {

    fn reset(&self) {}

    fn set_timer(&self, _: u32) {}

    fn enable(&self, _: bool) {}

    fn overflowed(&self) -> bool { false }

    fn value(&self) -> u32 { !0 }

    fn overflow_fired() -> bool { false }
}

