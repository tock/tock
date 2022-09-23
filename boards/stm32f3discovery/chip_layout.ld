/* Memory layout for the STM32F303VCT6
 * rom = 256KB (LENGTH = 0x00040000)
 * kernel = 128KB
 * user = 128KB
 * ram = 40KB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 128K
  prog (rx) : ORIGIN = 0x08020000, LENGTH = 128K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 40K
}

PAGE_SIZE = 2K;
