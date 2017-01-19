//! returncode.rs -- Standard return type for invoking operations, returning
//! success or an error code.
//!
//!  Author: Philip Levis <pal@cs.stanford.edu>
//!  Date: Dec 22, 2016

pub enum ReturnCode {
    SUCCESS = 0,
    FAIL = -1, //.......... Generic failure condition
    EBUSY = -2, //......... Underlying system is busy; retry
    EALREADY = -3, //...... The state requested is already set
    EOFF = -4, //.......... The component is powered down
    ERESERVE = -5, //...... Reservation required before use
    EINVAL = -6, //........ An invalid parameter was passed
    ESIZE = -7, //......... Parameter passed was too large
    ECANCEL = -8, //....... Operation cancelled by a call
    ENOMEM = -9, //........ Memory required not available
    ENOSUPPORT = -10, //... Operation or command is unsupported
    ENODEVICE = -11, //.... Device does not exist
}
