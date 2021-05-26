/* The RISC-V semihosting spec needs three exact instructions
 * in uncompressed form.
 * See https://github.com/riscv/riscv-semihosting-spec/blob/main/riscv-semihosting-spec.adoc#11-semihosting-trap-instruction-sequence
 * for more details on the three insturctions.
 *
 * Using Rust's inline asm
 * ```rust
 *        asm!(
 *           "slli x0, x0, 0x1f
 *           ebreak
 *           srai x0, x0, 7"
 *       );
 * ```
 * incorectly generates the compressed `9002` form of ebreak as the
 * assembler will compress the instructions from `ebreak` to `c.ebreak`.
 * This is the correct behaviour as the assembler should try to compress
 * the instructions if it can.
 *
 * In order to work with semihosting we include the assembly here
 * where we are able to disable compressed instruction support. This
 * follows the example in the RISC-V semihosting spec.
 */
.global semihost_command
.option norvc
.balign 16
semihost_command:
	slli x0, x0, 0x1f		# 0x01f01013		Entry NOP
	ebreak					# 0x00100073		Break to debugger
	srai x0, x0, 7			# 0x40705013		NOP encoding the semihosting call number 7
