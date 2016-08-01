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

#ifndef _OS_ARCH_ARM_H 
#define _OS_ARCH_ARM_H 

#include <stdint.h>
#include <mcu/cortex_m0.h>

struct os_task;

/* Run in priviliged or unprivileged Thread mode */
#define OS_RUN_PRIV         (0)
#define OS_RUN_UNPRIV       (1)

/* CPU status register */
typedef uint32_t os_sr_t;
/* Stack type, aligned to a 32-bit word. */
#define OS_STACK_PATTERN    (0xdeadbeef)

typedef uint32_t os_stack_t;
#define OS_ALIGNMENT        (4)
#define OS_STACK_ALIGNMENT  (8)

/*
 * Stack sizes for common OS tasks
 */
#define OS_SANITY_STACK_SIZE (64)
#define OS_IDLE_STACK_SIZE (64)

#define OS_STACK_ALIGN(__nmemb) \
    (OS_ALIGN((__nmemb), OS_STACK_ALIGNMENT))

/* Enter a critical section, save processor state, and block interrupts */
#define OS_ENTER_CRITICAL(__os_sr) (__os_sr = os_arch_save_sr()) 
/* Exit a critical section, restore processor state and unblock interrupts */
#define OS_EXIT_CRITICAL(__os_sr) (os_arch_restore_sr(__os_sr))
#define OS_ASSERT_CRITICAL() (assert(os_arch_in_critical()))

os_stack_t *os_arch_task_stack_init(struct os_task *, os_stack_t *, int);
void timer_handler(void);
void os_arch_ctx_sw(struct os_task *);
os_sr_t os_arch_save_sr(void);
void os_arch_restore_sr(os_sr_t);
int os_arch_in_critical(void);
void os_arch_init(void);
uint32_t os_arch_start(void);
os_error_t os_arch_os_init(void);
os_error_t os_arch_os_start(void);
void os_set_env(void);
void os_arch_init_task_stack(os_stack_t *sf);
void os_default_irq_asm(void);

/* External function prototypes supplied by BSP */
void os_bsp_systick_init(uint32_t os_ticks_per_sec, int prio);
void os_bsp_init(void);
void os_bsp_ctx_sw(void);

#endif /* _OS_ARCH_X86_H */ 
