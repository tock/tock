/* Memory layout for the STM32F446RE
 * rom = 1MB (LENGTH = 0x01000000)
 * kernel = 256KB
 * user = 256KB
 * ram = 256KB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 0x00040000
  prog (rx) : ORIGIN = 0x08040000, LENGTH = 0x00040000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 0x0003FFFF
}

MPU_MIN_ALIGN = 8K;
