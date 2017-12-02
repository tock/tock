// Author: Ana Helena Sánchez, Björn Haase (second implementation).
//
// public domain
//
//// This assmeble file was taken from
// https://munacl.cryptojedi.org/curve25519-cortexm0.shtml
// Credit to  Björn Haase and Ana Helena Sánchez, as well
// as M. Hutter and P. Schwabe
// This assemble file exports the function multiply256x256_asm
// with 2 inputs, each of type [u32;8]. The program will square
// the first input in modulo 2^255 − 19 and puts the result
// in the second input.
//The commented out code was the multiplication that was 
//replaces by the faster UMULL (supported by the arm cortex m4)


 .align	2
	.global	square256_asm
	.type	square256_asm, %function
square256_asm:
// ######################
// ASM Square 256 refined karatsuba:
// ######################
 // sqr 256 Refined Karatsuba
 // pInput in r1
 // pResult in r0
 // adheres to arm eabi calling convention. 
    push {r1,r4,r5,r6,r7,r14}
    .syntax unified
    mov r3,r8
    .syntax divided
    .syntax unified
    mov r4,r9
    .syntax divided
    .syntax unified
    mov r5,r10
    .syntax divided
    .syntax unified
    mov r6,r11
    .syntax divided
    .syntax unified
    mov r7,r12
    .syntax divided
    push {r3,r4,r5,r6,r7}
    .syntax unified
    mov r14,r0
    .syntax divided
    ldm r1!,{r4,r5,r6,r7}
 // sqr 128 Refined Karatsuba
 // Input in r4 ... r7 
 // Result in r0 ... r7 
 // clobbers all registers except for r14 
    .syntax unified
    mov r0,r4
    .syntax divided
    .syntax unified
    mov r1,r5
    .syntax divided
    sub r0,r6
    sbc r1,r7
    sbc r2,r2
    eor r0,r2
    eor r1,r2
    sub r0,r2
    sbc r1,r2
    .syntax unified
    mov r8,r0
    .syntax divided
    .syntax unified
    mov r9,r1
    .syntax divided
    .syntax unified
    mov r10,r6
    .syntax divided
 // START: sqr 64 Refined Karatsuba
 // Input operands in r4,r5
 // Result in r0,r1,r2,r3
 // Clobbers: r4-r6
 // START: sqr 32
 // Input operand in r4 
 // Result in r0 ,r1 
 // Clobbers: r2, r3 


/*
    uxth r0,r4



    lsr r1,r4,#16
    .syntax unified
    mov r2,r0
    .syntax divided
    mul r2,r1
    mul r0,r0
    mul r1,r1
    lsr r3,r2,#15
    lsl r2,r2,#17
    add r0,r2
    adc r1,r3
*/

    UMULL r0,r1, r4,r4
 // End: sqr 32
 // Result in r0 ,r1 
    sub r4,r5
    sbc r6,r6
    eor r4,r6
    sub r4,r6
 // START: sqr 32
 // Input operand in r5 
 // Result in r2 ,r3 
 // Clobbers: r5, r6 

/*
    uxth r2,r5
    lsr r3,r5,#16
    .syntax unified
    mov r5,r2
    .syntax divided
    mul r5,r3
    mul r2,r2
    mul r3,r3
    lsr r6,r5,#15
    lsl r5,r5,#17
    add r2,r5
    adc r3,r6
*/
   UMULL r2,r3, r5,r5 

 // End: sqr 32
 // Result in r2 ,r3 
    mov r6,#0
    add r2,r1
    adc r3,r6
 // START: sqr 32
 // Input operand in r4 
 // Result in r4 ,r5 
 // Clobbers: r1, r6 

/*
    lsr r5,r4,#16
    uxth r4,r4
    .syntax unified
    mov r1,r4
    .syntax divided
    mul r1,r5
    mul r4,r4
    mul r5,r5
    lsr r6,r1,#15
    lsl r1,r1,#17
    add r4,r1
    adc r5,r6
*/
    UMULL r4,r5, r4,r4

 // End: sqr 32
 // Result in r4 ,r5 
    .syntax unified
    mov r1,r2
    .syntax divided
    sub r1,r4
    sbc r2,r5
    .syntax unified
    mov r5,r3
    .syntax divided
    mov r6,#0
    sbc r3,r6
    add r1,r0
    adc r2,r5
    adc r3,r6
 // END: sqr 64 Refined Karatsuba
 // Result in r0,r1,r2,r3
 // Leaves r6 zero.
    .syntax unified
    mov r6,r10
    .syntax divided
    .syntax unified
    mov r10,r0
    .syntax divided
    .syntax unified
    mov r11,r1
    .syntax divided
    .syntax unified
    mov r12,r2
    .syntax divided
    .syntax unified
    mov r1,r3
    .syntax divided
 // START: sqr 64 Refined Karatsuba
 // Input operands in r6,r7
 // Result in r2,r3,r4,r5 
 // Clobbers: r0,r7,r6
 // START: sqr 32
 // Input operand in r6 
 // Result in r2 ,r3 
 // Clobbers: r4, r5 

