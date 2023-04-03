/* Memory layout for the STM32F401CCU6
 * rom = 256KiB (LENGTH = 0x00040000)
 * kernel = 150KiB
 * user = 112KiB
 * ram = 64KiB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 150K
  prog (rx) : ORIGIN = 0x08025800, LENGTH = 112K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}

PAGE_SIZE = 2K;

INCLUDE ../kernel_layout.ld
