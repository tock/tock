# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2025.

/* Minimal startup code for ARM Cortex-M4 (nRF52840) bootloader*/

    .syntax unified
    .cpu cortex-m4
    .thumb

/* Vector table - at 0x00000000 */
    .section .vector_table, "a", %progbits
    .align 2
    .globl vector_table
    .type vector_table, %object

vector_table:
    /* Cortex-M4 system exceptions */
    .word   _stack_start                /* Initial stack pointer */
    .word   Reset_Handler               /* Reset */
    .word   NMI_Handler                 /* NMI */
    .word   HardFault_Handler           /* Hard fault */
    .word   MemManage_Handler           /* Memory management fault */
    .word   BusFault_Handler            /* Bus fault */
    .word   UsageFault_Handler          /* Usage fault */
    .word   0                           /* Reserved */
    .word   0                           /* Reserved */
    .word   0                           /* Reserved */
    .word   0                           /* Reserved */
    .word   SVC_Handler                 /* SVCall */
    .word   DebugMon_Handler            /* Debug monitor */
    .word   0                           /* Reserved */
    .word   PendSV_Handler              /* PendSV */
    .word   SysTick_Handler             /* SysTick */

    .size vector_table, . - vector_table

/* Reset handler */
    .section .text.Reset_Handler
    .globl Reset_Handler
    .type Reset_Handler, %function
    .thumb_func

Reset_Handler:
call_main:
    /* Call main function */
    bl      main

    /* If main returns, loop forever */
infinite_loop:
    b       infinite_loop

    .size Reset_Handler, . - Reset_Handler