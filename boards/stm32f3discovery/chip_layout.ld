/* Memory layout for the STM32F303VCT6
 * rom = 256KB (LENGTH = 0x00040000)
 * kernel = 128KB
 * user = 128KB
 * ram = 48KB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 0x00020000
  prog (rx) : ORIGIN = 0x08020000, LENGTH = 0x00020000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 48K
}

MPU_MIN_ALIGN = 8K;
