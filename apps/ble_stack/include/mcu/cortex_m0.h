/**
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 * 
 *  http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

#ifndef __MCU_CORTEX_M0_H__
#define __MCU_CORTEX_M0_H__

#include "mcu/nrf51.h"

/*
 * The nRF51 microcontroller uses RTC0 for periodic interrupts and it is
 * clocked at 32768Hz. The tick frequency is chosen such that it divides
 * cleanly into 32768 to avoid a systemic bias in the actual tick frequency.
 */
#define OS_TICKS_PER_SEC    (1024)

#endif /* __MCU_CORTEX_M0_H__ */
