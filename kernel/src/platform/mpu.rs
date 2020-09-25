//! Interface for configuring the Memory Protection Unit.

use crate::callback::AppId;
use core::cmp;
use core::fmt::{self, Display};

/// User mode access permissions.
#[derive(Copy, Clone)]
pub enum Permissions {
    ReadWriteExecute,
    ReadWriteOnly,
    ReadExecuteOnly,
    ReadOnly,
    ExecuteOnly,
}

/// MPU region.
///
/// This is one contiguous address space protected by the MPU.
#[derive(Copy, Clone)]
pub struct Region {
    /// The memory address where the region starts.
    ///
    /// For maximum compatibility, we use a u8 pointer, however, note that many
    /// memory protection units have very strict alignment requirements for the
    /// memory regions protected by the MPU.
    start_address: *const u8,

    /// The number of bytes of memory in the MPU region.
    size: usize,
}

impl Region {
    /// Create a new MPU region with a given starting point and length in bytes.
    pub fn new(start_address: *const u8, size: usize) -> Region {
        Region {
            start_address: start_address,
            size: size,
        }
    }

    /// Getter: retrieve the address of the start of the MPU region.
    pub fn start_address(&self) -> *const u8 {
        self.start_address
    }

    /// Getter: retrieve the length of the region in bytes.
    pub fn size(&self) -> usize {
        self.size
    }
}

/// Null type for the default type of the `MpuConfig` type in an implementation
/// of the `MPU` trait. We need this to workaround a bug in the Rust compiler.
///
/// Depending how https://github.com/rust-lang/rust/issues/65774 is resolved we
/// may be able to remove this type, but only if a default `Display` is
/// provided for the `()` type.
#[derive(Default)]
pub struct MpuConfigDefault {}

impl Display for MpuConfigDefault {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

/// The generic trait that particular memory protection unit implementations
/// need to implement.
///
/// This trait is a blend of relatively generic MPU functionality that should be
/// common across different MPU implementations, and more specific requirements
/// that Tock needs to support protecting applications. While a less
/// Tock-specific interface may be desirable, due to the sometimes complex
/// alignment rules and other restrictions imposed by MPU hardware, some of the
/// Tock details have to be passed into this interface. That allows the MPU
/// implementation to have more flexibility when satisfying the protection
/// requirements, and also allows the MPU to specify some addresses used by the
/// kernel when deciding where to place certain application memory regions so
/// that the MPU can appropriately provide protection for those memory regions.
pub trait MPU {
    /// MPU-specific state that defines a particular configuration for the MPU.
    /// That is, this should contain all of the required state such that the
    /// implementation can be passed an object of this type and it should be
    /// able to correctly and entirely configure the MPU.
    ///
    /// This state will be held on a per-process basis as a way to cache all of
    /// the process settings. When the kernel switches to a new process it will
    /// use the `MpuConfig` for that process to quickly configure the MPU.
    ///
    /// It is `Default` so we can create empty state when the process is
    /// created, and `Display` so that the `panic!()` output can display the
    /// current state to help with debugging.
    type MpuConfig: Default + Display = MpuConfigDefault;

    /// Clears the MPU.
    ///
    /// This function will clear any access control enforced by the
    /// MPU where possible.
    /// On some hardware it is impossible to reset the MPU after it has
    /// been locked, in this case this function wont change those regions.
    fn clear_mpu(&self) {}

    /// Enables the MPU for userspace apps.
    ///
    /// This function must enable the permission restrictions on the various
    /// regions protected by the MPU.
    fn enable_app_mpu(&self) {}

    /// Disables the MPU for userspace apps.
    ///
    /// This function must disable any access control that was previously setup
    /// for an app if it will interfere with the kernel.
    /// This will be called before the kernel starts to execute as on some
    /// platforms the MPU rules apply to privileged code as well, and therefore
    /// some of the MPU configuration must be disabled for the kernel to effectively
    /// manage processes.
    fn disable_app_mpu(&self) {}

    /// Returns the maximum number of regions supported by the MPU.
    fn number_total_regions(&self) -> usize {
        0
    }

    /// Allocates a new MPU region.
    ///
    /// An implementation must allocate an MPU region at least `min_region_size`
    /// bytes in size within the specified stretch of unallocated memory, and
    /// with the specified user mode permissions, and store it in `config`. The
    /// allocated region may not overlap any of the regions already stored in
    /// `config`.
    ///
    /// # Arguments
    ///
    /// - `unallocated_memory_start`: start of unallocated memory
    /// - `unallocated_memory_size`:  size of unallocated memory
    /// - `min_region_size`:          minimum size of the region
    /// - `permissions`:              permissions for the region
    /// - `config`:                   MPU region configuration
    ///
    /// # Return Value
    ///
    /// Returns the start and size of the allocated MPU region. If it is
    /// infeasible to allocate the MPU region, returns None.
    #[allow(unused_variables)]
    fn allocate_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_region_size: usize,
        permissions: Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<Region> {
        if min_region_size > unallocated_memory_size {
            None
        } else {
            Some(Region::new(unallocated_memory_start, min_region_size))
        }
    }

