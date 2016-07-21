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
#ifndef H_HAL_OS_TICK_
#define H_HAL_OS_TICK_

#ifdef __cplusplus
extern "C" {
#endif

#include <os/os_time.h>

/*
 * Set up the periodic timer to interrupt at a frequency of 'os_ticks_per_sec'.
 * 'prio' is the cpu-specific priority of the periodic timer interrupt.
 */
void os_tick_init(uint32_t os_ticks_per_sec, int prio);

/*
 * Halt CPU for up to 'n' ticks.
 */
void os_tick_idle(os_time_t n);


#ifdef __cplusplus
}
#endif

#endif /* H_HAL_OS_TICK_ */
