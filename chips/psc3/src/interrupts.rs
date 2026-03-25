/// GPIO Port Interrupt #0 (dec: 0)
pub const IOSS_INTERRUPTS_GPIO_0: u32 = 0x00;
/// GPIO Port Interrupt #1 (dec: 1)
pub const IOSS_INTERRUPTS_GPIO_1: u32 = 0x01;
/// GPIO Port Interrupt #2 (dec: 2)
pub const IOSS_INTERRUPTS_GPIO_2: u32 = 0x02;
/// GPIO Port Interrupt #3 (dec: 3)
pub const IOSS_INTERRUPTS_GPIO_3: u32 = 0x03;
/// GPIO Port Interrupt #4 (dec: 4)
pub const IOSS_INTERRUPTS_GPIO_4: u32 = 0x04;
/// GPIO Port Interrupt #5 (dec: 5)
pub const IOSS_INTERRUPTS_GPIO_5: u32 = 0x05;
/// GPIO Port Interrupt #6 (dec: 6)
pub const IOSS_INTERRUPTS_GPIO_6: u32 = 0x06;
/// GPIO Port Interrupt #7 (dec: 7)
pub const IOSS_INTERRUPTS_GPIO_7: u32 = 0x07;
/// GPIO Port Interrupt #8 (dec: 8)
pub const IOSS_INTERRUPTS_GPIO_8: u32 = 0x08;
/// GPIO Port Interrupt #9 (dec: 9)
pub const IOSS_INTERRUPTS_GPIO_9: u32 = 0x09;
/// GPIO Port Secure Interrupt #0 (dec: 10)
pub const IOSS_INTERRUPTS_SEC_GPIO_0: u32 = 0x0A;
/// GPIO Port Secure Interrupt #1 (dec: 11)
pub const IOSS_INTERRUPTS_SEC_GPIO_1: u32 = 0x0B;
/// GPIO Port Secure Interrupt #2 (dec: 12)
pub const IOSS_INTERRUPTS_SEC_GPIO_2: u32 = 0x0C;
/// GPIO Port Secure Interrupt #3 (dec: 13)
pub const IOSS_INTERRUPTS_SEC_GPIO_3: u32 = 0x0D;
/// GPIO Port Secure Interrupt #4 (dec: 14)
pub const IOSS_INTERRUPTS_SEC_GPIO_4: u32 = 0x0E;
/// GPIO Port Secure Interrupt #5 (dec: 15)
pub const IOSS_INTERRUPTS_SEC_GPIO_5: u32 = 0x0F;
/// GPIO Port Secure Interrupt #6 (dec: 16)
pub const IOSS_INTERRUPTS_SEC_GPIO_6: u32 = 0x10;
/// GPIO Port Secure Interrupt #7 (dec: 17)
pub const IOSS_INTERRUPTS_SEC_GPIO_7: u32 = 0x11;
/// GPIO Port Secure Interrupt #8 (dec: 18)
pub const IOSS_INTERRUPTS_SEC_GPIO_8: u32 = 0x12;
/// GPIO Port Secure Interrupt #9 (dec: 19)
pub const IOSS_INTERRUPTS_SEC_GPIO_9: u32 = 0x13;
/// GPIO Supply Detect Interrupt (dec: 20)
pub const IOSS_INTERRUPT_VDD: u32 = 0x14;
/// GPIO All Ports - Interrupts (dec: 21)
pub const IOSS_INTERRUPT_GPIO: u32 = 0x15;
/// GPIO All Ports  - Secure  Interrupts (dec: 22)
pub const IOSS_INTERRUPT_SEC_GPIO: u32 = 0x16;
/// Serial Communication Block #0 (DeepSleep capable) (dec: 23)
pub const SCB_0_INTERRUPT: u32 = 0x17;
/// Multi Counter Watchdog Timer interrupt (dec: 24)
pub const SRSS_INTERRUPT_MCWDT_0: u32 = 0x18;
/// Backup domain interrupt (dec: 25)
pub const SRSS_INTERRUPT_BACKUP: u32 = 0x19;
/// cpuss Inter Process Communication Interrupt #0 (dec: 26)
pub const CPUSS_INTERRUPTS_IPC_DPSLP_0: u32 = 0x1A;
/// cpuss Inter Process Communication Interrupt #1 (dec: 27)
pub const CPUSS_INTERRUPTS_IPC_DPSLP_1: u32 = 0x1B;
/// Interrupt from WDT (dec: 28)
pub const SRSS_INTERRUPT_WDT: u32 = 0x1C;
/// LPCOMP (dec: 29)
pub const LPCOMP_INTERRUPT: u32 = 0x1D;
/// Other combined Interrupts for srss (LVD and CLKCAL, CLKCAL only supported in Active mode) (dec: 30)
pub const SRSS_INTERRUPT: u32 = 0x1E;
/// Serial Communication Block #1 (dec: 31)
pub const SCB_1_INTERRUPT: u32 = 0x1F;
/// Serial Communication Block #2 (dec: 32)
pub const SCB_2_INTERRUPT: u32 = 0x20;
/// Serial Communication Block #3 (dec: 33)
pub const SCB_3_INTERRUPT: u32 = 0x21;
/// Serial Communication Block #4 (dec: 34)
pub const SCB_4_INTERRUPT: u32 = 0x22;
/// Serial Communication Block #5 (dec: 35)
pub const SCB_5_INTERRUPT: u32 = 0x23;
/// FLASH Macro interrupt (dec: 36)
pub const CPUSS_INTERRUPT_FM_CBUS: u32 = 0x24;
/// cpuss DataWire #0, Channel #0 (dec: 37)
pub const CPUSS_INTERRUPTS_DW0_0: u32 = 0x25;
/// cpuss DataWire #0, Channel #1 (dec: 38)
pub const CPUSS_INTERRUPTS_DW0_1: u32 = 0x26;
/// cpuss DataWire #0, Channel #2 (dec: 39)
pub const CPUSS_INTERRUPTS_DW0_2: u32 = 0x27;
/// cpuss DataWire #0, Channel #3 (dec: 40)
pub const CPUSS_INTERRUPTS_DW0_3: u32 = 0x28;
/// cpuss DataWire #0, Channel #4 (dec: 41)
pub const CPUSS_INTERRUPTS_DW0_4: u32 = 0x29;
/// cpuss DataWire #0, Channel #5 (dec: 42)
pub const CPUSS_INTERRUPTS_DW0_5: u32 = 0x2A;
/// cpuss DataWire #0, Channel #6 (dec: 43)
pub const CPUSS_INTERRUPTS_DW0_6: u32 = 0x2B;
/// cpuss DataWire #0, Channel #7 (dec: 44)
pub const CPUSS_INTERRUPTS_DW0_7: u32 = 0x2C;
/// cpuss DataWire #0, Channel #8 (dec: 45)
pub const CPUSS_INTERRUPTS_DW0_8: u32 = 0x2D;
/// cpuss DataWire #0, Channel #9 (dec: 46)
pub const CPUSS_INTERRUPTS_DW0_9: u32 = 0x2E;
/// cpuss DataWire #0, Channel #10 (dec: 47)
pub const CPUSS_INTERRUPTS_DW0_10: u32 = 0x2F;
/// cpuss DataWire #0, Channel #11 (dec: 48)
pub const CPUSS_INTERRUPTS_DW0_11: u32 = 0x30;
/// cpuss DataWire #0, Channel #12 (dec: 49)
pub const CPUSS_INTERRUPTS_DW0_12: u32 = 0x31;
/// cpuss DataWire #0, Channel #13 (dec: 50)
pub const CPUSS_INTERRUPTS_DW0_13: u32 = 0x32;
/// cpuss DataWire #0, Channel #14 (dec: 51)
pub const CPUSS_INTERRUPTS_DW0_14: u32 = 0x33;
/// cpuss DataWire #0, Channel #15 (dec: 52)
pub const CPUSS_INTERRUPTS_DW0_15: u32 = 0x34;
/// cpuss DataWire #1, Channel #0 (dec: 53)
pub const CPUSS_INTERRUPTS_DW1_0: u32 = 0x35;
/// cpuss DataWire #1, Channel #1 (dec: 54)
pub const CPUSS_INTERRUPTS_DW1_1: u32 = 0x36;
/// cpuss DataWire #1, Channel #2 (dec: 55)
pub const CPUSS_INTERRUPTS_DW1_2: u32 = 0x37;
/// cpuss DataWire #1, Channel #3 (dec: 56)
pub const CPUSS_INTERRUPTS_DW1_3: u32 = 0x38;
/// cpuss DataWire #1, Channel #4 (dec: 57)
pub const CPUSS_INTERRUPTS_DW1_4: u32 = 0x39;
/// cpuss DataWire #1, Channel #5 (dec: 58)
pub const CPUSS_INTERRUPTS_DW1_5: u32 = 0x3A;
/// cpuss DataWire #1, Channel #6 (dec: 59)
pub const CPUSS_INTERRUPTS_DW1_6: u32 = 0x3B;
/// cpuss DataWire #1, Channel #7 (dec: 60)
pub const CPUSS_INTERRUPTS_DW1_7: u32 = 0x3C;
/// cpuss DataWire #1, Channel #8 (dec: 61)
pub const CPUSS_INTERRUPTS_DW1_8: u32 = 0x3D;
/// cpuss DataWire #1, Channel #9 (dec: 62)
pub const CPUSS_INTERRUPTS_DW1_9: u32 = 0x3E;
/// cpuss DataWire #1, Channel #10 (dec: 63)
pub const CPUSS_INTERRUPTS_DW1_10: u32 = 0x3F;
/// cpuss DataWire #1, Channel #11 (dec: 64)
pub const CPUSS_INTERRUPTS_DW1_11: u32 = 0x40;
/// cpuss DataWire #1, Channel #12 (dec: 65)
pub const CPUSS_INTERRUPTS_DW1_12: u32 = 0x41;
/// cpuss DataWire #1, Channel #13 (dec: 66)
pub const CPUSS_INTERRUPTS_DW1_13: u32 = 0x42;
/// cpuss DataWire #1, Channel #14 (dec: 67)
pub const CPUSS_INTERRUPTS_DW1_14: u32 = 0x43;
/// cpuss DataWire #1, Channel #15 (dec: 68)
pub const CPUSS_INTERRUPTS_DW1_15: u32 = 0x44;
/// PPU SRAM0 (dec: 69)
pub const CPUSS_INTERRUPT_PPU_SRAMC0: u32 = 0x45;
/// CM33 0 Floating Point Interrupt (dec: 70)
pub const CPUSS_INTERRUPT_CM33_0_FP: u32 = 0x46;
/// CM33-0 CTI interrupt outputs (dec: 71)
pub const CPUSS_INTERRUPTS_CM33_0_CTI_0: u32 = 0x47;
/// CM33-1 CTI interrupt outputs (dec: 72)
pub const CPUSS_INTERRUPTS_CM33_0_CTI_1: u32 = 0x48;
/// cpuss Faults interrupt (dec: 73)
pub const CPUSS_INTERRUPTS_FAULT_0: u32 = 0x49;
/// cpuss PPU Interrupt (dec: 74)
pub const CPUSS_INTERRUPT_PPU_CPUSS: u32 = 0x4A;
/// cpuss Master Security Controller Interrupt (dec: 75)
pub const CPUSS_INTERRUPT_MSC: u32 = 0x4B;
/// TCPWM #0, Counter #0 (dec: 76)
pub const TCPWM_0_INTERRUPTS_0: u32 = 0x4C;
/// TCPWM #0, Counter #1 (dec: 77)
pub const TCPWM_0_INTERRUPTS_1: u32 = 0x4D;
/// TCPWM #0, Counter #2 (dec: 78)
pub const TCPWM_0_INTERRUPTS_2: u32 = 0x4E;
/// TCPWM #0, Counter #3 (dec: 79)
pub const TCPWM_0_INTERRUPTS_3: u32 = 0x4F;
/// TCPWM #0, Counter #256 (dec: 80)
pub const TCPWM_0_INTERRUPTS_256: u32 = 0x50;
/// TCPWM #0, Counter #257 (dec: 81)
pub const TCPWM_0_INTERRUPTS_257: u32 = 0x51;
/// TCPWM #0, Counter #258 (dec: 82)
pub const TCPWM_0_INTERRUPTS_258: u32 = 0x52;
/// TCPWM #0, Counter #259 (dec: 83)
pub const TCPWM_0_INTERRUPTS_259: u32 = 0x53;
/// TCPWM #0, Counter #260 (dec: 84)
pub const TCPWM_0_INTERRUPTS_260: u32 = 0x54;
/// TCPWM #0, Counter #261 (dec: 85)
pub const TCPWM_0_INTERRUPTS_261: u32 = 0x55;
/// TCPWM #0, Counter #262 (dec: 86)
pub const TCPWM_0_INTERRUPTS_262: u32 = 0x56;
/// TCPWM #0, Counter #263 (dec: 87)
pub const TCPWM_0_INTERRUPTS_263: u32 = 0x57;
/// TCPWM #0, Counter #512 (dec: 88)
pub const TCPWM_0_INTERRUPTS_512: u32 = 0x58;
/// TCPWM #0, Counter #513 (dec: 89)
pub const TCPWM_0_INTERRUPTS_513: u32 = 0x59;
/// TCPWM #0, Counter #514 (dec: 90)
pub const TCPWM_0_INTERRUPTS_514: u32 = 0x5A;
/// TCPWM #0, Counter #515 (dec: 91)
pub const TCPWM_0_INTERRUPTS_515: u32 = 0x5B;
/// TCPWM #0, Counter #516 (dec: 92)
pub const TCPWM_0_INTERRUPTS_516: u32 = 0x5C;
/// TCPWM #0, Counter #517 (dec: 93)
pub const TCPWM_0_INTERRUPTS_517: u32 = 0x5D;
/// TCPWM #0, Counter #518 (dec: 94)
pub const TCPWM_0_INTERRUPTS_518: u32 = 0x5E;
/// TCPWM #0, Counter #519 (dec: 95)
pub const TCPWM_0_INTERRUPTS_519: u32 = 0x5F;
/// SRSS Main PPU Interrupt (dec: 96)
pub const SRSS_INTERRUPT_MAIN_PPU: u32 = 0x60;
/// Crypto Interrupt (dec: 97)
pub const CRYPTOLITE_INTERRUPT_ERROR: u32 = 0x61;
/// TRNG interrupt (dec: 98)
pub const CRYPTOLITE_INTERRUPT_TRNG: u32 = 0x62;
/// CAN #0, Interrupt #0, Channel #0 (dec: 99)
pub const CANFD_0_INTERRUPTS0_0: u32 = 0x63;
/// CAN #0, Interrupt #1, Channel #0 (dec: 100)
pub const CANFD_0_INTERRUPTS1_0: u32 = 0x64;
/// CAN #0, Interrupt #0, Channel #1 (dec: 101)
pub const CANFD_0_INTERRUPTS0_1: u32 = 0x65;
/// CAN #0, Interrupt #1, Channel #1 (dec: 102)
pub const CANFD_0_INTERRUPTS1_1: u32 = 0x66;
/// Can #0, Consolidated interrupt #0 (dec: 103)
pub const CANFD_0_INTERRUPT0: u32 = 0x67;
/// Can #0, Consolidated interrupt #1 (dec: 104)
pub const CANFD_0_INTERRUPT1: u32 = 0x68;
/// CORDIC (dec: 105)
pub const CORDIC_INTERRUPT_MXCORDIC: u32 = 0x69;
/// TCPWM #0, MOTIF #1 SR0 (dec: 106)
pub const TCPWM_0_MOTIF_INTERRUPT_64: u32 = 0x6A;
/// TCPWM #0, MOTIF #1 SR1 (dec: 107)
pub const TCPWM_0_MOTIF_INTERRUPT_65: u32 = 0x6B;
/// Combined MCPASS interrupt, AC, Error conditions (dec: 108)
pub const PASS_INTERRUPT_MCPASS: u32 = 0x6C;
/// Combined SAR Entry and FIR results interrupt (dec: 109)
pub const PASS_INTERRUPT_SAR_RESULTS: u32 = 0x6D;
/// Individual SAR Entry result interrupts [0] (dec: 110)
pub const PASS_INTERRUPT_SAR_ENTRY_DONE_0: u32 = 0x6E;
/// Individual SAR Entry result interrupts [1] (dec: 111)
pub const PASS_INTERRUPT_SAR_ENTRY_DONE_1: u32 = 0x6F;
/// Individual SAR Entry result interrupts [2] (dec: 112)
pub const PASS_INTERRUPT_SAR_ENTRY_DONE_2: u32 = 0x70;
/// Individual SAR Entry result interrupts [3] (dec: 113)
pub const PASS_INTERRUPT_SAR_ENTRY_DONE_3: u32 = 0x71;
/// Individual SAR Entry result interrupts [4] (dec: 114)
pub const PASS_INTERRUPT_SAR_ENTRY_DONE_4: u32 = 0x72;
/// Individual SAR Entry result interrupts [5] (dec: 115)
pub const PASS_INTERRUPT_SAR_ENTRY_DONE_5: u32 = 0x73;
/// Individual SAR Entry result interrupts [6] (dec: 116)
pub const PASS_INTERRUPT_SAR_ENTRY_DONE_6: u32 = 0x74;
/// Individual SAR Entry result interrupts [7] (dec: 117)
pub const PASS_INTERRUPT_SAR_ENTRY_DONE_7: u32 = 0x75;
/// Individual FIR result interrupts[0] (dec: 118)
pub const PASS_INTERRUPT_SAR_FIR_DONE_0: u32 = 0x76;
/// Individual FIR result interrupts[1] (dec: 119)
pub const PASS_INTERRUPT_SAR_FIR_DONE_1: u32 = 0x77;
/// Combined SAR range interrupt (dec: 120)
pub const PASS_INTERRUPT_SAR_RANGES: u32 = 0x78;
/// Individual SAR range interrupts[0] (dec: 121)
pub const PASS_INTERRUPT_SAR_RANGE_0: u32 = 0x79;
/// Individual SAR range interrupts[1] (dec: 122)
pub const PASS_INTERRUPT_SAR_RANGE_1: u32 = 0x7A;
/// Individual SAR range interrupts[2] (dec: 123)
pub const PASS_INTERRUPT_SAR_RANGE_2: u32 = 0x7B;
/// Individual SAR range interrupts[3] (dec: 124)
pub const PASS_INTERRUPT_SAR_RANGE_3: u32 = 0x7C;
/// Individual SAR range interrupts[4] (dec: 125)
pub const PASS_INTERRUPT_SAR_RANGE_4: u32 = 0x7D;
/// Individual SAR range interrupts[5] (dec: 126)
pub const PASS_INTERRUPT_SAR_RANGE_5: u32 = 0x7E;
/// Individual SAR range interrupts[6] (dec: 127)
pub const PASS_INTERRUPT_SAR_RANGE_6: u32 = 0x7F;
/// Individual SAR range interrupts[7] (dec: 128)
pub const PASS_INTERRUPT_SAR_RANGE_7: u32 = 0x80;
/// Combined CSG DAC interrupt (dec: 129)
pub const PASS_INTERRUPT_CSG_DACS: u32 = 0x81;
/// Individual CSG DAC interrupts ( 5 in PSOC C3 )[0] (dec: 130)
pub const PASS_INTERRUPT_CSG_DAC_0: u32 = 0x82;
/// Individual CSG DAC interrupts ( 5 in PSOC C3 )[1] (dec: 131)
pub const PASS_INTERRUPT_CSG_DAC_1: u32 = 0x83;
/// Individual CSG DAC interrupts ( 5 in PSOC C3 )[2] (dec: 132)
pub const PASS_INTERRUPT_CSG_DAC_2: u32 = 0x84;
/// Individual CSG DAC interrupts ( 5 in PSOC C3 )[3] (dec: 133)
pub const PASS_INTERRUPT_CSG_DAC_3: u32 = 0x85;
/// Individual CSG DAC interrupts ( 5 in PSOC C3 )[4] (dec: 134)
pub const PASS_INTERRUPT_CSG_DAC_4: u32 = 0x86;
/// Individual CSG DAC interrupts ( 5 in PSOC C3 )[5] (dec: 135)
pub const PASS_INTERRUPT_CSG_DAC_5: u32 = 0x87;
/// Individual CSG DAC interrupts ( 5 in PSOC C3 )[6] (dec: 136)
pub const PASS_INTERRUPT_CSG_DAC_6: u32 = 0x88;
/// Individual CSG DAC interrupts ( 5 in PSOC C3 )[7] (dec: 137)
pub const PASS_INTERRUPT_CSG_DAC_7: u32 = 0x89;
/// Combined CSG CMP interrupts (dec: 138)
pub const PASS_INTERRUPT_CSG_CMPS: u32 = 0x8A;
/// Combined FIFO interrupts (dec: 139)
pub const PASS_INTERRUPT_FIFOS: u32 = 0x8B;
