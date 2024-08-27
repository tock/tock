// Copyright OxidOS Automotive 2024.

use super::NoSupport;

pub trait Rng: crate::Component + std::fmt::Debug + std::fmt::Display {}

impl Rng for NoSupport {}
