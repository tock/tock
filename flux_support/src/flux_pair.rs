#[derive(Copy, Clone)]
#[flux_rs::refined_by(fst: T, snd: E)]
pub struct Pair<T, E> {
    #[field(T[fst])]
    pub fst: T,
    #[field(E[snd])]
    pub snd: E,
}
