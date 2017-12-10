use driver::Driver;

pub mod mpu;
pub mod systick;

/// Interface for individual boards.
pub trait Platform {
    /// Platform-specific mapping of syscall numbers to objects that implement
    /// the Driver methods for that syscall
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&Driver>) -> R;
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


//use core::marker::PhantomData;

pub trait MMIOInterface<C> where
    C: ClockInterface,
{
    type MMIORegisterType : ?Sized;
    type MMIOClockType : ClockInterface;

    fn get_hardware_address(&self) -> *mut Self::MMIORegisterType;
    fn get_clock(&self) -> &C;
    fn can_disable_clock(&self, &Self::MMIORegisterType) -> bool;
}

pub struct MMIOManager<'a, H, C> where
    H: 'a + MMIOInterface<C>,
    C: 'a + ClockInterface,
{
    pub registers: &'a H::MMIORegisterType,
    periphal_hardware: &'a H,
}

impl<'a, H, C> MMIOManager<'a, H, C> where
    H: 'a + MMIOInterface<C>,
    C: 'a + ClockInterface,
{
    pub fn new(hw: &'a H) -> MMIOManager<'a, H, C> {
        let clock = hw.get_clock();
        if clock.is_enabled() == false {
            clock.enable();
        }
        MMIOManager {
            registers: unsafe { &* hw.get_hardware_address() },
            periphal_hardware: hw,
        }
    }
}

impl<'a, H, C> Drop for MMIOManager<'a, H, C> where
    H: 'a + MMIOInterface<C>,
    C: 'a + ClockInterface,
{
    fn drop(&mut self) {
        if self.periphal_hardware.can_disable_clock(self.registers) {
            self.periphal_hardware.get_clock().disable();
        }
    }
}
