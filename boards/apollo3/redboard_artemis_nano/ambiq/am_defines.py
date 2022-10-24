#!/usr/bin/env python3
# Utility functioins

import sys
from Crypto.Cipher import AES
from Crypto.PublicKey import RSA 
from Crypto.Signature import PKCS1_v1_5 
from Crypto.Hash import SHA256 
import array
import hashlib
import hmac
import os
import binascii


ivVal0 = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]

FLASH_PAGE_SIZE             = 0x2000                # 8K
MAX_DOWNLOAD_SIZE           = 0x48000               # 288K
AM_SECBOOT_DEFAULT_NONSECURE_MAIN   = 0xC000

AM_SECBOOT_AESCBC_BLOCK_SIZE_WORDS  = 4
AM_SECBOOT_AESCBC_BLOCK_SIZE_BYTES  = 4*AM_SECBOOT_AESCBC_BLOCK_SIZE_WORDS

AM_SECBOOT_MIN_KEYIDX_INFO0    = 8 ## KeyIdx 8 - 15
AM_SECBOOT_MAX_KEYIDX_INFO0    = 15
AM_SECBOOT_MIN_KEYIDX_INFO1    = 0 ## KeyIdx 0 - 7
AM_SECBOOT_MAX_KEYIDX_INFO1    = 7
AM_SECBOOT_KEYIDX_BYTES             = 16

# Encryption Algorithm
AM_SECBOOT_ENC_ALGO_NONE = 0
AM_SECBOOT_ENC_ALGO_AES128 = 1
AM_SECBOOT_ENC_ALGO_MAX  = AM_SECBOOT_ENC_ALGO_AES128
# String constants
helpEncAlgo = 'Encryption Algo? (0(default) = none, 1 = AES128)'

# Authentication Algorithm
AM_SECBOOT_AUTH_ALGO_NONE = 0
AM_SECBOOT_AUTH_ALGO_SHA256HMAC = 1
AM_SECBOOT_AUTH_ALGO_MAX = AM_SECBOOT_AUTH_ALGO_SHA256HMAC
# String constants
helpAuthAlgo = 'Authentication Algo? (0(default) = none, 1 = SHA256)'


FLASH_INVALID               = 0xFFFFFFFF

# KeyWrap Mode
AM_SECBOOT_KEYWRAP_NONE     = 0
AM_SECBOOT_KEYWRAP_XOR      = 1
AM_SECBOOT_KEYWRAP_AES128   = 2
AM_SECBOOT_KEYWRAP_MAX      = AM_SECBOOT_KEYWRAP_AES128

#******************************************************************************
#
# Magic Numbers
#
#******************************************************************************
AM_IMAGE_MAGIC_MAIN       = 0xC0
AM_IMAGE_MAGIC_CHILD      = 0xCC
AM_IMAGE_MAGIC_NONSECURE  = 0xCB
AM_IMAGE_MAGIC_INFO0      = 0xCF

# Dummy for creating images for customer - not understood by SBL
# This could be any value from the definition:
# #define AM_IMAGE_MAGIC_CUST(x)   ((((x) & 0xF0) == 0xC0) && ((x) != 0xC0) && ((x) != 0xCC) && ((x) != 0xCB) && ((x) != 0xCF))
AM_IMAGE_MAGIC_CUSTPATCH  = 0xC1

#******************************************************************************
#
# Image Types
#
#******************************************************************************
AM_SECBOOT_WIRED_IMAGETYPE_SBL                  = 0
AM_SECBOOT_WIRED_IMAGETYPE_AM3P                 = 1
AM_SECBOOT_WIRED_IMAGETYPE_PATCH                = 2
AM_SECBOOT_WIRED_IMAGETYPE_MAIN                 = 3
AM_SECBOOT_WIRED_IMAGETYPE_CHILD                = 4
AM_SECBOOT_WIRED_IMAGETYPE_CUSTPATCH            = 5
AM_SECBOOT_WIRED_IMAGETYPE_NONSECURE            = 6
AM_SECBOOT_WIRED_IMAGETYPE_INFO0                = 7
AM_SECBOOT_WIRED_IMAGETYPE_INFO0_NOOTA          = 32
AM_SECBOOT_WIRED_IMAGETYPE_INVALID              = 0xFF


