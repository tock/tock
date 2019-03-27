/* Memory Space Definitions, 512K flash, 64K ram */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 158K
  prog (rx) : ORIGIN = 0x00020000, LENGTH = 354K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}

MPU_MIN_ALIGN = 8K;
