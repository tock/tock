// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Components for screen adapters.

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;

#[macro_export]
macro_rules! screen_adapter_argb8888_to_mono8bitpage_component_static {
    ($S:ty, $SCREEN_WIDTH:expr, $SCREEN_HEIGHT:expr, $PIXEL_STRIDE:expr $(,)?) => {{
        let adapter = kernel::static_buf!(
            capsules_extra::screen::screen_adapters::ScreenARGB8888ToMono8BitPage<'static, $S>
        );
        let frame_buffer =
            kernel::static_buf!([u8; $SCREEN_WIDTH * $SCREEN_HEIGHT * $PIXEL_STRIDE]);

        (adapter, frame_buffer)
    };};
}

pub type ScreenAdapterARGB8888ToMono8BitPageComponentType<S> =
    capsules_extra::screen::screen_adapters::ScreenARGB8888ToMono8BitPage<'static, S>;

pub struct ScreenAdapterARGB8888ToMono8BitPageComponent<
    S: hil::screen::Screen<'static> + 'static,
    const FRAME_BUFFER_LEN: usize,
> {
    screen: &'static S,
}

impl<S: hil::screen::Screen<'static>, const FRAME_BUFFER_LEN: usize>
    ScreenAdapterARGB8888ToMono8BitPageComponent<S, FRAME_BUFFER_LEN>
{
    pub fn new(screen: &'static S) -> Self {
        Self { screen }
    }
}

impl<S: hil::screen::Screen<'static> + 'static, const FRAME_BUFFER_LEN: usize> Component
    for ScreenAdapterARGB8888ToMono8BitPageComponent<S, FRAME_BUFFER_LEN>
{
    type StaticInput = (
        &'static mut MaybeUninit<
            capsules_extra::screen::screen_adapters::ScreenARGB8888ToMono8BitPage<'static, S>,
        >,
        &'static mut MaybeUninit<[u8; FRAME_BUFFER_LEN]>,
    );
    type Output =
        &'static capsules_extra::screen::screen_adapters::ScreenARGB8888ToMono8BitPage<'static, S>;

    fn finalize(self, static_input: Self::StaticInput) -> Self::Output {
        let frame_buffer = static_input.1.write([0; FRAME_BUFFER_LEN]);
        let adapter = static_input.0.write(
            capsules_extra::screen::screen_adapters::ScreenARGB8888ToMono8BitPage::new(
                self.screen,
                frame_buffer,
            ),
        );

        kernel::hil::screen::Screen::set_client(self.screen, adapter);

        adapter
    }
}
