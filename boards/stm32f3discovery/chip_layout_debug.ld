/* Memory layout for the STM32F303VCT6
 * rom = 256KB (LENGTH = 0x00040000)
 * kernel = 192KB
 * user = 64KB
 * ram = 48KB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 0x00030000
  prog (rx) : ORIGIN = 0x08030000, LENGTH = 0x00010000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 48K
}

MPU_MIN_ALIGN = 8K;
PAGE_SIZE = 2K;
