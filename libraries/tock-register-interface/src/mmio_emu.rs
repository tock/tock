//! MMIO backend for emulating register reads and writes.
//!

// This module requires libstd.
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};
use std::collections::BTreeMap;
use std::fmt;
use std::mem::size_of;
use std::slice;
use std::sync::{Arc, Mutex};

use crate::lazy::Lazy;
use crate::registers::IntLike;

/// Errors that can be encountered by MMIO emulation code.
#[derive(Debug)]
pub enum Error {
    RegionOverlaps(usize, usize),
    ZeroSizedRegion(usize),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match self {
            RegionOverlaps(base, size) => write!(
                f,
                "region overlaps with existing region of base {:#x} size {:#x}",
                base, size
            ),
            ZeroSizedRegion(base) => {
                write!(f, "region at base address {:#x} cannot be zero-sized", base)
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct MmioRegion {
    base: usize,
    size: usize,
}

impl MmioRegion {
    /// Returns true if there is overlap with the given range.
    fn overlaps(&self, base: usize, size: usize) -> bool {
        self.base < (base + size) && base < self.base + self.size
    }
}

/// A trait for devices that can be controlled by reading or writing MMIO registers.
pub trait MmioDevice: Send {
    /// Reads at `offset` from this device into `data`.
    fn mmio_read(&mut self, offset: usize, data: &mut [u8]);
    /// Writes `data` at `offset` of this device.
    fn mmio_write(&mut self, offset: usize, data: &[u8]);
}

struct MmioEmu {
    devices: BTreeMap<MmioRegion, Arc<Mutex<dyn MmioDevice>>>,
}

impl MmioEmu {
    /// Creates a new `MmioEmu`.
    fn new() -> Self {
        MmioEmu {
            devices: BTreeMap::new(),
        }
    }

    /// Adds `device` to be associated with the memory region in `region`.
    ///
    /// This may fail if the region is invalid (zero size), or the region conflicts with the
    /// region of a different device.
    fn add_device(&mut self, region: MmioRegion, device: Arc<Mutex<dyn MmioDevice>>) -> Result<()> {
        if region.size == 0 {
            return Err(Error::ZeroSizedRegion(region.base));
        }

        // Reject all cases where the new device's range overlaps with an existing device.
        if self
            .devices
            .iter()
            .any(|(range, _dev)| range.overlaps(region.base, region.size))
        {
            return Err(Error::RegionOverlaps(region.base, region.size));
        }
        let foo = self.devices.insert(region, device);
        match foo {
            Some(_) => Err(Error::RegionOverlaps(region.base, region.size)),
            None => Ok(()),
        }
    }

    /// Finds the first region with a base address before `addr`. This region could contain
    /// `addr`, but only if it is large enough.
    fn first_before(&self, addr: usize) -> Option<(MmioRegion, &Mutex<dyn MmioDevice>)> {
        let (range, dev) = self
            .devices
            .range(
                ..=MmioRegion {
                    base: addr,
                    size: usize::MAX,
                },
            )
            .rev()
            .next()?;
        Some((*range, dev))
    }

    /// Gets the device associated with `addr`, if any.
    ///
    /// If `addr` lies in a register block owned by a device, returns the offset
    /// into that register block and the device. Otherwise, returns None.
    fn get_device(&self, addr: usize) -> Option<(usize, &Mutex<dyn MmioDevice>)> {
        if let Some((range, dev)) = self.first_before(addr) {
            let offset = addr - range.base;
            if offset < range.size {
                return Some((offset, dev));
            }
        }
        None
    }

    /// Reads data from the device that owns the range containing `addr` and puts it into `data`.
    ///
    /// # Panics
    ///
    /// Panics if there is no device associated with the address.
    fn read(&self, addr: usize, data: &mut [u8]) {
        let (offset, dev) = self
            .get_device(addr)
            .expect("mmio_emu: read: no device for address");
        dev.lock().unwrap().mmio_read(offset, data);
    }

    /// Writes `data` to the device that owns the range containing `addr`.
    ///
    /// # Panics
    ///
    /// Panics if there is no device associated with the address.
    fn write(&self, addr: usize, data: &[u8]) {
        let (offset, dev) = self
            .get_device(addr)
            .expect("mmio_emu: write: no device for address");
        dev.lock().unwrap().mmio_write(offset, data);
    }
}

static MMIO_STATE: Lazy<Mutex<MmioEmu>> = Lazy::new();

fn get_mmio_state() -> &'static Mutex<MmioEmu> {
    MMIO_STATE.get(|| Mutex::new(MmioEmu::new()))
}

/// Registers `device` to be associated with the static register block `registers`.
///
/// # Requirements
///
/// Registering a device for emulation requires a static item for the register block to be
/// emulated. This allows the MMIO region for a device to be allocated globally, which is necessary
/// for drivers that typically expect a `'static` lifetime for registers.
///
/// Devices must implement the `MmioDevice` trait. Devices implementing this trait should try to
/// handle all potential register reads and writes. Unhandled reads will become register reads with
/// value 0. Unhandled writes will be dropped.
///
/// # Safety
///
/// Registering an MMIO device is safe. However, users of this function will also need to create
/// a static item for the register block, which will likely involve unsafe code. There are normally
/// no constructors for register blocks, so instantation in device code will manifest as creating a
/// reference from a pointer to an MMIO region, an unsafe operation.
///
/// Test code could use `MaybeUninit<RegisterBlock>` to allocate a static item of the required size.
/// Creating a reference to the `RegisterBlock` for driver code to use in testing is (according to
/// the `MaybeUninit` docs) undefined behavior and could exhibit soundness issues. In practice,
/// this should not be the case as long as any `RegisterBlock` struct consists only of fields from
/// the `tock-registers` crate, as these will call into MMIO emulation and will never access the
/// uninitialized memory.
///
/// # Panics
///
/// Using a register block without calling `register_mmio_device` on that block first will cause
/// a panic.
///
/// # Example usage
///
/// ```rust
/// # use std::mem::MaybeUninit;
/// # use std::sync::{Arc, Mutex};
/// # use tock_registers::registers::*;
/// # use tock_registers::mmio;
/// # #[repr(C)]
/// # struct FooRegisters { _unused: ReadWrite<u64> }
/// # struct FooDevice { _unused: u64 }
/// # impl FooDevice {
/// #     fn new() -> Self {
/// #         FooDevice { _unused: 0u64 }
/// #     }
/// # }
/// # impl mmio::MmioDevice for FooDevice {
/// #     fn mmio_read(&mut self, offset: usize, data: &mut [u8]) {}
/// #     fn mmio_write(&mut self, offset: usize, data: &[u8]) {}
/// # }
/// #
/// static FOO_REGS: MaybeUninit<FooRegisters> = MaybeUninit::uninit();
/// let _regs = unsafe { &*FOO_REGS.as_ptr() };
/// let device = Arc::new(Mutex::new(FooDevice::new()));
/// mmio::register_mmio_device(device, &FOO_REGS).unwrap();
/// ```
pub fn register_mmio_device<R: 'static>(
    device: Arc<Mutex<dyn MmioDevice>>,
    registers: &'static R,
) -> Result<()> {
    let region = MmioRegion {
        base: registers as *const R as usize,
        size: size_of::<R>(),
    };

    let mut mmio_state = get_mmio_state().lock().unwrap();
    mmio_state.add_device(region, device)?;

    Ok(())
}

/// Emulates an MMIO volatile read of `T`.
///
/// `src` must be a pointer to a field in a register block registered with `register_mmio_device`.
///
/// # Panics
///
/// Panics if there is no device associated with the address.
pub(crate) unsafe fn read_volatile<T: IntLike>(src: *const T) -> T {
    let mut result = T::zero();
    let bytes_slice = slice::from_raw_parts_mut(&mut result as *mut T as *mut u8, size_of::<T>());
    let mmio_state = get_mmio_state().lock().unwrap();
    mmio_state.read(src as usize, bytes_slice);
    result
}

/// Emulates an MMIO volatile write of `T`.
///
/// `dst` must be a pointer to a field in a register block registered with `register_mmio_device`.
///
/// # Panics
///
/// Panics if there is no device associated with the address.
pub(crate) unsafe fn write_volatile<T: IntLike>(dst: *mut T, src: T) {
    let bytes_slice = slice::from_raw_parts(&src as *const T as *const u8, size_of::<T>());
    let mmio_state = get_mmio_state().lock().unwrap();
    mmio_state.write(dst as usize, bytes_slice);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registers::*;
    use std::convert::TryInto;
    use std::mem::MaybeUninit;

    /// MMIO register interface for a monotonically increasing counter device.
    #[repr(C)]
    struct CounterRegisters {
        /// Current counter value.
        counter: ReadOnly<u64>,
        /// Writing to this register increments the counter value by the amount in `increment`.
        increment: WriteOnly<u32>,
    }

    /// Internal state of the counter device.
    struct CounterDevice {
        value: u64,
    }

    impl CounterDevice {
        /// Creates a new counter device with an initial counter value of 0.
        fn new() -> Self {
            CounterDevice { value: 0u64 }
        }
    }

    impl MmioDevice for CounterDevice {
        fn mmio_read(&mut self, offset: usize, data: &mut [u8]) {
            match offset {
                0x0 => data.copy_from_slice(&self.value.to_ne_bytes()),
                _ => panic!("CounterDevice: illegal read offset {:#x}", offset),
            }
        }

        fn mmio_write(&mut self, offset: usize, data: &[u8]) {
            match offset {
                0x8 => {
                    let increment = u32::from_ne_bytes(data.try_into().unwrap());
                    self.value += u64::from(increment);
                }
                _ => panic!("CounterDevice: illegal write offset {:#x}", offset),
            }
        }
    }

    #[test]
    fn counter_device() {
        static FAKE_REGS: MaybeUninit<CounterRegisters> = MaybeUninit::uninit();
        let device = Arc::new(Mutex::new(CounterDevice::new()));
        register_mmio_device(device, &FAKE_REGS).unwrap();

        let regs = unsafe { &*FAKE_REGS.as_ptr() };

        assert_eq!(regs.counter.get(), 0);
        regs.increment.set(5);
        assert_eq!(regs.counter.get(), 5);
        regs.increment.set(1);
        assert_eq!(regs.counter.get(), 6);
    }
}
