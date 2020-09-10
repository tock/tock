/* Memory Space Definitions, 512K flash, 64K ram */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 192K
  prog (rx) : ORIGIN = 0x00030000, LENGTH = 320K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}

MPU_MIN_ALIGN = 8K;
PAGE_SIZE = 4K;
