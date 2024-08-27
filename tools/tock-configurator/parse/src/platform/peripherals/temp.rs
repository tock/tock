// Copyright OxidOS Automotive 2024.

use super::NoSupport;

pub trait Temperature: crate::Component + std::fmt::Debug + std::fmt::Display {}

impl Temperature for NoSupport {}
