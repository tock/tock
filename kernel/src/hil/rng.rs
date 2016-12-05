//! Interfaces for accessing a random number generator

#[derive(PartialEq, Eq)]
pub enum Continue {
    More,
    Done,
}

pub trait RNG {
    fn get(&self);
}

pub trait Client {
    fn randomness_available(&self, randomness: &mut Iterator<Item = u32>) -> Continue;
}