    /// Chooses the location for a process's memory, and allocates an MPU region
    /// covering the app-owned part.
    ///
    /// An implementation must choose a contiguous block of memory that is at
    /// least `min_memory_size` bytes in size and lies completely within the
    /// specified stretch of unallocated memory.
    ///
    /// It must also allocate an MPU region with the following properties:
    ///
    /// 1. The region covers at least the first `initial_app_memory_size` bytes
    ///    at the beginning of the memory block.
    /// 2. The region does not overlap the last `initial_kernel_memory_size`
    ///    bytes.
    /// 3. The region has the user mode permissions specified by `permissions`.
    ///
    /// The end address of app-owned memory will increase in the future, so the
    /// implementation should choose the location of the process memory block
    /// such that it is possible for the MPU region to grow along with it. The
    /// implementation must store the allocated region in `config`. The
    /// allocated region may not overlap any of the regions already stored in
    /// `config`.
    ///
    /// # Arguments
    ///
    /// - `unallocated_memory_start`:   start of unallocated memory
    /// - `unallocated_memory_size`:    size of unallocated memory
    /// - `min_memory_size`:            minimum total memory to allocate for process
    /// - `initial_app_memory_size`:    initial size of app-owned memory
    /// - `initial_kernel_memory_size`: initial size of kernel-owned memory
    /// - `permissions`:                permissions for the MPU region
    /// - `config`:                     MPU region configuration
    ///
    /// # Return Value
    ///
    /// This function returns the start address and the size of the memory block
    /// chosen for the process. If it is infeasible to find a memory block or
    /// allocate the MPU region, or if the function has already been called,
    /// returns None. If None is returned no changes are made.
    #[allow(unused_variables)]
    fn allocate_app_memory_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_memory_size: usize,
        initial_app_memory_size: usize,
        initial_kernel_memory_size: usize,
        permissions: Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<(*const u8, usize)> {
        let memory_size = cmp::max(
            min_memory_size,
            initial_app_memory_size + initial_kernel_memory_size,
        );
        if memory_size > unallocated_memory_size {
            None
        } else {
            Some((unallocated_memory_start, memory_size))
        }
    }

    /// Updates the MPU region for app-owned memory.
    ///
    /// An implementation must reallocate the MPU region for app-owned memory
    /// stored in `config` to maintain the 3 conditions described in
    /// `allocate_app_memory_region`.
    ///
    /// # Arguments
    ///
    /// - `app_memory_break`:    new address for the end of app-owned memory
    /// - `kernel_memory_break`: new address for the start of kernel-owned memory
    /// - `permissions`:         permissions for the MPU region
    /// - `config`:              MPU region configuration
    ///
    /// # Return Value
    ///
    /// Returns an error if it is infeasible to update the MPU region, or if it
    /// was never created. If an error is returned no changes are made to the
    /// configuration.
    #[allow(unused_variables)]
    fn update_app_memory_region(
        &self,
        app_memory_break: *const u8,
        kernel_memory_break: *const u8,
        permissions: Permissions,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        if (app_memory_break as usize) > (kernel_memory_break as usize) {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Configures the MPU with the provided region configuration.
    ///
    /// An implementation must ensure that all memory locations not covered by
    /// an allocated region are inaccessible in user mode and accessible in
    /// supervisor mode.
    ///
    /// # Arguments
    ///
    /// - `config`: MPU region configuration
    /// - `app_id`: AppId of the process that the MPU is configured for
    #[allow(unused_variables)]
    fn configure_mpu(&self, config: &Self::MpuConfig, app_id: &AppId) {}
}

/// Implement default MPU trait for unit.
impl MPU for () {}

/// The generic trait that particular kernel level memory protection unit
/// implementations need to implement.
///
/// This trait provides generic functionality to extend the MPU trait above
/// to also allow the kernel to protect itself. It is expected that only a
/// limited number of SoCs can support this, which is why it is a seperate
/// implementation.
pub trait KernelMPU {
    /// MPU-specific state that defines a particular configuration for the kernel
    /// MPU.
    /// That is, this should contain all of the required state such that the
    /// implementation can be passed an object of this type and it should be
    /// able to correctly and entirely configure the MPU.
    ///
    /// It is `Default` so we can create empty state when the kernel is
    /// created, and `Display` so that the `panic!()` output can display the
    /// current state to help with debugging.
    type KernelMpuConfig: Default + Display = MpuConfigDefault;

    /// Mark a region of memory that the Tock kernel owns.
    ///
    /// This function will optionally set the MPU to enforce the specified
    /// constraints for all accessess (even from the kernel).
    /// This should be used to mark read/write/execute areas of the Tock
    /// kernel to have the hardware enforce those permissions.
    ///
    /// If the KernelMPU trait is supported a board should use this function
    /// to set permissions for all areas of memory the kernel will use.
    /// Once all regions of memory have been allocated, the board must call
    /// enable_kernel_mpu(). After enable_kernel_mpu() is called no changes
    /// to kernel level code permissions can be made.
    ///
    /// Note that kernel level permissions also apply to apps, although apps
    /// will have more constraints applied on top of the kernel ones as
    /// specified by the `MPU` trait.
    ///
    /// Not all architectures support this, so don't assume this will be
    /// implemented.
    ///
    /// # Arguments
    ///
    /// - `memory_start`:             start of memory region
    /// - `memory_size`:              size of unallocated memory
    /// - `permissions`:              permissions for the region
    /// - `config`:                   MPU region configuration
    ///
    /// # Return Value
    ///
    /// Returns the start and size of the requested memory region. If it is
    /// infeasible to allocate the MPU region, returns None. If None is
    /// returned no changes are made.
    #[allow(unused_variables)]
    fn allocate_kernel_region(
        &self,
        memory_start: *const u8,
        memory_size: usize,
        permissions: Permissions,
        config: &mut Self::KernelMpuConfig,
    ) -> Option<Region>;

    /// Enables the MPU for the kernel.
    ///
    /// This function must enable the permission restrictions on the various
    /// kernel regions specified by `allocate_kernel_region()` protected by
    /// the MPU.
    ///
    /// It is expected that this function is called in `reset_handler()`.
    ///
    /// Once enabled this cannot be disabled. It is expected there won't be any
    /// changes to the kernel regions after this is enabled.
    #[allow(unused_variables)]
    fn enable_kernel_mpu(&self, config: &mut Self::KernelMpuConfig);
}
