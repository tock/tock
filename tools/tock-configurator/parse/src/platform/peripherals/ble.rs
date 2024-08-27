// Copyright OxidOS Automotive 2024.

use super::NoSupport;

pub trait BleAdvertisement: crate::Component + std::fmt::Debug + std::fmt::Display {}

impl BleAdvertisement for NoSupport {}
