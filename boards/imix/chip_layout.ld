/* Memory Spaces Definitions, 448K flash, 64K ram */
/* Use bootloader starting at 0x0000 */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00010000, LENGTH = 0x00030000
  prog (rx) : ORIGIN = 0x00040000, LENGTH = 0x00040000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 0x00010000
}

MPU_MIN_ALIGN = 8K;
