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

#ifndef _OS_ARCH_SIM_H
#define _OS_ARCH_SIM_H

#include <mcu/mcu_sim.h>

struct os_task;

/* CPU status register */
typedef unsigned int os_sr_t;
/* Stack type, aligned to a 32-bit word. */
#define OS_STACK_PATTERN (0xdeadbeef)

typedef unsigned int os_stack_t;
#define OS_ALIGNMENT (4)
#define OS_STACK_ALIGNMENT (16)

/*
 * Stack sizes for common OS tasks
 */
#define OS_SANITY_STACK_SIZE (1024)
#define OS_IDLE_STACK_SIZE (1024)

/*
 * The 'sim' architecture-specific code does not have as much control on
 * stack usage as the real embedded architectures.
 *
 * For e.g. the space occupied by the signal handler frame on the task
 * stack is entirely dependent on the host OS.
 *
 * Deal with this by scaling the stack size by a factor of 16. The scaling
 * factor can be arbitrarily large because the stacks are allocated from
 * BSS and thus don't add to either the executable size or resident
 * memory.
 */
#define OS_STACK_ALIGN(__nmemb) \
    (OS_ALIGN(((__nmemb) * 16), OS_STACK_ALIGNMENT))

/* Enter a critical section, save processor state, and block interrupts */
#define OS_ENTER_CRITICAL(__os_sr) (__os_sr = os_arch_save_sr())
/* Exit a critical section, restore processor state and unblock interrupts */
#define OS_EXIT_CRITICAL(__os_sr) (os_arch_restore_sr(__os_sr))
#define OS_ASSERT_CRITICAL() (assert(os_arch_in_critical()))

void _Die(char *file, int line);

os_stack_t *os_arch_task_stack_init(struct os_task *, os_stack_t *, int);
void os_arch_ctx_sw(struct os_task *);
os_sr_t os_arch_save_sr(void);
void os_arch_restore_sr(os_sr_t sr);
int os_arch_in_critical(void);
os_error_t os_arch_os_init(void);
void os_arch_os_stop(void);
os_error_t os_arch_os_start(void);

void os_bsp_init(void);

#endif /* _OS_ARCH_SIM_H */
