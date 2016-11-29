//! result.rs -- Standard return type for invoking operations, returning
//! success or an error code.
//!
//!  Author: Philip Levis <pal@cs.stanford.edu>
//!  Date: Nov 21, 2016

enum Result {
    SUCCESS,
    FAIL, // Generic failure condition
    EBUSY, // Underlying system is busy; retry
    EALREADY, // The state requested is already set
    EOFF, // The component is powered down
    ERESERVE, // Reservation required before use
    EINVAL, // An invalid parameter was passed
    ESIZE, // Parameter passed was too large
    ECANCEL, // Operation cancelled by a call
    ENOMEM, // Memory required not available
}