#******************************************************************************
#
# Wired Message Types
#
#******************************************************************************
AM_SECBOOT_WIRED_MSGTYPE_HELLO          = 0
AM_SECBOOT_WIRED_MSGTYPE_STATUS         = 1
AM_SECBOOT_WIRED_MSGTYPE_OTADESC        = 2
AM_SECBOOT_WIRED_MSGTYPE_UPDATE         = 3
AM_SECBOOT_WIRED_MSGTYPE_ABORT          = 4
AM_SECBOOT_WIRED_MSGTYPE_RECOVER        = 5
AM_SECBOOT_WIRED_MSGTYPE_RESET          = 6
AM_SECBOOT_WIRED_MSGTYPE_ACK            = 7
AM_SECBOOT_WIRED_MSGTYPE_DATA           = 8


#******************************************************************************
#
# Wired Message ACK Status
#
#******************************************************************************
AM_SECBOOT_WIRED_ACK_STATUS_SUCCESS              = 0
AM_SECBOOT_WIRED_ACK_STATUS_FAILURE              = 1
AM_SECBOOT_WIRED_ACK_STATUS_INVALID_INFO0        = 2
AM_SECBOOT_WIRED_ACK_STATUS_CRC                  = 3
AM_SECBOOT_WIRED_ACK_STATUS_SEC                  = 4
AM_SECBOOT_WIRED_ACK_STATUS_MSG_TOO_BIG          = 5
AM_SECBOOT_WIRED_ACK_STATUS_UNKNOWN_MSGTYPE      = 6
AM_SECBOOT_WIRED_ACK_STATUS_INVALID_ADDR         = 7
AM_SECBOOT_WIRED_ACK_STATUS_INVALID_OPERATION    = 8
AM_SECBOOT_WIRED_ACK_STATUS_INVALID_PARAM        = 9
AM_SECBOOT_WIRED_ACK_STATUS_SEQ                  = 10
AM_SECBOOT_WIRED_ACK_STATUS_TOO_MUCH_DATA        = 11

#******************************************************************************
#
# Definitions related to Image Headers
#
#******************************************************************************
AM_HMAC_SIG_SIZE                = 32
AM_KEK_SIZE                     = 16
AM_CRC_SIZE                     = 4

AM_MAX_UART_MSG_SIZE            = 8192  # 8K buffer in SBL

# Wiredupdate Image Header
AM_WU_IMAGEHDR_OFFSET_SIG       = 16
AM_WU_IMAGEHDR_OFFSET_IV        = 48
AM_WU_IMAGEHDR_OFFSET_KEK       = 64
AM_WU_IMAGEHDR_OFFSET_IMAGETYPE = (AM_WU_IMAGEHDR_OFFSET_KEK + AM_KEK_SIZE)
AM_WU_IMAGEHDR_OFFSET_OPTIONS   = (AM_WU_IMAGEHDR_OFFSET_IMAGETYPE + 1)
AM_WU_IMAGEHDR_OFFSET_KEY       = (AM_WU_IMAGEHDR_OFFSET_IMAGETYPE + 4)
AM_WU_IMAGEHDR_OFFSET_ADDR      = (AM_WU_IMAGEHDR_OFFSET_KEY + 4)
AM_WU_IMAGEHDR_OFFSET_SIZE      = (AM_WU_IMAGEHDR_OFFSET_ADDR + 4)

AM_WU_IMAGEHDR_START_HMAC       = (AM_WU_IMAGEHDR_OFFSET_SIG + AM_HMAC_SIG_SIZE)
AM_WU_IMAGEHDR_START_ENCRYPT    = (AM_WU_IMAGEHDR_OFFSET_KEK + AM_KEK_SIZE)
AM_WU_IMAGEHDR_SIZE             = (AM_WU_IMAGEHDR_OFFSET_KEK + AM_KEK_SIZE + 16)


# Image Header
AM_IMAGEHDR_SIZE_MAIN           = 256
AM_IMAGEHDR_SIZE_AUX            = (112 + AM_KEK_SIZE)

