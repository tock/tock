//! Interface for configuring the Memory Protection Unit.

#[derive(Copy, Clone)]
pub enum Permission {
    //                 Privileged  Unprivileged
    //                 Access      Access
    NoAccess,       // --          --
    PrivilegedOnly, // V           --
    Full,           // V           V
}

#[derive(Copy, Clone)]
pub struct Region {
    start: usize,
    len: usize,
    read: Permission,
    write: Permission,
    execute: Permission,
}

impl Region {
    pub fn new(
        start: usize,
        len: usize,
        read: Permission,
        write: Permission,
        execute: Permission,
    ) -> Region {
        Region {
            start: start,
            len: len,
            read: read,
            write: write,
            execute: execute,
        }
    }

    pub fn empty() -> Region {
        Region {
            start: 0,
            len: 0,
            read: Permission::NoAccess,
            write: Permission::NoAccess,
            execute: Permission::NoAccess,
        }
    }

    pub fn get_start(&self) -> usize {
        self.start
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn get_read_permission(&self) -> Permission {
        self.read
    }

    pub fn get_write_permission(&self) -> Permission {
        self.write
    }

    pub fn get_execute_permission(&self) -> Permission {
        self.execute
    }
}

pub trait MPU {
    /// Enables the MPU.
    fn enable_mpu(&self);

    /// Disables the MPU.
    fn disable_mpu(&self);

    /// Returns the number of supported MPU regions.
    fn num_supported_regions(&self) -> u32;

    /// Allocates memory protection regions.
    ///
    /// # Arguments
    ///
    /// `regions`: array of regions to be allocated. The index of the array
    ///            encodes the priority of the region. In the event of an
    ///            overlap between regions, the implementor must ensure
    ///            that the permissions of the region with higher priority
    ///            take precendence.
    ///
    /// # Return Value
    ///
    /// If it is infeasible to allocate a memory region, returns its index
    /// wrapped in a Result.
    fn allocate_regions(&self, regions: &[Region]) -> Result<(), usize>;
}

/// No-op implementation of MPU trait
impl MPU for () {
    fn enable_mpu(&self) {}

    fn disable_mpu(&self) {}

    fn num_supported_regions(&self) -> u32 {
        8
    }

    fn allocate_regions(&self, _: &[Region]) -> Result<(), usize> {
        Ok(())
    }
}
