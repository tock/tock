/* Memory Space Definitions, 360K flash, 80K ram */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 64K
  prog (rx) : ORIGIN = 0x00010000, LENGTH = 192K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}

MPU_MIN_ALIGN = 4K;
