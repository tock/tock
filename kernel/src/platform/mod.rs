use driver::Driver;

pub mod mpu;
pub mod systick;

/// Interface for individual boards.
pub trait Platform {
    /// Platform-specific mapping of syscall numbers to objects that implement
    /// the Driver methods for that syscall
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R where F: FnOnce(Option<&Driver>) -> R;
}

/// Interface for individual MCUs.
pub trait Chip {
    type MPU: mpu::MPU;
    type SysTick: systick::SysTick;

    fn service_pending_interrupts(&mut self);
    fn has_pending_interrupts(&self) -> bool;
    fn mpu(&self) -> &Self::MPU;
    fn systick(&self) -> &Self::SysTick;
    fn prepare_for_sleep(&self) {}
}

/// Generic operations that clock-like things are expected to support.
pub trait ClockInterface {
    type PlatformClockType;

    fn is_enabled(&self) -> bool;
    fn enable(&self);
    fn disable(&self);
}


use core::marker::PhantomData;

pub trait MMIOInterface<I, C> where
    I: ?Sized + ClockInterface<PlatformClockType=C>,
    C: ,
{
    type MMIORegisterType : ?Sized;
    type MMIOClockType : ?Sized + ClockInterface<PlatformClockType=C>;

    fn get_hardware_address(&self) -> *mut Self::MMIORegisterType;
    //fn get_clock(&self) -> Option<&Self::MMIOClockType>;
    fn get_clock(&self) -> &I;
}

pub struct MMIOManager<'a, H, R, I, C> where
    H: 'a + ?Sized + MMIOInterface<I, C, MMIORegisterType=R>,
    R: 'a + ?Sized,
    I: 'a + ?Sized + ClockInterface<PlatformClockType=C>,
    C: 'a,
{
    pub registers: &'a R,
    //clock: Option<&'a I>,
    clock: &'a I,
    // We don't actually store ref to PeriphHW struct, but do need to go
    // through it during construction, so need its type bound
    phantom: PhantomData<&'a H>,
}

impl<'a, H, R, I, C> MMIOManager<'a, H, R, I, C> where
    H: 'a + MMIOInterface<I, C, MMIORegisterType=R>,
    R: 'a,
    I: 'a + ClockInterface<PlatformClockType=C>,
    C: 'a,
{
    pub fn new(hw: &'a H) -> MMIOManager<'a, H, R, I, C> {
        let clock = hw.get_clock();
        if clock.is_enabled() == false {
            clock.enable();
        }
        MMIOManager {
            registers: unsafe { &* hw.get_hardware_address() },
            clock: hw.get_clock(),
            phantom: PhantomData,
        }
    }
}
