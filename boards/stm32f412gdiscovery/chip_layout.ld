/* Memory layout for the STM32F412G
 * rom = 1MB (LENGTH = 0x01000000)
 * kernel = 192KB
 * user = 832KB
 * ram = 256KB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 0x00030000
  prog (rx) : ORIGIN = 0x08030000, LENGTH = 0x000D0000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 0x0003FFFF
}

MPU_MIN_ALIGN = 8K;
