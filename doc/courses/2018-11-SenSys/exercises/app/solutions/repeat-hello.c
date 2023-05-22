// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

#include <stdbool.h>
#include <stdio.h>

#include <timer.h>

int main (void) {
  while (true) {
    printf("Hello, World!\n");
    delay_ms(500);
  }
}