/*
    uxth r2,r6
    lsr r3,r6,#16
    .syntax unified
    mov r4,r2
    .syntax divided
    mul r4,r3
    mul r2,r2
    mul r3,r3
    lsr r5,r4,#15
    lsl r4,r4,#17
    add r2,r4
    adc r3,r5
*/

    UMULL r2,r3, r6,r6

 // End: sqr 32
 // Result in r2 ,r3 
    sub r6,r7
    sbc r4,r4
    eor r6,r4
    sub r6,r4
 // START: sqr 32
 // Input operand in r7 
 // Result in r4 ,r5 
 // Clobbers: r0, r7 
/*
    uxth r4,r7
    lsr r5,r7,#16
    .syntax unified
    mov r0,r4
    .syntax divided
    mul r0,r5
    mul r4,r4
    mul r5,r5
    lsr r7,r0,#15
    lsl r0,r0,#17
    add r4,r0
    adc r5,r7
*/
    UMULL r4,r5, r7,r7
 // End: sqr 32
 // Result in r4 ,r5 
    mov r7,#0
    add r4,r3
    adc r5,r7
 // START: sqr 32
 // Input operand in r6 
 // Result in r7 ,r0 
 // Clobbers: r6, r3 

/*
    uxth r7,r6
    lsr r0,r6,#16
    .syntax unified
    mov r6,r7
    .syntax divided
    mul r6,r0
    mul r7,r7
    mul r0,r0
    lsr r3,r6,#15
    lsl r6,r6,#17
    add r7,r6
    adc r0,r3
*/

    UMULL r7,r0, r6,r6
 // End: sqr 32
 // Result in r7 ,r0 
    .syntax unified
    mov r3,r4
    .syntax divided
    sub r3,r7
    sbc r4,r0
    .syntax unified
    mov r0,r5
    .syntax divided
    mov r6,#0
    sbc r5,r6
    add r3,r2
    adc r4,r0
    adc r5,r6
 // END: sqr 64 Refined Karatsuba
 // Result in r2,r3,r4,r5
 // Leaves r6 zero.
    .syntax unified
    mov r0,r12
    .syntax divided
    add r2,r0
    adc r3,r1
    adc r4,r6
    adc r5,r6
    .syntax unified
    mov r12,r2
    .syntax divided
    .syntax unified
    mov r2,r8
    .syntax divided
    .syntax unified
    mov r8,r3
    .syntax divided
    .syntax unified
    mov r3,r9
    .syntax divided
    .syntax unified
    mov r9,r4
    .syntax divided
 // START: sqr 64 Refined Karatsuba
 // Input operands in r2,r3
 // Result in r6,r7,r0,r1 
 // Clobbers: r2,r3,r4
 // START: sqr 32
 // Input operand in r2 
 // Result in r6 ,r7 
 // Clobbers: r0, r1 
/*
    uxth r6,r2
    lsr r7,r2,#16
    .syntax unified
    mov r0,r6
    .syntax divided
    mul r0,r7
    mul r6,r6
    mul r7,r7
    lsr r1,r0,#15
    lsl r0,r0,#17
    add r6,r0
    adc r7,r1
*/

    UMULL r6,r7, r2,r2
 // End: sqr 32
 // Result in r6 ,r7 
    sub r2,r3
    sbc r4,r4
    eor r2,r4
    sub r2,r4
 // START: sqr 32
 // Input operand in r3 
 // Result in r0 ,r1 
 // Clobbers: r3, r4 
