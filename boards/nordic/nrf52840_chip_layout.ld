/* Memory Space Definitions, 1M flash, 256K ram */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 192K
  prog (rx) : ORIGIN = 0x00030000, LENGTH = 832K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 256K
}

MPU_MIN_ALIGN = 8K;
