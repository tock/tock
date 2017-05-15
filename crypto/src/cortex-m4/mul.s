g// This assmeble file was taken from
// https://munacl.cryptojedi.org/curve25519-cortexm0.shtml
// Credit to  Björn Haase and Ana Helena Sánchez, as well
// as M. Hutter and P. Schwabe
// This assemble file exports the function multiply256x256_asm
// with 3 inputs, each of type [u32;8]. The program will multiple
// the first two inputs in modulo 2^255 − 19 and puts the result
//in the third input.
//The commented out code was the multiplication that was 
//replaces by the faster UMULL (supported by the arm cortex m4)


 .align	2
	.global	multiply256x256_asm
	.type	multiply256x256_asm, %function
multiply256x256_asm:
	push {r4-r7,lr}
	mov r3, r8
	mov r4, r9
	mov r5, r10
	mov r6, r11
	push {r0-r6}
	mov r12, r0
	mov r10, r2
	mov r11, r1
	mov r0,r2
	//ldm r0!, {r4,r5,r6,r7}
	ldm r0!, {r4,r5}
  add r0,#8
	ldm r1!, {r2,r3,r6,r7}
	push {r0,r1}
	/////////BEGIN LOW PART //////////////////////
		/////////MUL128/////////////
			//MUL64
			//Instructions for the variant of subtractive karasuba multiplication for the 0th and 1st two u32 result
			mov r6, r5
			mov r1, r2
			sub r5, r4
			sbc r0, r0
			eor r5, r0
			sub r5, r0
			sub r1, r3
			sbc r7, r7
			eor r1, r7
			sub r1, r7
			eor r7, r0
			mov r9, r1
			mov r8, r5
      UMULL r0,r1,r2,r4
      UMULL r2,r3,r6,r3
			eor r6, r6 //can we change this to be better??
			add r2, r1
			adc r3, r6
			mov r1, r9
			mov r5, r8
			mov r8, r0
      UMULL r1, r0, r1,r5
			eor r1,r7
			eor r0,r7
			eor r4, r4
			asr r7, r7, #1
			adc r1, r2
			adc r2, r0
			adc r7, r4
			mov r0, r8
			add r1, r0
			adc r2, r3
			adc r3, r7 
		//////////////////////////
	mov r4, r12
	stm r4!, {r0,r1} 
	//Instructions for the variant of subtractive karasuba multiplication for the 2nd and 3rd lower u32 result
	push {r4}
		push {r0,r1}
		mov r1, r10
		mov r10, r2
		ldm r1, {r0, r1, r4, r5}
		mov r2, r4
		mov r7, r5
		sub r2, r0
		sbc r7, r1
		sbc r6, r6
		eor r2, r6
		eor r7, r6
		sub r2, r6
		sbc r7, r6
		push {r2, r7}
		mov r2, r11
		mov r11, r3
		ldm r2, {r0, r1, r2, r3}
		sub r0, r2
		sbc r1, r3
		sbc r7, r7
		eor r0, r7
		eor r1, r7
		sub r0, r7
		sbc r1, r7
		eor r7, r6	
		mov r12, r7
		push {r0, r1}
			//MUL64
			mov r6, r5
			mov r1, r2
			sub r5, r4
			sbc r0, r0
			eor r5, r0
			sub r5, r0
			sub r1, r3
			sbc r7, r7
			eor r1, r7
			sub r1, r7
			eor r7, r0
			mov r9, r1
			mov r8, r5

      UMULL r0,r1,r2,r4

      UMULL r2,r3,r6,r3
			eor r6, r6
			add r2, r1
			adc r3, r6
			mov r1, r9
			mov r5, r8
			mov r8, r0
      UMULL r1, r0, r1,r5
			eor r1,r7
			eor r0,r7
			eor r4, r4
			asr r7, r7, #1
			adc r1, r2
			adc r2, r0
			adc r7, r4
			mov r0, r8
			add r1, r0
			adc r2, r3
			adc r3, r7 
		mov r4, r10
		mov r5, r11
		eor r6, r6
		add r0, r4
		adc r1, r5
		adc r2, r6
		adc r3, r6
		mov r10, r2
		mov r11, r3
		pop {r2-r5}
		push {r0, r1}
			mov r6, r5
			mov r1, r2
			sub r5, r4
			sbc r0, r0
			eor r5, r0
			sub r5, r0
			sub r1, r3
			sbc r7, r7
			eor r1, r7
			sub r1, r7
			eor r7, r0
			mov r9, r1
			mov r8, r5
      UMULL r0,r1,r2,r4
      UMULL r2,r3,r6,r3
			eor r6, r6
			add r2, r1
			adc r3, r6
			mov r1, r9
			mov r5, r8
			mov r8, r0
      UMULL r1, r0, r1,r5
			eor r1,r7
			eor r0,r7
			eor r4, r4
			asr r7, r7, #1
			adc r1, r2
			adc r2, r0
			adc r7, r4
			mov r0, r8
			add r1, r0
			adc r2, r3
			adc r3, r7 
		pop {r4, r5}
		mov r6, r12
		mov r7, r12
		eor r0, r6
		eor r1, r6
		eor r2, r6
		eor r3, r6
		asr r6, r6, #1	
		adc r0, r4
		adc r1, r5
		adc r4, r2
		adc r5, r3
		eor r2, r2
		adc r6,r2 
		adc r7,r2
		pop {r2, r3}
		mov r8, r2
		mov r9, r3
		add r2, r0
		adc r3, r1
		mov r0, r10
		mov r1, r11
		adc r4, r0
		adc r5, r1
		adc r6, r0
		adc r7, r1
	////////END LOW PART/////////////////////
	pop {r0}
	stm r0!, {r2,r3}
	pop {r1,r2}
	push {r0}
	push {r4-r7}
	mov r10, r1
	mov r11, r2
	ldm r1!, {r4, r5}
	ldm r2, {r2, r3}
	/////////BEGIN HIGH PART////////////////
		/////////MUL128/////////////
			//MUL64
			//Instructions for the variant of subtractive karasuba multiplication for part of the lower 4th, 5th, 6th and 7th u32 result
			mov r6, r5
			mov r1, r2
			sub r5, r4
			sbc r0, r0
			eor r5, r0
			sub r5, r0
			sub r1, r3
			sbc r7, r7
			eor r1, r7
			sub r1, r7
			eor r7, r0
			mov r9, r1
			mov r8, r5

      UMULL r0,r1,r2,r4

      UMULL r2,r3,r6,r3
			eor r6, r6
			add r2, r1
			adc r3, r6
			mov r1, r9
			mov r5, r8
			mov r8, r0
       UMULL r1, r0, r1,r5

			eor r1,r7
			eor r0,r7
			eor r4, r4
			asr r7, r7, #1
			adc r1, r2
			adc r2, r0
			adc r7, r4
			mov r0, r8
			add r1, r0
			adc r2, r3
			adc r3, r7 
		push {r0,r1}
		mov r1, r10
		mov r10, r2
		ldm r1, {r0, r1, r4, r5}
		mov r2, r4
		mov r7, r5
		sub r2, r0
		sbc r7, r1
		sbc r6, r6
		eor r2, r6
		eor r7, r6
		sub r2, r6
		sbc r7, r6
		push {r2, r7}
		mov r2, r11
		mov r11, r3
		ldm r2, {r0, r1, r2, r3}
		sub r0, r2
		sbc r1, r3
		sbc r7, r7
		eor r0, r7
		eor r1, r7
		sub r0, r7
		sbc r1, r7
		eor r7, r6	
		mov r12, r7
		push {r0, r1}
			//MUL64
			mov r6, r5
			mov r1, r2
			sub r5, r4
			sbc r0, r0
			eor r5, r0
			sub r5, r0
			sub r1, r3
			sbc r7, r7
			eor r1, r7
			sub r1, r7
			eor r7, r0
			mov r9, r1
			mov r8, r5
      UMULL r0,r1,r2,r4
      UMULL r2,r3,r6,r3


			eor r6, r6
			add r2, r1
			adc r3, r6
			mov r1, r9
			mov r5, r8
			mov r8, r0
      UMULL r1, r0, r1,r5

			eor r1,r7
			eor r0,r7
			eor r4, r4
			asr r7, r7, #1
			adc r1, r2
			adc r2, r0
			adc r7, r4
			mov r0, r8
			add r1, r0
			adc r2, r3
			adc r3, r7 
		mov r4, r10
		mov r5, r11
		eor r6, r6
		add r0, r4
		adc r1, r5
		adc r2, r6
		adc r3, r6
		mov r10, r2
		mov r11, r3
		pop {r2-r5}
		push {r0, r1}
			mov r6, r5
			mov r1, r2
			sub r5, r4
			sbc r0, r0
			eor r5, r0
			sub r5, r0
			sub r1, r3
			sbc r7, r7
			eor r1, r7
			sub r1, r7
			eor r7, r0
			mov r9, r1
			mov r8, r5
      UMULL r0,r1,r2,r4
      UMULL r2,r3,r6,r3


			eor r6, r6
			add r2, r1
			adc r3, r6
			mov r1, r9
			mov r5, r8
			mov r8, r0
      UMULL r1, r0, r1,r5

			eor r1,r7
			eor r0,r7
			eor r4, r4
			asr r7, r7, #1
			adc r1, r2
			adc r2, r0
			adc r7, r4
			mov r0, r8
			add r1, r0
			adc r2, r3
			adc r3, r7 
		pop {r4, r5}
		mov r6, r12
		mov r7, r12
		eor r0, r6
		eor r1, r6
		eor r2, r6
		eor r3, r6
		asr r6, r6, #1	
		adc r0, r4
		adc r1, r5
		adc r4, r2
		adc r5, r3
		eor r2, r2
		adc r6,r2 //0,1
		adc r7,r2
		pop {r2, r3}
		mov r8, r2
		mov r9, r3
		add r2, r0
		adc r3, r1
		mov r0, r10
		mov r1, r11
		adc r4, r0
		adc r5, r1
		adc r6, r0
		adc r7, r1
	////////END HIGH PART/////////////////////
	mov r0, r8
	mov r1, r9
	mov r8, r6
	mov r9, r7
	pop {r6, r7}
	add r0, r6
	adc r1, r7
	pop {r6, r7}
	adc r2, r6
	adc r3, r7
	pop {r7}
	stm r7!, {r0-r3}
	//Instructions for the variant of subtractive karasuba multiplication for part of the lower 8th, 9th, 10th and 11th u32 result
	mov r10, r7
	eor r0,r0
	mov r6, r8
	mov r7, r9
	adc r4, r0
	adc r5, r0
	adc r6, r0
	adc r7, r0
	pop {r0,r1,r2}
	mov r12, r2
	push {r0, r4-r7}
	ldm r1, {r0-r7}
	sub r0, r4
	sbc r1, r5
	sbc r2, r6
	sbc r3, r7
	eor r4, r4
	sbc r4, r4
	eor r0, r4
	eor r1, r4
	eor r2, r4
	eor r3, r4
	sub r0, r4
	sbc r1, r4
	sbc r2, r4
	sbc r3, r4
	mov r6, r12
	mov r12, r4 //carry
	mov r5, r10
	stm r5!, {r0-r3}
	//Instructions for the variant of subtractive karasuba multiplication for part of the lower 12th, 13th, 14th and 15th u32 result
	mov r11, r5
	mov r8, r0
	mov r9, r1
	ldm r6, {r0-r7}
	sub r4, r0
	sbc r5, r1
	sbc r6, r2
	sbc r7, r3
	eor r0, r0
	sbc r0, r0
	eor r4, r0
	eor r5, r0
	eor r6, r0
	eor r7, r0
	sub r4, r0
	sbc r5, r0
	sbc r6, r0
	sbc r7, r0
	mov r1, r12
	eor r0, r1
	mov r1, r11
	stm r1!, {r4-r7}
	push {r0}
	mov r2, r8
	mov r3, r9
	/////////BEGIN MIDDLE PART////////////////
		/////////MUL128/////////////
			//MUL64
			//Instructions for the variant of subtractive karasuba multiplication for part of the lower 4th, 5th, 6th and 7th u32 result
			mov r6, r5
			mov r1, r2
			sub r5, r4
			sbc r0, r0
			eor r5, r0
			sub r5, r0
			sub r1, r3
			sbc r7, r7
			eor r1, r7
			sub r1, r7
			eor r7, r0
			mov r9, r1
			mov r8, r5
       UMULL r0,r1,r2,r4
      UMULL r2,r3,r6,r3

			eor r6, r6
			add r2, r1
			adc r3, r6
			mov r1, r9
			mov r5, r8
			mov r8, r0
      UMULL r1, r0, r1,r5

			eor r1,r7
			eor r0,r7
			eor r4, r4
			asr r7, r7, #1
			adc r1, r2
			adc r2, r0
			adc r7, r4
			mov r0, r8
			add r1, r0
			adc r2, r3
			adc r3, r7 
		push {r0,r1}
		mov r1, r10
		mov r10, r2
		ldm r1, {r0, r1, r4, r5}
		mov r2, r4
		mov r7, r5
		sub r2, r0
		sbc r7, r1
		sbc r6, r6
		eor r2, r6
		eor r7, r6
		sub r2, r6
		sbc r7, r6
		push {r2, r7}
		mov r2, r11
		mov r11, r3
		ldm r2, {r0, r1, r2, r3}
		sub r0, r2
		sbc r1, r3
		sbc r7, r7
		eor r0, r7
		eor r1, r7
		sub r0, r7
		sbc r1, r7
		eor r7, r6	
		mov r12, r7
		push {r0, r1}
			//MUL64
			mov r6, r5
			mov r1, r2
			sub r5, r4
			sbc r0, r0
			eor r5, r0
			sub r5, r0
			sub r1, r3
			sbc r7, r7
			eor r1, r7
			sub r1, r7
			eor r7, r0
			mov r9, r1
			mov r8, r5
      UMULL r0,r1,r2,r4
      UMULL r2,r3,r6,r3

			eor r6, r6
			add r2, r1
			adc r3, r6
			mov r1, r9
			mov r5, r8
			mov r8, r0
      UMULL r1, r0, r1,r5


			eor r1,r7
			eor r0,r7
			eor r4, r4
			asr r7, r7, #1
			adc r1, r2
			adc r2, r0
			adc r7, r4
			mov r0, r8
			add r1, r0
			adc r2, r3
			adc r3, r7 
		mov r4, r10
		mov r5, r11
		eor r6, r6
		add r0, r4
		adc r1, r5
		adc r2, r6
		adc r3, r6
		mov r10, r2
		mov r11, r3
		pop {r2-r5}
		push {r0, r1}
			mov r6, r5
			mov r1, r2
			sub r5, r4
			sbc r0, r0
			eor r5, r0
			sub r5, r0
			sub r1, r3
			sbc r7, r7
			eor r1, r7
			sub r1, r7
			eor r7, r0
			mov r9, r1
			mov r8, r5
      UMULL r0,r1,r2,r4
      UMULL r2,r3,r6,r3

			eor r6, r6
			add r2, r1
			adc r3, r6
			mov r1, r9
			mov r5, r8
			mov r8, r0
      UMULL r1, r0, r1,r5

			eor r1,r7
			eor r0,r7
			eor r4, r4
			asr r7, r7, #1
			adc r1, r2
			adc r2, r0
			adc r7, r4
			mov r0, r8
			add r1, r0
			adc r2, r3
			adc r3, r7 
		pop {r4, r5}
		mov r6, r12
		mov r7, r12
		eor r0, r6
		eor r1, r6
		eor r2, r6
		eor r3, r6
		asr r6, r6, #1	
		adc r0, r4
		adc r1, r5
		adc r4, r2
		adc r5, r3
		eor r2, r2
		adc r6,r2 //0,1
		adc r7,r2
		pop {r2, r3}
		mov r8, r2
		mov r9, r3
		add r2, r0
		adc r3, r1
		mov r0, r10
		mov r1, r11
		adc r4, r0
		adc r5, r1
		adc r6, r0
		adc r7, r1
	//////////END MIDDLE PART////////////////
	pop {r0,r1} //r0,r1
	mov r12, r0 //negative
	eor r2, r0
	eor r3, r0
	eor r4, r0
	eor r5, r0
	eor r6, r0
	eor r7, r0
	push {r4-r7}
	ldm r1!, {r4-r7}
	mov r11, r1 //reference
	mov r1, r9
	eor r1, r0
	mov r10, r4
	mov r4, r8
	asr r0, #1
	eor r0, r4
	mov r4, r10
	adc r0, r4
	adc r1, r5
	adc r2, r6
	adc r3, r7
	eor r4, r4
	adc r4, r4
	mov r10, r4 //carry
	mov r4, r11
	ldm r4, {r4-r7}
	add r0, r4
	adc r1, r5
	adc r2, r6
	adc r3, r7
	mov r9, r4
	mov r4, r11
	stm r4!, {r0-r3}
	//Instructions for the variant of subtractive karasuba multiplication for part of the lower 8th, 9th, 10th and 11th u32 result
	mov r11, r4
	pop {r0-r3}
	mov r4, r9
	adc r4, r0
	adc r5, r1
	adc r6, r2
	adc r7, r3
	mov r1, #0
	adc r1, r1
	mov r0, r10
	mov r10, r1 //carry
	asr r0, #1
	pop {r0-r3}
	adc r4, r0
	adc r5, r1
	adc r6, r2
	adc r7, r3
	mov r8, r0
	mov r0, r11
	stm r0!, {r4-r7}
	//Instructions for the variant of subtractive karasuba multiplication for part of the lower 12th, 13th, 14th and 15th u32 result
	mov r11, r0
	mov r0, r8
	mov r6, r12
	mov r5, r10
	eor r4, r4
	adc r5, r6
	adc r6, r4
	add r0, r5
	adc r1, r6
	adc r2, r6
	adc r3, r6
	mov r7, r11
	stm r7!, {r0-r3}
	pop {r3-r6}
	mov r8, r3
	mov r9, r4
	mov r10, r5
	mov r11, r6
	pop {r4-r7,pc}
	bx	lr
.size	multiply256x256_asm, .-multiply256x256_asm
