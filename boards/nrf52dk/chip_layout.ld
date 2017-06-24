/* Memory Space Definitions, 512K flash, 64K ram */
ROM_ORIGIN  = 0x00000000;
ROM_LENGTH  = 128K;         /* assumming this is for the booloader */
PROG_ORIGIN = 0x00020000;
PROG_LENGTH = 384K;
RAM_ORIGIN  = 0x20000000;
RAM_LENGTH  = 512K;

MPU_MIN_ALIGN = 8;
