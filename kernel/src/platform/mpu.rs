
#[derive(Debug)]
pub enum AccessPermission {
    //                                 Privileged  Unprivileged
    //                                 Access      Access
    NoAccess = 0b000, //.............. --          --
    PrivilegedOnly = 0b001, //........ RW          --
    UnprivilegedReadOnly = 0b010, //.. RW          R-
    ReadWrite = 0b011, //............. RW          RW
    Reserved = 0b100, //.............. undef       undef
    PrivilegedOnlyReadOnly = 0b101, // R-          --
    ReadOnly = 0b110, //.............. R-          R-
    ReadOnlyAlais = 0b111, //......... R-          R-
}

#[derive(Debug)]
pub enum ExecutePermission {
    ExecutionPermitted = 0b0,
    ExecutionNotPermitted = 0b1,
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
    fn set_mpu(&self,
               region_num: u32,
               start_addr: u32,
               len: u32,
               execute: ExecutePermission,
               ap: AccessPermission);
}

/// Noop implementation of MPU trait
impl MPU for () {
    fn enable_mpu(&self) {}

    fn set_mpu(&self, _: u32, _: u32, _: u32, _: ExecutePermission, _: AccessPermission) {}
}