AM_IMAGEHDR_OFFSET_CRC          = 4
AM_IMAGEHDR_OFFSET_SIG          = 16
AM_IMAGEHDR_OFFSET_IV           = 48
AM_IMAGEHDR_OFFSET_KEK          = 64
AM_IMAGEHDR_OFFSET_SIGCLR       = (AM_IMAGEHDR_OFFSET_KEK + AM_KEK_SIZE)
AM_IMAGEHDR_START_CRC           = (AM_IMAGEHDR_OFFSET_CRC + AM_CRC_SIZE)
AM_IMAGEHDR_START_HMAC_INST     = (AM_IMAGEHDR_OFFSET_SIG + AM_HMAC_SIG_SIZE)
AM_IMAGEHDR_START_ENCRYPT       = (AM_IMAGEHDR_OFFSET_KEK + AM_KEK_SIZE)
AM_IMAGEHDR_START_HMAC          = (AM_IMAGEHDR_OFFSET_SIGCLR + AM_HMAC_SIG_SIZE)
AM_IMAGEHDR_OFFSET_ADDR         = AM_IMAGEHDR_START_HMAC
AM_IMAGEHDR_OFFSET_VERKEY       = (AM_IMAGEHDR_OFFSET_ADDR + 4)
AM_IMAGEHDR_OFFSET_CHILDPTR     = (AM_IMAGEHDR_OFFSET_VERKEY + 4)

# Recover message
AM_WU_RECOVERY_HDR_SIZE             = 44
AM_WU_RECOVERY_HDR_OFFSET_CUSTID    = 8
AM_WU_RECOVERY_HDR_OFFSET_RECKEY    = (AM_WU_RECOVERY_HDR_OFFSET_CUSTID + 4)
AM_WU_RECOVERY_HDR_OFFSET_NONCE     = (AM_WU_RECOVERY_HDR_OFFSET_RECKEY + 16)
AM_WU_RECOVERY_HDR_OFFSET_RECBLOB   = (AM_WU_RECOVERY_HDR_OFFSET_NONCE + 16)


#******************************************************************************
#
# INFOSPACE related definitions
#
#******************************************************************************
AM_SECBOOT_INFO0_SIGN_PROGRAMMED0   = 0x48EAAD88
AM_SECBOOT_INFO0_SIGN_PROGRAMMED1   = 0xC9705737
AM_SECBOOT_INFO0_SIGN_PROGRAMMED2   = 0x0A6B8458
AM_SECBOOT_INFO0_SIGN_PROGRAMMED3   = 0xE41A9D74

AM_SECBOOT_INFO0_SIGN_UINIT0        = 0x5B75A5FA
AM_SECBOOT_INFO0_SIGN_UINIT1        = 0x7B9C8674
AM_SECBOOT_INFO0_SIGN_UINIT2        = 0x869A96FE
AM_SECBOOT_INFO0_SIGN_UINIT3        = 0xAEC90860

INFO_SIZE_BYTES                     = (8 * 1024)
INFO_MAX_AUTH_KEY_WORDS             = 32
INFO_MAX_ENC_KEY_WORDS              = 32

