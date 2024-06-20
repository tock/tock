// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! This file stores the variables that change every new load process
//! iteration.
//!
//! The ProcessLoadMetadata struct is wrapped as an OptionalCell in
//! the dynamic_process_loader.rs file to track the variables.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PaddingRequirement {
    None,
    PrePad,
    PostPad,
    PreAndPostPad,
}

#[derive(Clone, Copy, Debug)]
pub struct ProcessLoadMetadata {
    pub new_app_start_addr: usize,
    pub new_app_length: usize,
    pub previous_app_end_addr: usize,
    pub next_app_start_addr: usize,
    pub padding_requirement: PaddingRequirement,
}

// Implement the Default trait for the Person struct
impl Default for ProcessLoadMetadata {
    fn default() -> Self {
        ProcessLoadMetadata {
            new_app_start_addr: 0,
            new_app_length: 0,
            previous_app_end_addr: 0,
            next_app_start_addr: 0,
            padding_requirement: PaddingRequirement::None,
        }
    }
}

impl ProcessLoadMetadata {
    pub fn get_new_app_addr(&self) -> usize {
        self.new_app_start_addr
    }

    pub fn get_new_app_length(&self) -> usize {
        self.new_app_length
    }

    pub fn get_previous_app_end_addr(&self) -> usize {
        self.previous_app_end_addr
    }

    pub fn get_next_app_start_addr(&self) -> usize {
        self.next_app_start_addr
    }

    pub fn get_padding_requirement(&self) -> PaddingRequirement {
        self.padding_requirement
    }
}
