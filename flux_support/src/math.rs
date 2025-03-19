use crate::FluxPtr;

#[flux_rs::sig(fn(lhs: usize, rhs: usize) -> usize[if lhs >= rhs { lhs } else { rhs }])]
pub fn max_usize(lhs: usize, rhs: usize) -> usize {
    if lhs >= rhs {
        lhs
    } else {
        rhs
    }
}

#[flux_rs::sig(fn(lhs: usize, rhs: usize) -> usize[if lhs <= rhs { lhs } else { rhs }])]
pub fn min_usize(lhs: usize, rhs: usize) -> usize {
    if lhs <= rhs {
        lhs
    } else {
        rhs
    }
}

#[flux_rs::sig(fn(lhs: u32, rhs: u32) -> u32 {r: (lhs >= rhs => r == lhs) && (rhs > lhs => r == rhs)})]
pub fn max_u32(lhs: u32, rhs: u32) -> u32 {
    if lhs >= rhs {
        lhs
    } else {
        rhs
    }
}

#[flux_rs::sig(fn(self: FluxPtr[@lhs], other: FluxPtr[@rhs]) -> FluxPtr {r: (lhs >= rhs => r == lhs) && (rhs > lhs => r == rhs)})]
pub fn max_ptr(lhs: FluxPtr, rhs: FluxPtr) -> FluxPtr {
    if lhs >= rhs {
        lhs
    } else {
        rhs
    }
}