INFO_MAX_AUTH_KEYS   = (INFO_MAX_AUTH_KEY_WORDS*4//AM_SECBOOT_KEYIDX_BYTES)
INFO_MAX_ENC_KEYS    = (INFO_MAX_ENC_KEY_WORDS*4//AM_SECBOOT_KEYIDX_BYTES)

INFO0_SIGNATURE0_O 	= 0x00000000
INFO0_SIGNATURE1_O 	= 0x00000004
INFO0_SIGNATURE2_O 	= 0x00000008
INFO0_SIGNATURE3_O 	= 0x0000000c
INFO0_SECURITY_O 	= 0x00000010
INFO0_CUSTOMER_TRIM_O 	= 0x00000014
INFO0_CUSTOMER_TRIM2_O 	= 0x00000018
INFO0_SECURITY_OVR_O 	= 0x00000020
INFO0_SECURITY_WIRED_CFG_O 	= 0x00000024
INFO0_SECURITY_WIRED_IFC_CFG0_O 	= 0x00000028
INFO0_SECURITY_WIRED_IFC_CFG1_O 	= 0x0000002C
INFO0_SECURITY_WIRED_IFC_CFG2_O 	= 0x00000030
INFO0_SECURITY_WIRED_IFC_CFG3_O 	= 0x00000034
INFO0_SECURITY_WIRED_IFC_CFG4_O 	= 0x00000038
INFO0_SECURITY_WIRED_IFC_CFG5_O 	= 0x0000003C
INFO0_SECURITY_VERSION_O 	= 0x00000040
INFO0_SECURITY_SRAM_RESV_O 	= 0x00000050
AM_REG_INFO0_SECURITY_SRAM_RESV_SRAM_RESV_M = 0x0000FFFF
INFO0_WRITE_PROTECT_L_O 	= 0x000001f8
INFO0_WRITE_PROTECT_H_O 	= 0x000001fc
INFO0_COPY_PROTECT_L_O 	= 0x00000200
INFO0_COPY_PROTECT_H_O 	= 0x00000204
INFO0_WRITE_PROTECT_SBL_L_O     = 0x000009f8
INFO0_WRITE_PROTECT_SBL_H_O     = 0x000009fc
INFO0_COPY_PROTECT_SBL_L_O  = 0x00000A00
INFO0_COPY_PROTECT_SBL_H_O  = 0x00000A04
INFO0_MAIN_PTR1_O   = 0x00000C00
INFO0_MAIN_PTR2_O   = 0x00000C04
INFO0_KREVTRACK_O   = 0x00000C08
INFO0_AREVTRACK_O   = 0x00000C0C
INFO0_MAIN_CNT0_O   = 0x00000FF8
INFO0_MAIN_CNT1_O   = 0x00000FFC

INFO0_CUST_KEK_W0_O 	= 0x00001800
INFO0_CUST_KEK_W1_O 	= 0x00001804
INFO0_CUST_KEK_W2_O 	= 0x00001808
INFO0_CUST_KEK_W3_O 	= 0x0000180c
INFO0_CUST_KEK_W4_O 	= 0x00001810
INFO0_CUST_KEK_W5_O 	= 0x00001814
INFO0_CUST_KEK_W6_O 	= 0x00001818
INFO0_CUST_KEK_W7_O 	= 0x0000181c
INFO0_CUST_KEK_W8_O 	= 0x00001820
INFO0_CUST_KEK_W9_O 	= 0x00001824
INFO0_CUST_KEK_W10_O 	= 0x00001828
INFO0_CUST_KEK_W11_O 	= 0x0000182c
INFO0_CUST_KEK_W12_O 	= 0x00001830
INFO0_CUST_KEK_W13_O 	= 0x00001834
INFO0_CUST_KEK_W14_O 	= 0x00001838
INFO0_CUST_KEK_W15_O 	= 0x0000183c
INFO0_CUST_KEK_W16_O 	= 0x00001840
INFO0_CUST_KEK_W17_O 	= 0x00001844
INFO0_CUST_KEK_W18_O 	= 0x00001848
INFO0_CUST_KEK_W19_O 	= 0x0000184c
INFO0_CUST_KEK_W20_O 	= 0x00001850
INFO0_CUST_KEK_W21_O 	= 0x00001854
INFO0_CUST_KEK_W22_O 	= 0x00001858
INFO0_CUST_KEK_W23_O 	= 0x0000185c
INFO0_CUST_KEK_W24_O 	= 0x00001860
INFO0_CUST_KEK_W25_O 	= 0x00001864
INFO0_CUST_KEK_W26_O 	= 0x00001868
INFO0_CUST_KEK_W27_O 	= 0x0000186c
INFO0_CUST_KEK_W28_O 	= 0x00001870
INFO0_CUST_KEK_W29_O 	= 0x00001874
INFO0_CUST_KEK_W30_O 	= 0x00001878
INFO0_CUST_KEK_W31_O 	= 0x0000187c
INFO0_CUST_AUTH_W0_O 	= 0x00001880
INFO0_CUST_AUTH_W1_O 	= 0x00001884
INFO0_CUST_AUTH_W2_O 	= 0x00001888
INFO0_CUST_AUTH_W3_O 	= 0x0000188c
INFO0_CUST_AUTH_W4_O 	= 0x00001890
INFO0_CUST_AUTH_W5_O 	= 0x00001894
INFO0_CUST_AUTH_W6_O 	= 0x00001898
INFO0_CUST_AUTH_W7_O 	= 0x0000189c
INFO0_CUST_AUTH_W8_O 	= 0x000018a0
INFO0_CUST_AUTH_W9_O 	= 0x000018a4
INFO0_CUST_AUTH_W10_O 	= 0x000018a8
INFO0_CUST_AUTH_W11_O 	= 0x000018ac
INFO0_CUST_AUTH_W12_O 	= 0x000018b0
INFO0_CUST_AUTH_W13_O 	= 0x000018b4
INFO0_CUST_AUTH_W14_O 	= 0x000018b8
INFO0_CUST_AUTH_W15_O 	= 0x000018bc
INFO0_CUST_AUTH_W16_O 	= 0x000018c0
INFO0_CUST_AUTH_W17_O 	= 0x000018c4
INFO0_CUST_AUTH_W18_O 	= 0x000018c8
INFO0_CUST_AUTH_W19_O 	= 0x000018cc
INFO0_CUST_AUTH_W20_O 	= 0x000018d0
INFO0_CUST_AUTH_W21_O 	= 0x000018d4
INFO0_CUST_AUTH_W22_O 	= 0x000018d8
INFO0_CUST_AUTH_W23_O 	= 0x000018dc
INFO0_CUST_AUTH_W24_O 	= 0x000018e0
INFO0_CUST_AUTH_W25_O 	= 0x000018e4
INFO0_CUST_AUTH_W26_O 	= 0x000018e8
INFO0_CUST_AUTH_W27_O 	= 0x000018ec
INFO0_CUST_AUTH_W28_O 	= 0x000018f0
INFO0_CUST_AUTH_W29_O 	= 0x000018f4
INFO0_CUST_AUTH_W30_O 	= 0x000018f8
INFO0_CUST_AUTH_W31_O 	= 0x000018fc
INFO0_CUST_PUBKEY_W0_O 	= 0x00001900
INFO0_CUST_PUBKEY_W1_O 	= 0x00001904
INFO0_CUST_PUBKEY_W2_O 	= 0x00001908
INFO0_CUST_PUBKEY_W3_O 	= 0x0000190c
INFO0_CUST_PUBKEY_W4_O 	= 0x00001910
INFO0_CUST_PUBKEY_W5_O 	= 0x00001914
INFO0_CUST_PUBKEY_W6_O 	= 0x00001918
INFO0_CUST_PUBKEY_W7_O 	= 0x0000191c
INFO0_CUST_PUBKEY_W8_O 	= 0x00001920
INFO0_CUST_PUBKEY_W9_O 	= 0x00001924
INFO0_CUST_PUBKEY_W10_O 	= 0x00001928
INFO0_CUST_PUBKEY_W11_O 	= 0x0000192c
INFO0_CUST_PUBKEY_W12_O 	= 0x00001930
INFO0_CUST_PUBKEY_W13_O 	= 0x00001934
INFO0_CUST_PUBKEY_W14_O 	= 0x00001938
INFO0_CUST_PUBKEY_W15_O 	= 0x0000193c
INFO0_CUST_PUBKEY_W16_O 	= 0x00001940
INFO0_CUST_PUBKEY_W17_O 	= 0x00001944
INFO0_CUST_PUBKEY_W18_O 	= 0x00001948
INFO0_CUST_PUBKEY_W19_O 	= 0x0000194c
INFO0_CUST_PUBKEY_W20_O 	= 0x00001950
INFO0_CUST_PUBKEY_W21_O 	= 0x00001954
INFO0_CUST_PUBKEY_W22_O 	= 0x00001958
INFO0_CUST_PUBKEY_W23_O 	= 0x0000195c
INFO0_CUST_PUBKEY_W24_O 	= 0x00001960
INFO0_CUST_PUBKEY_W25_O 	= 0x00001964
INFO0_CUST_PUBKEY_W26_O 	= 0x00001968
INFO0_CUST_PUBKEY_W27_O 	= 0x0000196c
INFO0_CUST_PUBKEY_W28_O 	= 0x00001970
INFO0_CUST_PUBKEY_W29_O 	= 0x00001974
INFO0_CUST_PUBKEY_W30_O 	= 0x00001978
INFO0_CUST_PUBKEY_W31_O 	= 0x0000197c
INFO0_CUST_PUBKEY_W32_O 	= 0x00001980
INFO0_CUST_PUBKEY_W33_O 	= 0x00001984
INFO0_CUST_PUBKEY_W34_O 	= 0x00001988
INFO0_CUST_PUBKEY_W35_O 	= 0x0000198c
INFO0_CUST_PUBKEY_W36_O 	= 0x00001990
INFO0_CUST_PUBKEY_W37_O 	= 0x00001994
INFO0_CUST_PUBKEY_W38_O 	= 0x00001998
INFO0_CUST_PUBKEY_W39_O 	= 0x0000199c
INFO0_CUST_PUBKEY_W40_O 	= 0x000019a0
INFO0_CUST_PUBKEY_W41_O 	= 0x000019a4
INFO0_CUST_PUBKEY_W42_O 	= 0x000019a8
INFO0_CUST_PUBKEY_W43_O 	= 0x000019ac
INFO0_CUST_PUBKEY_W44_O 	= 0x000019b0
INFO0_CUST_PUBKEY_W45_O 	= 0x000019b4
INFO0_CUST_PUBKEY_W46_O 	= 0x000019b8
INFO0_CUST_PUBKEY_W47_O 	= 0x000019bc
INFO0_CUST_PUBKEY_W48_O 	= 0x000019c0
INFO0_CUST_PUBKEY_W49_O 	= 0x000019c4
INFO0_CUST_PUBKEY_W50_O 	= 0x000019c8
INFO0_CUST_PUBKEY_W51_O 	= 0x000019cc
INFO0_CUST_PUBKEY_W52_O 	= 0x000019d0
INFO0_CUST_PUBKEY_W53_O 	= 0x000019d4
INFO0_CUST_PUBKEY_W54_O 	= 0x000019d8
INFO0_CUST_PUBKEY_W55_O 	= 0x000019dc
INFO0_CUST_PUBKEY_W56_O 	= 0x000019e0
INFO0_CUST_PUBKEY_W57_O 	= 0x000019e4
INFO0_CUST_PUBKEY_W58_O 	= 0x000019e8
INFO0_CUST_PUBKEY_W59_O 	= 0x000019ec
INFO0_CUST_PUBKEY_W60_O 	= 0x000019f0
INFO0_CUST_PUBKEY_W61_O 	= 0x000019f4
INFO0_CUST_PUBKEY_W62_O 	= 0x000019f8
INFO0_CUST_PUBKEY_W63_O 	= 0x000019fc
INFO0_CUSTOMER_KEY0_O 	= 0x00001a00
INFO0_CUSTOMER_KEY1_O 	= 0x00001a04
INFO0_CUSTOMER_KEY2_O 	= 0x00001a08
INFO0_CUSTOMER_KEY3_O 	= 0x00001a0c
INFO0_CUST_PUBHASH_W0_O 	= 0x00001a10
INFO0_CUST_PUBHASH_W1_O 	= 0x00001a14
INFO0_CUST_PUBHASH_W2_O 	= 0x00001a18
INFO0_CUST_PUBHASH_W3_O 	= 0x00001a1c


#******************************************************************************
#
# CRC using ethernet poly, as used by Corvette hardware for validation
#
#******************************************************************************
def crc32(L):
    return (binascii.crc32(L) & 0xFFFFFFFF)

#******************************************************************************
#
# Pad the text to the block_size. bZeroPad determines how to handle text which 
# is already multiple of block_size
#
#******************************************************************************
def pad_to_block_size(text, block_size, bZeroPad):
    text_length = len(text)
    amount_to_pad = block_size - (text_length % block_size)
    if (amount_to_pad == block_size):
        if (bZeroPad == 0):
            amount_to_pad = 0
    for i in range(0, amount_to_pad, 1):
        text += bytes(chr(amount_to_pad), 'ascii')
    return text


#******************************************************************************
#
# AES CBC encryption
#
#******************************************************************************
def encrypt_app_aes(cleartext, encKey, iv):
    key = array.array('B', encKey).tostring()
    ivVal = array.array('B', iv).tostring()
    plaintext = array.array('B', cleartext).tostring()

    encryption_suite = AES.new(key, AES.MODE_CBC, ivVal)
    cipher_text = encryption_suite.encrypt(plaintext)
    
    return cipher_text

#******************************************************************************
#
# AES 128 CBC encryption
#
#******************************************************************************
def encrypt_app_aes128(cleartext, encKey, iv):
    key = array.array('B', encKey).tostring()
    ivVal = array.array('B', iv).tostring()
    plaintext = array.array('B', cleartext).tostring()

    encryption_suite = AES.new(key, AES.MODE_CBC, ivVal)
    cipher_text = encryption_suite.encrypt(plaintext)
    
    return cipher_text
    
#******************************************************************************
#
# SHA256 HMAC
#
#******************************************************************************
def compute_hmac(key, data):
    sig = hmac.new(array.array('B', key).tostring(), array.array('B', data).tostring(), hashlib.sha256).digest()
    return sig

#******************************************************************************
#
# RSA PKCS1_v1_5 sign
#
#******************************************************************************
def compute_rsa_sign(prvKeyFile, data):
    key = open(prvKeyFile, "r").read() 
    rsakey = RSA.importKey(key) 
    signer = PKCS1_v1_5.new(rsakey) 
    digest = SHA256.new() 
    digest.update(bytes(data)) 
    sign = signer.sign(digest) 
    return sign

#******************************************************************************
#
# RSA PKCS1_v1_5 sign verification
#
#******************************************************************************
def verify_rsa_sign(pubKeyFile, data, sign):
    key = open(pubKeyFile, "r").read() 
    rsakey = RSA.importKey(key) 
    #print(hex(rsakey.n))
    verifier = PKCS1_v1_5.new(rsakey)
    digest = SHA256.new() 
    digest.update(bytes(data)) 
    return verifier.verify(digest, sign)

#******************************************************************************
#
# Fill one word in bytearray
#
#******************************************************************************
def fill_word(barray, offset, w):
    barray[offset + 0]  = (w >>  0) & 0x000000ff;
    barray[offset + 1]  = (w >>  8) & 0x000000ff;
    barray[offset + 2]  = (w >> 16) & 0x000000ff;
    barray[offset + 3]  = (w >> 24) & 0x000000ff;


#******************************************************************************
#
# Turn a 32-bit number into a series of bytes for transmission.
#
# This command will split a 32-bit integer into an array of bytes, ordered
# LSB-first for transmission over the UART.
#
#******************************************************************************
def int_to_bytes(n):
    A = [n & 0xFF,
         (n >> 8) & 0xFF,
         (n >> 16) & 0xFF,
         (n >> 24) & 0xFF]

    return A

#******************************************************************************
#
# Extract a word from a byte array
#
#******************************************************************************
def word_from_bytes(B, n):
    return (B[n] + (B[n + 1] << 8) + (B[n + 2] << 16) + (B[n + 3] << 24))


#******************************************************************************
#
# automatically figure out the integer format (base 10 or 16)
#
#******************************************************************************
def auto_int(x):
    return int(x, 0)

#******************************************************************************
#
# User controllable Prints control
#
#******************************************************************************
# Defined print levels
AM_PRINT_LEVEL_MIN     = 0
AM_PRINT_LEVEL_NONE    = AM_PRINT_LEVEL_MIN
AM_PRINT_LEVEL_ERROR   = 1
AM_PRINT_LEVEL_INFO    = 2
AM_PRINT_LEVEL_VERBOSE = 4
AM_PRINT_LEVEL_DEBUG   = 5
AM_PRINT_LEVEL_MAX     = AM_PRINT_LEVEL_DEBUG

# Global variable to control the prints
AM_PRINT_VERBOSITY = AM_PRINT_LEVEL_INFO

helpPrintLevel = 'Set Log Level (0: None), (1: Error), (2: INFO), (4: Verbose), (5: Debug) [Default = Info]'

def am_set_print_level(level):
    global AM_PRINT_VERBOSITY
    AM_PRINT_VERBOSITY = level

def am_print(*args, level=AM_PRINT_LEVEL_INFO, **kwargs):
    global AM_PRINT_VERBOSITY
    if (AM_PRINT_VERBOSITY >= level):
        print(*args, **kwargs)
