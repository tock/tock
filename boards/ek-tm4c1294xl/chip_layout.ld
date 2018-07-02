/* Memory Spaces Definitions, 1024K flash, 256K ram */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 128K
  prog (rx) : ORIGIN = 0x00020000, LENGTH = 896K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 256K
}

MPU_MIN_ALIGN = 8K;
