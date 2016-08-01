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

#include "hal/hal_cputime.h"

int cputime_init(uint32_t clock_freq);
uint64_t cputime_get64(void);
uint32_t cputime_get32(void);
uint32_t cputime_nsecs_to_ticks(uint32_t nsecs);
uint32_t cputime_ticks_to_nsecs(uint32_t ticks);
uint32_t cputime_usecs_to_ticks(uint32_t usecs) {return usecs;}
uint32_t cputime_ticks_to_usecs(uint32_t ticks);
void cputime_delay_ticks(uint32_t ticks);
void cputime_delay_nsecs(uint32_t nsecs);
void cputime_delay_usecs(uint32_t usecs);
void cputime_timer_init(struct cpu_timer *timer, cputimer_func fp, void *arg);
void cputime_timer_start(struct cpu_timer *timer, uint32_t cputime);
void cputime_timer_relative(struct cpu_timer *timer, uint32_t usecs);
void cputime_timer_stop(struct cpu_timer *timer);

