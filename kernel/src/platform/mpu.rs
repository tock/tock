//! Interface for configuring the Memory Protection Unit.

use core::cmp;

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
    type MpuConfig: Default = ();

    /// Enables the MPU.
    ///
    /// This function must enable the permission restrictions on the various
    /// regions protected by the MPU.
    fn enable_mpu(&self) {}

    /// Disables the MPU.
    ///
    /// This function must completely disable any access control enforced by the
    /// MPU. This will be called before the kernel starts to execute as on some
    /// platforms the MPU rules apply to privileged code as well, and therefore
    /// the MPU must be completely disabled for the kernel to effectively manage
    /// processes.
    fn disable_mpu(&self) {}

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
    /// returns None.
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
    /// was never created.
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
    /// - `config: MPU region configuration
    #[allow(unused_variables)]
    fn configure_mpu(&self, config: &Self::MpuConfig) {}
}

/// Implement default MPU trait for unit.
impl MPU for () {}
