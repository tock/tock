/* Memory layout for the STM32F446RE
 * rom = 512KB
 * ram = 128KB */

/* For now there is no `prog` and we assign all the available rom and
 * ram */
MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 0x00080000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 0x00020000
}

MPU_MIN_ALIGN = 8K;
