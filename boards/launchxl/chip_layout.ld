/* Memory Space Definitions, 360K flash, 80K ram */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 0x00030000
  prog (rx) : ORIGIN = 0x00030000, LENGTH = 0x00010000
  ccfg (rx) : ORIGIN = 0x00057FA8, LENGTH = 0x00000058
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 80K
}

MPU_MIN_ALIGN = 4K;
