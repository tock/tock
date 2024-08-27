// Copyright OxidOS Automotive 2024.

use crate::Component;

use super::NoSupport;

pub trait Flash: Component + std::fmt::Display {}
impl Flash for NoSupport {}