/*
    uxth r0,r3
    lsr r1,r3,#16
    .syntax unified
    mov r3,r0
    .syntax divided
    mul r3,r1
    mul r0,r0
    mul r1,r1
    lsr r4,r3,#15
    lsl r3,r3,#17
    add r0,r3
    adc r1,r4
*/
    UMULL r0,r1, r3,r3

 // End: sqr 32
 // Result in r0 ,r1 
    mov r4,#0
    add r0,r7
    adc r1,r4
 // START: sqr 32
 // Input operand in r2 
 // Result in r3 ,r4 
 // Clobbers: r2, r7 

/*
    uxth r3,r2
    lsr r4,r2,#16
    .syntax unified
    mov r2,r3
    .syntax divided
    mul r2,r4
    mul r3,r3
    mul r4,r4
    lsr r7,r2,#15
    lsl r2,r2,#17
    add r3,r2
    adc r4,r7
*/

    UMULL r3,r4, r2,r2
 // End: sqr 32
 // Result in r3 ,r4 
    .syntax unified
    mov r7,r0
    .syntax divided
    sub r7,r3
    sbc r0,r4
    .syntax unified
    mov r2,r1
    .syntax divided
    mov r4,#0
    sbc r1,r4
    add r7,r6
    adc r0,r2
    adc r1,r4
 // END: sqr 64 Refined Karatsuba
 // Result in r6,r7,r0,r1
 // Returns r4 as zero.
    .syntax unified
    mov r2,r12
    .syntax divided
    .syntax unified
    mov r3,r8
    .syntax divided
    .syntax unified
    mov r4,r9
    .syntax divided
    sub r2,r6
    sbc r3,r7
    .syntax unified
    mov r6,r4
    .syntax divided
    .syntax unified
    mov r7,r5
    .syntax divided
    sbc r4,r0
    sbc r5,r1
    mov r0,#0
    sbc r6,r0
    sbc r7,r0
    .syntax unified
    mov r0,r10
    .syntax divided
    add r2,r0
    .syntax unified
    mov r1,r11
    .syntax divided
    adc r3,r1
    .syntax unified
    mov r0,r12
    .syntax divided
    adc r4,r0
    .syntax unified
    mov r0,r8
    .syntax divided
    adc r5,r0
    mov r0,#0
    adc r6,r0
    adc r7,r0
    .syntax unified
    mov r0,r10
    .syntax divided
 // END: sqr 128 Refined Karatsuba
 // Result in r0 ... r7 
    push {r4,r5,r6,r7}
    .syntax unified
    mov r4,r14
    .syntax divided
    stm r4!,{r0,r1,r2,r3}
    ldr r4,[SP,#36]
    add r4,#16
    ldm r4,{r4,r5,r6,r7}
 // sqr 128 Refined Karatsuba
 // Input in r4 ... r7 
 // Result in r0 ... r7 
 // clobbers all registers except for r14 
    .syntax unified
    mov r0,r4
    .syntax divided
    .syntax unified
    mov r1,r5
    .syntax divided
    sub r0,r6
    sbc r1,r7
    sbc r2,r2
    eor r0,r2
    eor r1,r2
    sub r0,r2
    sbc r1,r2
    .syntax unified
    mov r8,r0
    .syntax divided
    .syntax unified
    mov r9,r1
    .syntax divided
    .syntax unified
    mov r10,r6
    .syntax divided
 // START: sqr 64 Refined Karatsuba
 // Input operands in r4,r5
 // Result in r0,r1,r2,r3
 // Clobbers: r4-r6
 // START: sqr 32
 // Input operand in r4 
 // Result in r0 ,r1 
 // Clobbers: r2, r3 
/*
    uxth r0,r4
    lsr r1,r4,#16
    .syntax unified
    mov r2,r0
    .syntax divided
    mul r2,r1
    mul r0,r0
    mul r1,r1
    lsr r3,r2,#15
    lsl r2,r2,#17
    add r0,r2
    adc r1,r3
*/
    UMULL r0,r1, r4,r4

 // End: sqr 32
 // Result in r0 ,r1 
    sub r4,r5
    sbc r6,r6
    eor r4,r6
    sub r4,r6
 // START: sqr 32
 // Input operand in r5 
 // Result in r2 ,r3 
 // Clobbers: r5, r6 
/*
    uxth r2,r5
    lsr r3,r5,#16
    .syntax unified
    mov r5,r2
    .syntax divided
    mul r5,r3
    mul r2,r2
    mul r3,r3
    lsr r6,r5,#15
    lsl r5,r5,#17
    add r2,r5
    adc r3,r6
*/
    UMULL r2,r3, r5,r5

 // End: sqr 32
 // Result in r2 ,r3 
    mov r6,#0
    add r2,r1
    adc r3,r6
 // START: sqr 32
 // Input operand in r4 
 // Result in r4 ,r5 
 // Clobbers: r1, r6 
/*
    lsr r5,r4,#16
    uxth r4,r4
    .syntax unified
    mov r1,r4
    .syntax divided
    mul r1,r5
    mul r4,r4
    mul r5,r5
    lsr r6,r1,#15
    lsl r1,r1,#17
    add r4,r1
    adc r5,r6
*/
    UMULL r4,r5, r4,r4
 // End: sqr 32
 // Result in r4 ,r5 
    .syntax unified
    mov r1,r2
    .syntax divided
    sub r1,r4
    sbc r2,r5
    .syntax unified
    mov r5,r3
    .syntax divided
    mov r6,#0
    sbc r3,r6
    add r1,r0
    adc r2,r5
    adc r3,r6
 // END: sqr 64 Refined Karatsuba
 // Result in r0,r1,r2,r3
 // Leaves r6 zero.
    .syntax unified
    mov r6,r10
    .syntax divided
    .syntax unified
    mov r10,r0
    .syntax divided
    .syntax unified
    mov r11,r1
    .syntax divided
    .syntax unified
    mov r12,r2
    .syntax divided
    .syntax unified
    mov r1,r3
    .syntax divided
 // START: sqr 64 Refined Karatsuba
 // Input operands in r6,r7
 // Result in r2,r3,r4,r5 
 // Clobbers: r0,r7,r6
 // START: sqr 32
 // Input operand in r6 
 // Result in r2 ,r3 
 // Clobbers: r4, r5 

/*
    uxth r2,r6
    lsr r3,r6,#16
    .syntax unified
    mov r4,r2
    .syntax divided
    mul r4,r3
    mul r2,r2
    mul r3,r3
    lsr r5,r4,#15
    lsl r4,r4,#17
    add r2,r4
    adc r3,r5
*/

    UMULL r2,r3, r6,r6
 // End: sqr 32
 // Result in r2 ,r3 
    sub r6,r7
    sbc r4,r4
    eor r6,r4
    sub r6,r4
 // START: sqr 32
 // Input operand in r7 
 // Result in r4 ,r5 
 // Clobbers: r0, r7 

/*
    uxth r4,r7
    lsr r5,r7,#16
    .syntax unified
    mov r0,r4
    .syntax divided
    mul r0,r5
    mul r4,r4
    mul r5,r5
    lsr r7,r0,#15
    lsl r0,r0,#17
    add r4,r0
    adc r5,r7
*/
    UMULL r4,r5, r7,r7
 // End: sqr 32
 // Result in r4 ,r5 
    mov r7,#0
    add r4,r3
    adc r5,r7
 // START: sqr 32
 // Input operand in r6 
 // Result in r7 ,r0 
 // Clobbers: r6, r3 
/*
    uxth r7,r6
    lsr r0,r6,#16
    .syntax unified
    mov r6,r7
    .syntax divided
    mul r6,r0
    mul r7,r7
    mul r0,r0
    lsr r3,r6,#15
    lsl r6,r6,#17
    add r7,r6
    adc r0,r3
*/
    UMULL r7,r0, r6,r6
 // End: sqr 32
 // Result in r7 ,r0 
    .syntax unified
    mov r3,r4
    .syntax divided
    sub r3,r7
    sbc r4,r0
    .syntax unified
    mov r0,r5
    .syntax divided
    mov r6,#0
    sbc r5,r6
    add r3,r2
    adc r4,r0
    adc r5,r6
 // END: sqr 64 Refined Karatsuba
 // Result in r2,r3,r4,r5
 // Leaves r6 zero.
    .syntax unified
    mov r0,r12
    .syntax divided
    add r2,r0
    adc r3,r1
    adc r4,r6
    adc r5,r6
    .syntax unified
    mov r12,r2
    .syntax divided
    .syntax unified
    mov r2,r8
    .syntax divided
    .syntax unified
    mov r8,r3
    .syntax divided
    .syntax unified
    mov r3,r9
    .syntax divided
    .syntax unified
    mov r9,r4
    .syntax divided
 // START: sqr 64 Refined Karatsuba
 // Input operands in r2,r3
 // Result in r6,r7,r0,r1 
 // Clobbers: r2,r3,r4
 // START: sqr 32
 // Input operand in r2 
 // Result in r6 ,r7 
 // Clobbers: r0, r1 
/*
    uxth r6,r2
    lsr r7,r2,#16
    .syntax unified
    mov r0,r6
    .syntax divided
    mul r0,r7
    mul r6,r6
    mul r7,r7
    lsr r1,r0,#15
    lsl r0,r0,#17
    add r6,r0
    adc r7,r1
*/
    UMULL r6,r7, r2,r2
 // End: sqr 32
 // Result in r6 ,r7 
    sub r2,r3
    sbc r4,r4
    eor r2,r4
    sub r2,r4
 // START: sqr 32
 // Input operand in r3 
 // Result in r0 ,r1 
 // Clobbers: r3, r4 
/*
    uxth r0,r3
    lsr r1,r3,#16
    .syntax unified
    mov r3,r0
    .syntax divided
    mul r3,r1
    mul r0,r0
    mul r1,r1
    lsr r4,r3,#15
    lsl r3,r3,#17
    add r0,r3
    adc r1,r4
*/
    UMULL r0,r1, r3,r3
 // End: sqr 32
 // Result in r0 ,r1 
    mov r4,#0
    add r0,r7
    adc r1,r4
 // START: sqr 32
 // Input operand in r2 
 // Result in r3 ,r4 
 // Clobbers: r2, r7 
/*
    uxth r3,r2
    lsr r4,r2,#16
    .syntax unified
    mov r2,r3
    .syntax divided
    mul r2,r4
    mul r3,r3
    mul r4,r4
    lsr r7,r2,#15
    lsl r2,r2,#17
    add r3,r2
    adc r4,r7
*/

    UMULL r3,r4, r2,r2
 // End: sqr 32
 // Result in r3 ,r4 
    .syntax unified
    mov r7,r0
    .syntax divided
    sub r7,r3
    sbc r0,r4
    .syntax unified
    mov r2,r1
    .syntax divided
    mov r4,#0
    sbc r1,r4
    add r7,r6
    adc r0,r2
    adc r1,r4
 // END: sqr 64 Refined Karatsuba
 // Result in r6,r7,r0,r1
 // Returns r4 as zero.
    .syntax unified
    mov r2,r12
    .syntax divided
    .syntax unified
    mov r3,r8
    .syntax divided
    .syntax unified
    mov r4,r9
    .syntax divided
    sub r2,r6
    sbc r3,r7
    .syntax unified
    mov r6,r4
    .syntax divided
    .syntax unified
    mov r7,r5
    .syntax divided
    sbc r4,r0
    sbc r5,r1
    mov r0,#0
    sbc r6,r0
    sbc r7,r0
    .syntax unified
    mov r0,r10
    .syntax divided
    add r2,r0
    .syntax unified
    mov r1,r11
    .syntax divided
    adc r3,r1
    .syntax unified
    mov r0,r12
    .syntax divided
    adc r4,r0
    .syntax unified
    mov r0,r8
    .syntax divided
    adc r5,r0
    mov r0,#0
    adc r6,r0
    adc r7,r0
    .syntax unified
    mov r0,r10
    .syntax divided
 // END: sqr 128 Refined Karatsuba
 // Result in r0 ... r7 
    .syntax unified
    mov r8,r4
    .syntax divided
    .syntax unified
    mov r9,r5
    .syntax divided
    .syntax unified
    mov r10,r6
    .syntax divided
    .syntax unified
    mov r11,r7
    .syntax divided
    pop {r4,r5,r6,r7}
    add r0,r4
    adc r1,r5
    adc r2,r6
    adc r3,r7
    .syntax unified
    mov r4,r8
    .syntax divided
    .syntax unified
    mov r5,r9
    .syntax divided
    .syntax unified
    mov r6,r10
    .syntax divided
    .syntax unified
    mov r7,r11
    .syntax divided
    .syntax unified
    mov r8,r0
    .syntax divided
    mov r0,#0
    adc r4,r0
    adc r5,r0
    adc r6,r0
    adc r7,r0
    .syntax unified
    mov r0,r8
    .syntax divided
    push {r0,r1,r2,r3,r4,r5,r6,r7}
    ldr r4,[SP,#52]
    ldm r4,{r0,r1,r2,r3,r4,r5,r6,r7}
    sub r4,r0
    sbc r5,r1
    sbc r6,r2
    sbc r7,r3
    sbc r0,r0
    eor r4,r0
    eor r5,r0
    eor r6,r0
    eor r7,r0
    sub r4,r0
    sbc r5,r0
    sbc r6,r0
    sbc r7,r0
 // sqr 128 Refined Karatsuba
 // Input in r4 ... r7 
 // Result in r0 ... r7 
 // clobbers all registers except for r14 
    .syntax unified
    mov r0,r4
    .syntax divided
    .syntax unified
    mov r1,r5
    .syntax divided
    sub r0,r6
    sbc r1,r7
    sbc r2,r2
    eor r0,r2
    eor r1,r2
    sub r0,r2
    sbc r1,r2
    .syntax unified
    mov r8,r0
    .syntax divided
    .syntax unified
    mov r9,r1
    .syntax divided
    .syntax unified
    mov r10,r6
    .syntax divided
 // START: sqr 64 Refined Karatsuba
 // Input operands in r4,r5
 // Result in r0,r1,r2,r3
 // Clobbers: r4-r6
 // START: sqr 32
 // Input operand in r4 
 // Result in r0 ,r1 
 // Clobbers: r2, r3 
/*
    uxth r0,r4
    lsr r1,r4,#16
    .syntax unified
    mov r2,r0
    .syntax divided
    mul r2,r1
    mul r0,r0
    mul r1,r1
    lsr r3,r2,#15
    lsl r2,r2,#17
    add r0,r2
    adc r1,r3
*/
    UMULL r0,r1, r4,r4
 // End: sqr 32
 // Result in r0 ,r1 
    sub r4,r5
    sbc r6,r6
    eor r4,r6
    sub r4,r6
 // START: sqr 32
 // Input operand in r5 
 // Result in r2 ,r3 
 // Clobbers: r5, r6 
/*
    uxth r2,r5
    lsr r3,r5,#16
    .syntax unified
    mov r5,r2
    .syntax divided
    mul r5,r3
    mul r2,r2
    mul r3,r3
    lsr r6,r5,#15
    lsl r5,r5,#17
    add r2,r5
    adc r3,r6
*/
    UMULL r2,r3, r5,r5
 // End: sqr 32
 // Result in r2 ,r3 
    mov r6,#0
    add r2,r1
    adc r3,r6
 // START: sqr 32
 // Input operand in r4 
 // Result in r4 ,r5 
 // Clobbers: r1, r6 

/*
    lsr r5,r4,#16
    uxth r4,r4
    .syntax unified
    mov r1,r4
    .syntax divided
    mul r1,r5
    mul r4,r4
    mul r5,r5
    lsr r6,r1,#15
    lsl r1,r1,#17
    add r4,r1
    adc r5,r6
*/
    UMULL r4,r5, r4,r4
 // End: sqr 32
 // Result in r4 ,r5 
    .syntax unified
    mov r1,r2
    .syntax divided
    sub r1,r4
    sbc r2,r5
    .syntax unified
    mov r5,r3
    .syntax divided
    mov r6,#0
    sbc r3,r6
    add r1,r0
    adc r2,r5
    adc r3,r6
 // END: sqr 64 Refined Karatsuba
 // Result in r0,r1,r2,r3
 // Leaves r6 zero.
    .syntax unified
    mov r6,r10
    .syntax divided
    .syntax unified
    mov r10,r0
    .syntax divided
    .syntax unified
    mov r11,r1
    .syntax divided
    .syntax unified
    mov r12,r2
    .syntax divided
    .syntax unified
    mov r1,r3
    .syntax divided
 // START: sqr 64 Refined Karatsuba
 // Input operands in r6,r7
 // Result in r2,r3,r4,r5 
 // Clobbers: r0,r7,r6
 // START: sqr 32
 // Input operand in r6 
 // Result in r2 ,r3 
 // Clobbers: r4, r5 

/*
    uxth r2,r6
    lsr r3,r6,#16
    .syntax unified
    mov r4,r2
    .syntax divided
    mul r4,r3
    mul r2,r2
    mul r3,r3
    lsr r5,r4,#15
    lsl r4,r4,#17
    add r2,r4
    adc r3,r5
*/

    UMULL r2,r3, r6,r6
 // End: sqr 32
 // Result in r2 ,r3 
    sub r6,r7
    sbc r4,r4
    eor r6,r4
    sub r6,r4
 // START: sqr 32
 // Input operand in r7 
 // Result in r4 ,r5 
 // Clobbers: r0, r7 
/*
    uxth r4,r7
    lsr r5,r7,#16
    .syntax unified
    mov r0,r4
    .syntax divided
    mul r0,r5
    mul r4,r4
    mul r5,r5
    lsr r7,r0,#15
    lsl r0,r0,#17
    add r4,r0
    adc r5,r7
*/
    UMULL r4,r5, r7,r7
 // End: sqr 32
 // Result in r4 ,r5 
    mov r7,#0
    add r4,r3
    adc r5,r7
 // START: sqr 32
 // Input operand in r6 
 // Result in r7 ,r0 
 // Clobbers: r6, r3 
/*
    uxth r7,r6
    lsr r0,r6,#16
    .syntax unified
    mov r6,r7
    .syntax divided
    mul r6,r0
    mul r7,r7
    mul r0,r0
    lsr r3,r6,#15
    lsl r6,r6,#17
    add r7,r6
    adc r0,r3
*/
    UMULL r7,r0, r6,r6
 // End: sqr 32
 // Result in r7 ,r0 
    .syntax unified
    mov r3,r4
    .syntax divided
    sub r3,r7
    sbc r4,r0
    .syntax unified
    mov r0,r5
    .syntax divided
    mov r6,#0
    sbc r5,r6
    add r3,r2
    adc r4,r0
    adc r5,r6
 // END: sqr 64 Refined Karatsuba
 // Result in r2,r3,r4,r5
 // Leaves r6 zero.
    .syntax unified
    mov r0,r12
    .syntax divided
    add r2,r0
    adc r3,r1
    adc r4,r6
    adc r5,r6
    .syntax unified
    mov r12,r2
    .syntax divided
    .syntax unified
    mov r2,r8
    .syntax divided
    .syntax unified
    mov r8,r3
    .syntax divided
    .syntax unified
    mov r3,r9
    .syntax divided
    .syntax unified
    mov r9,r4
    .syntax divided
 // START: sqr 64 Refined Karatsuba
 // Input operands in r2,r3
 // Result in r6,r7,r0,r1 
 // Clobbers: r2,r3,r4
 // START: sqr 32
 // Input operand in r2 
 // Result in r6 ,r7 
 // Clobbers: r0, r1 

/*
    uxth r6,r2
    lsr r7,r2,#16
    .syntax unified
    mov r0,r6
    .syntax divided
    mul r0,r7
    mul r6,r6
    mul r7,r7
    lsr r1,r0,#15
    lsl r0,r0,#17
    add r6,r0
    adc r7,r1
*/
    UMULL r6,r7, r2,r2
 // End: sqr 32
 // Result in r6 ,r7 
    sub r2,r3
    sbc r4,r4
    eor r2,r4
    sub r2,r4
 // START: sqr 32
 // Input operand in r3 
 // Result in r0 ,r1 
 // Clobbers: r3, r4 

/*
    uxth r0,r3
    lsr r1,r3,#16
    .syntax unified
    mov r3,r0
    .syntax divided
    mul r3,r1
    mul r0,r0
    mul r1,r1
    lsr r4,r3,#15
    lsl r3,r3,#17
    add r0,r3
    adc r1,r4
*/
    UMULL r0,r1, r3,r3
 // End: sqr 32
 // Result in r0 ,r1 
    mov r4,#0
    add r0,r7
    adc r1,r4
 // START: sqr 32
 // Input operand in r2 
 // Result in r3 ,r4 
 // Clobbers: r2, r7 
/*
    uxth r3,r2
    lsr r4,r2,#16
    .syntax unified
    mov r2,r3
    .syntax divided
    mul r2,r4
    mul r3,r3
    mul r4,r4
    lsr r7,r2,#15
    lsl r2,r2,#17
    add r3,r2
    adc r4,r7
*/
    UMULL r3,r4, r2,r2
 // End: sqr 32
 // Result in r3 ,r4 
    .syntax unified
    mov r7,r0
    .syntax divided
    sub r7,r3
    sbc r0,r4
    .syntax unified
    mov r2,r1
    .syntax divided
    mov r4,#0
    sbc r1,r4
    add r7,r6
    adc r0,r2
    adc r1,r4
 // END: sqr 64 Refined Karatsuba
 // Result in r6,r7,r0,r1
 // Returns r4 as zero.
    .syntax unified
    mov r2,r12
    .syntax divided
    .syntax unified
    mov r3,r8
    .syntax divided
    .syntax unified
    mov r4,r9
    .syntax divided
    sub r2,r6
    sbc r3,r7
    .syntax unified
    mov r6,r4
    .syntax divided
    .syntax unified
    mov r7,r5
    .syntax divided
    sbc r4,r0
    sbc r5,r1
    mov r0,#0
    sbc r6,r0
    sbc r7,r0
    .syntax unified
    mov r0,r10
    .syntax divided
    add r2,r0
    .syntax unified
    mov r1,r11
    .syntax divided
    adc r3,r1
    .syntax unified
    mov r0,r12
    .syntax divided
    adc r4,r0
    .syntax unified
    mov r0,r8
    .syntax divided
    adc r5,r0
    mov r0,#0
    adc r6,r0
    adc r7,r0
    .syntax unified
    mov r0,r10
    .syntax divided
 // END: sqr 128 Refined Karatsuba
 // Result in r0 ... r7 
    mvn r0,r0
    mvn r1,r1
    mvn r2,r2
    mvn r3,r3
    mvn r4,r4
    mvn r5,r5
    mvn r6,r6
    mvn r7,r7
    .syntax unified
    mov r8,r4
    .syntax divided
    .syntax unified
    mov r9,r5
    .syntax divided
    .syntax unified
    mov r10,r6
    .syntax divided
    .syntax unified
    mov r11,r7
    .syntax divided
    mov r4,#143
    asr r4,r4,#1
    pop {r4,r5,r6,r7}
    adc r0,r4
    adc r1,r5
    adc r2,r6
    adc r3,r7
    .syntax unified
    mov r12,r4
    .syntax divided
    mov r4,#16
    add r4,r14
    stm r4!,{r0,r1,r2,r3}
    .syntax unified
    mov r4,r12
    .syntax divided
    .syntax unified
    mov r0,r8
    .syntax divided
    adc r0,r4
    .syntax unified
    mov r8,r0
    .syntax divided
    .syntax unified
    mov r1,r9
    .syntax divided
    adc r1,r5
    .syntax unified
    mov r9,r1
    .syntax divided
    .syntax unified
    mov r2,r10
    .syntax divided
    adc r2,r6
    .syntax unified
    mov r10,r2
    .syntax divided
    .syntax unified
    mov r3,r11
    .syntax divided
    adc r3,r7
    .syntax unified
    mov r11,r3
    .syntax divided
    mov r0,#0
    adc r0,r0
    .syntax unified
    mov r12,r0
    .syntax divided
    .syntax unified
    mov r0,r14
    .syntax divided
    ldm r0,{r0,r1,r2,r3,r4,r5,r6,r7}
    add r0,r4
    adc r1,r5
    adc r2,r6
    adc r3,r7
    mov r4,#16
    add r4,r14
    stm r4!,{r0,r1,r2,r3}
    .syntax unified
    mov r14,r4
    .syntax divided
    .syntax unified
    mov r0,r13
    .syntax divided
    ldm r0!,{r4,r5,r6,r7}
    .syntax unified
    mov r1,r8
    .syntax divided
    adc r4,r1
    .syntax unified
    mov r1,r9
    .syntax divided
    adc r5,r1
    .syntax unified
    mov r1,r10
    .syntax divided
    adc r6,r1
    .syntax unified
    mov r1,r11
    .syntax divided
    adc r7,r1
    .syntax unified
    mov r0,r14
    .syntax divided
    stm r0!,{r4,r5,r6,r7}
    pop {r4,r5,r6,r7}
    .syntax unified
    mov r1,r12
    .syntax divided
    mov r2,#0
    mvn r2,r2
    adc r1,r2
    asr r2,r1,#4
    add r4,r1
    adc r5,r2
    adc r6,r2
    adc r7,r2
    stm r0!,{r4,r5,r6,r7}
    pop {r3,r4,r5,r6,r7}
    .syntax unified
    mov r8,r3
    .syntax divided
    .syntax unified
    mov r9,r4
    .syntax divided
    .syntax unified
    mov r10,r5
    .syntax divided
    .syntax unified
    mov r11,r6
    .syntax divided
    .syntax unified
    mov r12,r7
    .syntax divided
    pop {r0,r4,r5,r6,r7,r15}
//Cycle Count ASM-Version of 256 sqr (Refined Karatsuba) (Cortex M0): 793 (697 instructions).
	.size	square256_asm, .-square256_asm
