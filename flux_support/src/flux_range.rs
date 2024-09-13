#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[flux_rs::refined_by(start: int, end: int)]
pub struct FluxRange {
    #[field(usize[start])]
    pub start: usize,
    #[field(usize[end])]
    pub end: usize,
}
