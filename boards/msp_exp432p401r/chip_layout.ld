/* Memory Space Definitions, 256K flash, 64K ram */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 128K
  prog (rx) : ORIGIN = 0x00020000, LENGTH = 128K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}

MPU_MIN_ALIGN = 8K;
