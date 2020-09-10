/* Memory layout for the STM32F446RE
 * rom = 2MB (LENGTH = 0x02000000)
 * kernel = 256KB
 * user = 256KB
 * ram = 192KB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 0x00040000
  prog (rx) : ORIGIN = 0x08040000, LENGTH = 0x00040000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 0x00030000
}

MPU_MIN_ALIGN = 8K;
PAGE_SIZE = 2K;
