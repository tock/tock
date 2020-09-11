#!/usr/bin/env python3
# Combination of the three steps to take an 'application.bin' file and run it on a SparkFun Artemis module

# Information:
#   This script performs the three main tasks:
#       1. Convert 'application.bin' to an OTA update blob
#       2. Convert the OTA blob into a wired update blob
#       3. Push the wired update blob into the Artemis module

import argparse
import sys
from Crypto.Cipher import AES
import array
import hashlib
import hmac
import os
import binascii
import serial
import serial.tools.list_ports as list_ports
import time
# from sf_am_defines import *
from sys import exit

from am_defines import *
from keys_info import keyTblAes, keyTblHmac, minAesKeyIdx, maxAesKeyIdx, minHmacKeyIdx, maxHmacKeyIdx, INFO_KEY, FLASH_KEY

#******************************************************************************
#
# Global Variables
#
#******************************************************************************
loadTries = 0 #If we fail, try again. Tracks the number of tries we've attempted
loadSuccess = False
blob2wiredfile = ''
uploadbinfile = ''


#******************************************************************************
#
# Generate the image blob as per command line parameters
#
#******************************************************************************
def bin2blob_process(loadaddress, appFile, magicNum, crcI, crcB, authI, authB, protection, authKeyIdx, output, encKeyIdx, version, erasePrev, child0, child1, authalgo, encalgo):

    global blob2wiredfile

    app_binarray = bytearray()
    # Open the file, and read it into an array of integers.
    with appFile as f_app:
        app_binarray.extend(f_app.read())
        f_app.close()

    encVal = 0
    if (encalgo != 0):
        encVal = 1
        if ((encKeyIdx < minAesKeyIdx) or (encKeyIdx > maxAesKeyIdx)):
            am_print("Invalid encKey Idx ", encKeyIdx, level=AM_PRINT_LEVEL_ERROR);
            return
        if (encalgo == 2):
            if (encKeyIdx & 0x1):
                am_print("Invalid encKey Idx ", encKeyIdx, level=AM_PRINT_LEVEL_ERROR);
                return
            keySize = 32
        else:
            keySize = 16
    if (authalgo != 0):
        if ((authKeyIdx < minHmacKeyIdx) or (authKeyIdx > maxHmacKeyIdx) or (authKeyIdx & 0x1)):
            am_print("Invalid authKey Idx ", authKeyIdx, level=AM_PRINT_LEVEL_ERROR);
            return

    if (magicNum == AM_IMAGE_MAGIC_MAIN):
        hdr_length  = AM_IMAGEHDR_SIZE_MAIN;   #fixed header length
    elif ((magicNum == AM_IMAGE_MAGIC_CHILD) or (magicNum == AM_IMAGE_MAGIC_CUSTPATCH) or (magicNum == AM_IMAGE_MAGIC_NONSECURE) or (magicNum == AM_IMAGE_MAGIC_INFO0)):
        hdr_length  = AM_IMAGEHDR_SIZE_AUX;   #fixed header length
    else:
        am_print("magic number", hex(magicNum), " not supported", level=AM_PRINT_LEVEL_ERROR)
        return
    am_print("Header Size = ", hex(hdr_length))

    #generate mutable byte array for the header
    hdr_binarray = bytearray([0x00]*hdr_length);

    orig_app_length  = (len(app_binarray))
    am_print("original app_size ",hex(orig_app_length), "(",orig_app_length,")")

    am_print("load_address ",hex(loadaddress), "(",loadaddress,")")
    if (loadaddress & 0x3):
        am_print("load address needs to be word aligned", level=AM_PRINT_LEVEL_ERROR)
        return

    if (magicNum == AM_IMAGE_MAGIC_INFO0):
        if (orig_app_length & 0x3):
            am_print("INFO0 blob length needs to be multiple of 4", level=AM_PRINT_LEVEL_ERROR)
            return
        if ((loadaddress + orig_app_length) > INFO_SIZE_BYTES):
            am_print("INFO0 Offset and length exceed size", level=AM_PRINT_LEVEL_ERROR)
            return

    if (encVal == 1):
        block_size = AM_SECBOOT_AESCBC_BLOCK_SIZE_BYTES
        app_binarray = pad_to_block_size(app_binarray, block_size, 1)
    else:
        # Add Padding
        app_binarray = pad_to_block_size(app_binarray, 4, 0)
    
    app_length  = (len(app_binarray))
    am_print("app_size ",hex(app_length), "(",app_length,")")

    # Create Image blobs

    # w0
    blobLen = hdr_length + app_length
    w0 = (magicNum << 24) | ((encVal & 0x1) << 23) | blobLen

    am_print("w0 =", hex(w0))
    fill_word(hdr_binarray, 0, w0)
        
    # w2
    securityVal = ((authI << 1) | crcI) << 4 | (authB << 1) | crcB
    am_print("Security Value ", hex(securityVal))
    w2 = ((securityVal << 24) & 0xff000000) | ((authalgo) & 0xf) | ((authKeyIdx << 4) & 0xf0) | ((encalgo << 8) & 0xf00) | ((encKeyIdx << 12) & 0xf000)
    fill_word(hdr_binarray, 8, w2)
    am_print("w2 = ",hex(w2))
    

    if (magicNum == AM_IMAGE_MAGIC_INFO0):
        # Insert the INFO0 size and offset
        addrWord = ((orig_app_length>>2) << 16) | ((loadaddress>>2) & 0xFFFF)
        versionKeyWord = INFO_KEY
    else:
        # Insert the application binary load address.
        addrWord = loadaddress | (protection & 0x3)
        # Initialize versionKeyWord
        versionKeyWord = (version & 0x7FFF) | ((erasePrev & 0x1) << 15)

    am_print("addrWord = ",hex(addrWord))
    fill_word(hdr_binarray, AM_IMAGEHDR_OFFSET_ADDR, addrWord)

    am_print("versionKeyWord = ",hex(versionKeyWord))
    fill_word(hdr_binarray, AM_IMAGEHDR_OFFSET_VERKEY, versionKeyWord)

    # Initialize child (Child Ptr/ Feature key)
    am_print("child0/feature = ",hex(child0))
    fill_word(hdr_binarray, AM_IMAGEHDR_OFFSET_CHILDPTR, child0)
    am_print("child1 = ",hex(child1))
    fill_word(hdr_binarray, AM_IMAGEHDR_OFFSET_CHILDPTR + 4, child1)

    authKeyIdx = authKeyIdx - minHmacKeyIdx
    if (authB != 0): # Authentication needed
        am_print("Boot Authentication Enabled")
#        am_print("Key used for HMAC")
#        am_print([hex(keyTblHmac[authKeyIdx*AM_SECBOOT_KEYIDX_BYTES + n]) for n in range (0, AM_HMAC_SIG_SIZE)])
        # Initialize the clear image HMAC
        sigClr = compute_hmac(keyTblHmac[authKeyIdx*AM_SECBOOT_KEYIDX_BYTES:(authKeyIdx*AM_SECBOOT_KEYIDX_BYTES+AM_HMAC_SIG_SIZE)], (hdr_binarray[AM_IMAGEHDR_START_HMAC:hdr_length] + app_binarray))
        am_print("HMAC Clear")
        am_print([hex(n) for n in sigClr])
        # Fill up the HMAC
        for x in range(0, AM_HMAC_SIG_SIZE):
            hdr_binarray[AM_IMAGEHDR_OFFSET_SIGCLR + x]  = sigClr[x]

    # All the header fields part of the encryption are now final
    if (encVal == 1):
        am_print("Encryption Enabled")
        encKeyIdx = encKeyIdx - minAesKeyIdx
        ivValAes = os.urandom(AM_SECBOOT_AESCBC_BLOCK_SIZE_BYTES)
        am_print("Initialization Vector")
        am_print([hex(ivValAes[n]) for n in range (0, AM_SECBOOT_AESCBC_BLOCK_SIZE_BYTES)])
        keyAes = os.urandom(keySize)
        am_print("AES Key used for encryption")
        am_print([hex(keyAes[n]) for n in range (0, keySize)])
        # Encrypted Part
        am_print("Encrypting blob of size " , (hdr_length - AM_IMAGEHDR_START_ENCRYPT + app_length))
        enc_binarray = encrypt_app_aes((hdr_binarray[AM_IMAGEHDR_START_ENCRYPT:hdr_length] + app_binarray), keyAes, ivValAes)
#        am_print("Key used for encrypting AES Key")
#        am_print([hex(keyTblAes[encKeyIdx*keySize + n]) for n in range (0, keySize)])
        # Encrypted Key
        enc_key = encrypt_app_aes(keyAes, keyTblAes[encKeyIdx*keySize:encKeyIdx*keySize + keySize], ivVal0)
        am_print("Encrypted Key")
        am_print([hex(enc_key[n]) for n in range (0, keySize)])
        # Fill up the IV
        for x in range(0, AM_SECBOOT_AESCBC_BLOCK_SIZE_BYTES):
            hdr_binarray[AM_IMAGEHDR_OFFSET_IV + x]  = ivValAes[x]
        # Fill up the Encrypted Key
        for x in range(0, keySize):
            hdr_binarray[AM_IMAGEHDR_OFFSET_KEK + x]  = enc_key[x]
    else:
        enc_binarray = hdr_binarray[AM_IMAGEHDR_START_ENCRYPT:hdr_length] + app_binarray


    if (authI != 0): # Install Authentication needed
        am_print("Install Authentication Enabled")
#        am_print("Key used for HMAC")
#        am_print([hex(keyTblHmac[authKeyIdx*AM_SECBOOT_KEYIDX_BYTES + n]) for n in range (0, AM_HMAC_SIG_SIZE)])
        # Initialize the top level HMAC
        sig = compute_hmac(keyTblHmac[authKeyIdx*AM_SECBOOT_KEYIDX_BYTES:(authKeyIdx*AM_SECBOOT_KEYIDX_BYTES+AM_HMAC_SIG_SIZE)], (hdr_binarray[AM_IMAGEHDR_START_HMAC_INST:AM_IMAGEHDR_START_ENCRYPT] + enc_binarray))
        am_print("Generated Signature")
        am_print([hex(n) for n in sig])
        # Fill up the HMAC
        for x in range(0, AM_HMAC_SIG_SIZE):
            hdr_binarray[AM_IMAGEHDR_OFFSET_SIG + x]  = sig[x]
    # compute the CRC for the blob - this is done on a clear image
    crc = crc32(hdr_binarray[AM_IMAGEHDR_START_CRC:hdr_length] + app_binarray)
    am_print("crc =  ",hex(crc));
    w1 = crc
    fill_word(hdr_binarray, AM_IMAGEHDR_OFFSET_CRC, w1)

    # now output all three binary arrays in the proper order
    output = output + '_OTA_blob.bin'
    blob2wiredfile = output # save the output of bin2blob for use by blob2wired
    am_print("Writing to file ", output)
    with open(output, mode = 'wb') as out:
        out.write(hdr_binarray[0:AM_IMAGEHDR_START_ENCRYPT])
        out.write(enc_binarray)


#******************************************************************************
#
# Generate the image blob as per command line parameters
#
#******************************************************************************
def blob2wired_process(appFile, imagetype, loadaddress, authalgo, encalgo, authKeyIdx, encKeyIdx, optionsVal, maxSize, output):
    global uploadbinfile

    app_binarray = bytearray()
    # Open the file, and read it into an array of integers.
    print('testing: ' + appFile )
    with open(appFile,'rb') as f_app:
        app_binarray.extend(f_app.read())
        f_app.close()

    # Make sure it is page multiple
    if ((maxSize & (FLASH_PAGE_SIZE - 1)) != 0):
        am_print ("split needs to be multiple of flash page size", level=AM_PRINT_LEVEL_ERROR)
        return

    if (encalgo != 0):
        if ((encKeyIdx < minAesKeyIdx) or (encKeyIdx > maxAesKeyIdx)):
            am_print("Invalid encKey Idx ", encKeyIdx, level=AM_PRINT_LEVEL_ERROR)
            return
        if (encalgo == 2):
            if (encKeyIdx & 0x1):
                am_print("Invalid encKey Idx ", encKeyIdx, level=AM_PRINT_LEVEL_ERROR);
                return
            keySize = 32
        else:
            keySize = 16
    if (authalgo != 0):
        if ((authKeyIdx < minHmacKeyIdx) or (authKeyIdx > maxHmacKeyIdx) or (authKeyIdx & 0x1)):
            am_print("Invalid authKey Idx ", authKeyIdx, level=AM_PRINT_LEVEL_ERROR);
            return

    hdr_length  = AM_WU_IMAGEHDR_SIZE;   #fixed header length
    am_print("Header Size = ", hex(hdr_length))

    orig_app_length  = (len(app_binarray))

    if (encalgo != 0):
        block_size = keySize
        app_binarray = pad_to_block_size(app_binarray, block_size, 1)
    else:
        # Add Padding
        app_binarray = pad_to_block_size(app_binarray, 4, 0)
    
    app_length  = (len(app_binarray))
    am_print("app_size ",hex(app_length), "(",app_length,")")

    if (app_length + hdr_length > maxSize):
        am_print("Image size bigger than max - Creating Split image")

    start = 0
    # now output all three binary arrays in the proper order
    output = output + '_Wired_OTA_blob.bin'
    uploadbinfile = output; # save the name of the output from blob2wired
    out = open(output, mode = 'wb')

    while (start < app_length):
        #generate mutable byte array for the header
        hdr_binarray = bytearray([0x00]*hdr_length);

        if (app_length - start > maxSize):
            end = start + maxSize
        else:
            end = app_length

        if (imagetype == AM_SECBOOT_WIRED_IMAGETYPE_INFO0_NOOTA):
            key = INFO_KEY
            # word offset
            fill_word(hdr_binarray, AM_WU_IMAGEHDR_OFFSET_ADDR, loadaddress>>2)
        else:
            key = FLASH_KEY
            # load address
            fill_word(hdr_binarray, AM_WU_IMAGEHDR_OFFSET_ADDR, loadaddress)
        # Create imageType & options
        hdr_binarray[AM_WU_IMAGEHDR_OFFSET_IMAGETYPE] = imagetype
        # Set the options only for the first block
        if (start == 0):
            hdr_binarray[AM_WU_IMAGEHDR_OFFSET_OPTIONS] = optionsVal
        else:
            hdr_binarray[AM_WU_IMAGEHDR_OFFSET_OPTIONS] = 0

        # Create Info0 Update Blob for wired update
        fill_word(hdr_binarray, AM_WU_IMAGEHDR_OFFSET_KEY, key)
        # update size
        fill_word(hdr_binarray, AM_WU_IMAGEHDR_OFFSET_SIZE, end-start)

        w0 = ((authalgo & 0xf) | ((authKeyIdx << 8) & 0xf00) | ((encalgo << 16) & 0xf0000) | ((encKeyIdx << 24) & 0x0f000000))

        fill_word(hdr_binarray, 0, w0)

        if (encalgo != 0):
            keyIdx = encKeyIdx - minAesKeyIdx
            ivValAes = os.urandom(AM_SECBOOT_AESCBC_BLOCK_SIZE_BYTES)
            am_print("Initialization Vector")
            am_print([hex(n) for n in ivValAes])
            keyAes = os.urandom(keySize)
            am_print("AES Key used for encryption")
            am_print([hex(keyAes[n]) for n in range (0, keySize)])
            # Encrypted Part - after security header
            enc_binarray = encrypt_app_aes((hdr_binarray[AM_WU_IMAGEHDR_START_ENCRYPT:hdr_length] + app_binarray[start:end]), keyAes, ivValAes)
#            am_print("Key used for encrypting AES Key")
#            am_print([hex(keyTblAes[keyIdx*AM_SECBOOT_KEYIDX_BYTES + n]) for n in range (0, keySize)])
            # Encrypted Key
            enc_key = encrypt_app_aes(keyAes, keyTblAes[keyIdx*AM_SECBOOT_KEYIDX_BYTES:(keyIdx*AM_SECBOOT_KEYIDX_BYTES + keySize)], ivVal0)
            am_print("Encrypted Key")
            am_print([hex(enc_key[n]) for n in range (0, keySize)])
            # Fill up the IV
            for x in range(0, AM_SECBOOT_AESCBC_BLOCK_SIZE_BYTES):
                hdr_binarray[AM_WU_IMAGEHDR_OFFSET_IV + x]  = ivValAes[x]
            # Fill up the Encrypted Key
            for x in range(0, keySize):
                hdr_binarray[AM_WU_IMAGEHDR_OFFSET_KEK + x]  = enc_key[x]
        else:
            enc_binarray = hdr_binarray[AM_WU_IMAGEHDR_START_ENCRYPT:hdr_length] + app_binarray[start:end]


        if (authalgo != 0): # Authentication needed
            keyIdx = authKeyIdx - minHmacKeyIdx
#            am_print("Key used for HMAC")
#            am_print([hex(keyTblHmac[keyIdx*AM_SECBOOT_KEYIDX_BYTES + n]) for n in range (0, AM_HMAC_SIG_SIZE)])
            # Initialize the HMAC - Sign is computed on image following the signature
            sig = compute_hmac(keyTblHmac[keyIdx*AM_SECBOOT_KEYIDX_BYTES:(keyIdx*AM_SECBOOT_KEYIDX_BYTES+AM_HMAC_SIG_SIZE)], hdr_binarray[AM_WU_IMAGEHDR_START_HMAC:AM_WU_IMAGEHDR_START_ENCRYPT] + enc_binarray)
            am_print("HMAC")
            am_print([hex(n) for n in sig])
            # Fill up the HMAC
            for x in range(0, AM_HMAC_SIG_SIZE):
                hdr_binarray[AM_WU_IMAGEHDR_OFFSET_SIG + x]  = sig[x]

        am_print("Writing to file ", output)
        am_print("Image from ", str(hex(start)), " to ", str(hex(end)), " will be loaded at", str(hex(loadaddress))) 
        out.write(hdr_binarray[0:AM_WU_IMAGEHDR_START_ENCRYPT])
        out.write(enc_binarray)

        # Reset start for next chunk
        start = end
        loadaddress = loadaddress + maxSize


#******************************************************************************
#
# Main function
#
#******************************************************************************
def upload(args, verboseprint):

    global loadTries
    global loadSuccess

    # Open a serial port, and communicate with Device
    #
    # Max flashing time depends on the amount of SRAM available.
    # For very large images, the flashing happens page by page.
    # However if the image can fit in the free SRAM, it could take a long time
    # for the whole image to be flashed at the end.
    # The largest image which can be stored depends on the max SRAM.
    # Assuming worst case ~100 ms/page of flashing time, and allowing for the
    # image to be close to occupying full SRAM (256K) which is 128 pages.

    connection_timeout = 5

    print('Connecting over serial port {}...'.format(args.port), flush=True)

    #Check to see if the com port is available
    try: 
        with serial.Serial(args.port, args.baud, timeout=connection_timeout) as ser:
            pass
    except:

        #Show a list of com ports and recommend one
        print("Detected Serial Ports:")
        devices = list_ports.comports()
        port = None
        for dev in devices:
            print(dev.description)
            # The SparkFun BlackBoard has CH340 in the description
            if 'CH340' in dev.description:
                print("The port you selected was not found. But we did detect a CH340 on " + dev.device + " so you might try again on that port.")
                break
            elif 'FTDI' in dev.description:
                print("The port you selected was not found. But we did detect an FTDI on " + dev.device + " so you might try again on that port.")
                break
            elif 'USB Serial Device' in dev.description:
                print("The port you selected was not found. But we did detect a USB Serial Device on " + dev.device + " so you might try again on that port.")
                break
        else: 
            print("Com Port not found - Did you select the right one?")

        exit()

    #Begin talking over com port

    #The auto-bootload sequence is good but not fullproof. The bootloader
    #fails to correctly catch the BOOT signal about 1 out of ten times.
    #Auto-retry this number of times before we give up.

    while loadTries < 3: 
        loadSuccess = False

        with serial.Serial(args.port, args.baud, timeout=connection_timeout) as ser:
            #DTR is driven low when serial port open. DTR has now pulled RST low.

            # time.sleep(0.005) #3ms and 10ms work well. Not 50, and not 0.
            time.sleep(0.008) #3ms and 10ms work well. Not 50, and not 0.

            #Setting RTS/DTR high causes the bootload pin to go high, then fall across 100ms
            ser.setDTR(0) #Set DTR high
            ser.setRTS(0) #Set RTS high - support the CH340E

            #Give bootloader a chance to run and check bootload pin before communication begins. But must initiate com before bootloader timeout of 250ms.
            time.sleep(0.100) #100ms works well

            ser.reset_input_buffer()    # reset the input bufer to discard any UART traffic that the device may have generated

            connect_device(ser, args, verboseprint)

            if(loadSuccess == True):
                print("Tries =", loadTries)
                print('Upload complete!')
                exit()
            else:
                print("Fail")
            
            loadTries = loadTries + 1
            
    print("Tries =", loadTries)
    print("Upload failed")
    exit()


#******************************************************************************
#
# Communicate with Device
#
# Given a serial port, connects to the target device using the
# UART.
#
#******************************************************************************
def connect_device(ser, args, verboseprint):

    global loadSuccess

    # Send Hello
    #generate mutable byte array for the header
    hello = bytearray([0x00]*4)
    fill_word(hello, 0, ((8 << 16) | AM_SECBOOT_WIRED_MSGTYPE_HELLO))
    verboseprint('Sending Hello.')
    response = send_command(hello, 88, ser, verboseprint)

    #Check if response failed
    if response == False:
        verboseprint("Failed to respond")
        return

    verboseprint("Received response for Hello")
    word = word_from_bytes(response, 4)
    if ((word & 0xFFFF) == AM_SECBOOT_WIRED_MSGTYPE_STATUS):
        # Received Status
        print("Bootloader connected")

        verboseprint("Received Status")
        verboseprint("length = ", hex((word >> 16)))
        verboseprint("version = ", hex(word_from_bytes(response, 8)))
        verboseprint("Max Storage = ", hex(word_from_bytes(response, 12)))
        verboseprint("Status = ", hex(word_from_bytes(response, 16)))
        verboseprint("State = ", hex(word_from_bytes(response, 20)))
        verboseprint("AMInfo = ")
        for x in range(24, 88, 4):
            verboseprint(hex(word_from_bytes(response, x)))

        abort = args.abort
        if (abort != -1):
            # Send OTA Desc
            verboseprint('Sending Abort command.')
            abortMsg = bytearray([0x00]*8);
            fill_word(abortMsg, 0, ((12 << 16) | AM_SECBOOT_WIRED_MSGTYPE_ABORT))
            fill_word(abortMsg, 4, abort)
            if send_ackd_command(abortMsg, ser, verboseprint) == False:
                verboseprint("Failed to ack command")
                return


        otadescaddr = args.otadesc
        if (otadescaddr != 0xFFFFFFFF):
            # Send OTA Desc
            verboseprint('Sending OTA Descriptor = ', hex(otadescaddr))
            otaDesc = bytearray([0x00]*8);
            fill_word(otaDesc, 0, ((12 << 16) | AM_SECBOOT_WIRED_MSGTYPE_OTADESC))
            fill_word(otaDesc, 4, otadescaddr)
            if send_ackd_command(otaDesc, ser, verboseprint) == False:
                verboseprint("Failed to ack command")
                return


        imageType = args.imagetype
        if (uploadbinfile != ''):

            # Read the binary file from the command line.
            with open(uploadbinfile, mode='rb') as binfile:
                application = binfile.read()
            # Gather the important binary metadata.
            totalLen = len(application)
            # Send Update command
            verboseprint('Sending Update Command.')

            # It is assumed that maxSize is 256b multiple
            maxImageSize = args.split
            if ((maxImageSize & (FLASH_PAGE_SIZE - 1)) != 0):
                verboseprint ("split needs to be multiple of flash page size")
                return

            # Each Block of image consists of AM_WU_IMAGEHDR_SIZE Bytes Image header and the Image blob
            maxUpdateSize = AM_WU_IMAGEHDR_SIZE + maxImageSize
            numUpdates = (totalLen + maxUpdateSize - 1) // maxUpdateSize # Integer division
            verboseprint("number of updates needed = ", numUpdates)

            end = totalLen
            for numUpdates in range(numUpdates, 0 , -1):
                start = (numUpdates-1)*maxUpdateSize
                crc = crc32(application[start:end])
                applen = end - start
                verboseprint("Sending block of size ", str(hex(applen)), " from ", str(hex(start)), " to ", str(hex(end)))
                end = end - applen

                update = bytearray([0x00]*16);
                fill_word(update, 0, ((20 << 16) | AM_SECBOOT_WIRED_MSGTYPE_UPDATE))
                fill_word(update, 4, applen)
                fill_word(update, 8, crc)
                # Size = 0 => We're not piggybacking any data to IMAGE command
                fill_word(update, 12, 0)

                if send_ackd_command(update, ser, verboseprint) == False:
                    verboseprint("Failed to ack command")
                    return

                # Loop over the bytes in the image, and send them to the target.
                resp = 0
                # Max chunk size is AM_MAX_UART_MSG_SIZE adjusted for the header for Data message
                maxChunkSize = AM_MAX_UART_MSG_SIZE - 12
                for x in range(0, applen, maxChunkSize):
                    # Split the application into chunks of maxChunkSize bytes.
                    # This is the max chunk size supported by the UART bootloader
                    if ((x + maxChunkSize) > applen):
                        chunk = application[start+x:start+applen]
#                        print(str(hex(start+x)), " to ", str(hex(applen)))
                    else:
                        chunk = application[start+x:start+x+maxChunkSize]
#                        print(str(hex(start+x)), " to ", str(hex(start + x + maxChunkSize)))

                    chunklen = len(chunk)

                    # Build a data packet with a "data command" a "length" and the actual
                    # payload bytes, and send it to the target.
                    dataMsg = bytearray([0x00]*8);
                    fill_word(dataMsg, 0, (((chunklen + 12) << 16) | AM_SECBOOT_WIRED_MSGTYPE_DATA))
                    # seqNo
                    fill_word(dataMsg, 4, x)

                    verboseprint("Sending Data Packet of length ", chunklen)
                    if send_ackd_command(dataMsg + chunk, ser, verboseprint) == False:
                        verboseprint("Failed to ack command")
                        return

        if (args.raw != ''):

            # Read the binary file from the command line.
            with open(args.raw, mode='rb') as rawfile:
                blob = rawfile.read()
            # Send Raw command
            verboseprint('Sending Raw Command.')
            ser.write(blob)

        if (args.reset != 0):
            # Send reset
            verboseprint('Sending Reset Command.')
            resetmsg = bytearray([0x00]*8);
            fill_word(resetmsg, 0, ((12 << 16) | AM_SECBOOT_WIRED_MSGTYPE_RESET))
            # options
            fill_word(resetmsg, 4, args.reset)
            if send_ackd_command(resetmsg, ser, verboseprint) == False:
                verboseprint("Failed to ack command")
                return

        
        #Success! We're all done
        loadSuccess = True
    else:
        # Received Wrong message
        verboseprint("Received Unknown Message")
        word = word_from_bytes(response, 4)
        verboseprint("msgType = ", hex(word & 0xFFFF))
        verboseprint("Length = ", hex(word >> 16))
        verboseprint([hex(n) for n in response])
        #print("!!!Wired Upgrade Unsuccessful!!!....Terminating the script")

        #exit()

#******************************************************************************
#
# Send ACK'd command
#
# Sends a command, and waits for an ACK.
#
#******************************************************************************
def send_ackd_command(command, ser, verboseprint):

    response = send_command(command, 20, ser, verboseprint)

    #Check if response failed
    if response == False:
        verboseprint("Response not valid")
        return False #Return error

    word = word_from_bytes(response, 4)
    if ((word & 0xFFFF) == AM_SECBOOT_WIRED_MSGTYPE_ACK):
        # Received ACK
        if (word_from_bytes(response, 12) != AM_SECBOOT_WIRED_ACK_STATUS_SUCCESS):
            verboseprint("Received NACK")
            verboseprint("msgType = ", hex(word_from_bytes(response, 8)))
            verboseprint("error = ", hex(word_from_bytes(response, 12)))
            verboseprint("seqNo = ", hex(word_from_bytes(response, 16)))
            #print("!!!Wired Upgrade Unsuccessful!!!....Terminating the script")
            verboseprint("Upload failed: No ack to command")

            return False #Return error

    return response

#******************************************************************************
#
# Send command
#
# Sends a command, and waits for the response.
#
#******************************************************************************
def send_command(params, response_len, ser, verboseprint):

    # Compute crc
    crc = crc32(params)
#    print([hex(n) for n in int_to_bytes(crc)])
#    print([hex(n) for n in params])
    # send crc first
    ser.write(int_to_bytes(crc))

    # Next, send the parameters.
    ser.write(params)

    response = ''
    response = ser.read(response_len)

    # Make sure we got the number of bytes we asked for.
    if len(response) != response_len:
        verboseprint('No response for command 0x{:08X}'.format(word_from_bytes(params, 0) & 0xFFFF))
        n = len(response)
        if (n != 0):
            verboseprint("received bytes ", len(response))
            verboseprint([hex(n) for n in response])
        return False

    return response

#******************************************************************************
#
# Send a command that uses an array of bytes as its parameters.
#
#******************************************************************************
def send_bytewise_command(command, params, response_len, ser):
    # Send the command first.
    ser.write(int_to_bytes(command))

    # Next, send the parameters.
    ser.write(params)

    response = ''
    response = ser.read(response_len)

    # Make sure we got the number of bytes we asked for.
    if len(response) != response_len:
        print("Upload failed: No reponse to command")
        verboseprint('No response for command 0x{:08X}'.format(command))
        exit()
        
    return response

#******************************************************************************
#
# Errors
#
#******************************************************************************
class BootError(Exception):
    pass

class NoAckError(BootError):
    pass


def parse_arguments():
    parser = argparse.ArgumentParser(description =
                                     'Combination script to upload application binaries to Artemis module. Includes:\n\t\'- bin2blob: create OTA blob from binary image\'\n\t\'- blob2wired: create wired update image from OTA blob\'\n\t\'- upload: send wired update image to Apollo3 Artemis module via serial port\'\n\nThere are many command-line arguments. They have been labeled by which steps they apply to\n')

    parser.add_argument('-a', dest = 'abort', default=-1, type=int, choices = [0,1,-1],
                        help = 'upload: Should it send abort command? (0 = abort, 1 = abort and quit, -1 = no abort) (default is -1)')

    parser.add_argument('--authalgo', dest = 'authalgo', type=auto_int, default=0, choices=range(0, AM_SECBOOT_AUTH_ALGO_MAX+1),
                        help = 'bin2blob, blob2wired: ' + str(helpAuthAlgo))

    parser.add_argument('--authI', dest = 'authI', type=auto_int, default=0, choices=[0,1],
                        help = 'bin2blob: Install Authentication check enabled (Default = N)?')

    parser.add_argument('--authB', dest = 'authB', type=auto_int, default=0, choices=[0,1],
                        help = 'bin2blob: Boot Authentication check enabled (Default = N)?')

    parser.add_argument('--authkey', dest = 'authkey', type=auto_int, default=(minHmacKeyIdx), choices = range(minHmacKeyIdx, maxHmacKeyIdx + 1),
                        help = 'bin2blob, blob2wired: Authentication Key Idx? (' + str(minHmacKeyIdx) + ' to ' + str(maxHmacKeyIdx) + ')')

    parser.add_argument('-b', dest='baud', default=115200, type=int,
                        help = 'upload: Baud Rate (default is 115200)')

    parser.add_argument('--bin', dest='appFile', type=argparse.FileType('rb'),
                        help='bin2blob: binary file (blah.bin)')

    parser.add_argument('-clean', dest='clean', default=0, type=int,
                        help = 'All: whether or not to remove intermediate files')

    parser.add_argument('--child0', dest = 'child0', type=auto_int, default=hex(0xFFFFFFFF),
                        help = 'bin2blob: child (blobPtr#0 for Main / feature key for AM3P)')

    parser.add_argument('--child1', dest = 'child1', type=auto_int, default=hex(0xFFFFFFFF),
                        help = 'bin2blob: child (blobPtr#1 for Main)')

    parser.add_argument('--crcI', dest = 'crcI', type=auto_int, default=1, choices=[0,1],
                        help = 'bin2blob: Install CRC check enabled (Default = Y)?')

    parser.add_argument('--crcB', dest = 'crcB', type=auto_int, default=0, choices=[0,1],
                        help = 'bin2blob: Boot CRC check enabled (Default = N)?')

    parser.add_argument('--encalgo', dest = 'encalgo', type=auto_int, default=0, choices = range(0, AM_SECBOOT_ENC_ALGO_MAX+1),
                        help = 'bin2blob, blob2wired: ' + str(helpEncAlgo))

    parser.add_argument('--erasePrev', dest = 'erasePrev', type=auto_int, default=0, choices=[0,1],
                        help = 'bin2blob: erasePrev (Valid only for main)')

    # parser.add_argument('-f', dest='binfile', default='',
    #                     help = 'upload: Binary file to program into the target device')

    parser.add_argument('-i', dest = 'imagetype', default=AM_SECBOOT_WIRED_IMAGETYPE_INVALID, type=auto_int,
                        choices = [
                                (AM_SECBOOT_WIRED_IMAGETYPE_SBL),
                                (AM_SECBOOT_WIRED_IMAGETYPE_AM3P),
                                (AM_SECBOOT_WIRED_IMAGETYPE_PATCH),
                                (AM_SECBOOT_WIRED_IMAGETYPE_MAIN),
                                (AM_SECBOOT_WIRED_IMAGETYPE_CHILD),
                                (AM_SECBOOT_WIRED_IMAGETYPE_CUSTPATCH),
                                (AM_SECBOOT_WIRED_IMAGETYPE_NONSECURE),
                                (AM_SECBOOT_WIRED_IMAGETYPE_INFO0),
                                (AM_SECBOOT_WIRED_IMAGETYPE_INFO0_NOOTA),
                                (AM_SECBOOT_WIRED_IMAGETYPE_INVALID)
                                ],
                        help = 'blob2wired, upload: ImageType ('
                                + str(AM_SECBOOT_WIRED_IMAGETYPE_SBL) + ': SBL, '
                                + str(AM_SECBOOT_WIRED_IMAGETYPE_AM3P) + ': AM3P, '
                                + str(AM_SECBOOT_WIRED_IMAGETYPE_PATCH) + ': Patch, '
                                + str(AM_SECBOOT_WIRED_IMAGETYPE_MAIN) + ': Main, '
                                + str(AM_SECBOOT_WIRED_IMAGETYPE_CHILD) + ': Child, '
                                + str(AM_SECBOOT_WIRED_IMAGETYPE_CUSTPATCH) + ': CustOTA, '
                                + str(AM_SECBOOT_WIRED_IMAGETYPE_NONSECURE) + ': NonSecure, '
                                + str(AM_SECBOOT_WIRED_IMAGETYPE_INFO0) + ': Info0 '
                                + str(AM_SECBOOT_WIRED_IMAGETYPE_INFO0_NOOTA) + ': Info0_NOOTA) '
                                + str(AM_SECBOOT_WIRED_IMAGETYPE_INVALID) + ': Invalid) '
                                '- default[Invalid]')

    parser.add_argument('--kek', dest = 'kek', type=auto_int, default=(minAesKeyIdx), choices = range(minAesKeyIdx, maxAesKeyIdx+1),
                        help = 'KEK index? (' + str(minAesKeyIdx) + ' to ' + str(maxAesKeyIdx) + ')')

    parser.add_argument('--load-address-wired', dest='loadaddress_blob', type=auto_int, default=hex(0x60000),
                        help='blob2wired: Load address of the binary - Where in flash the blob will be stored (could be different than install address of binary within).')

    parser.add_argument('--load-address-blob', dest='loadaddress_image', type=auto_int, default=hex(AM_SECBOOT_DEFAULT_NONSECURE_MAIN),
                        help='bin2blob: Load address of the binary.')

    parser.add_argument('--loglevel', dest='loglevel', type=auto_int, default=AM_PRINT_LEVEL_INFO,
                        choices = range(AM_PRINT_LEVEL_MIN, AM_PRINT_LEVEL_MAX+1),
                        help='bin2blob, blob2wired: ' + str(helpPrintLevel))

    parser.add_argument('--magic-num', dest='magic_num', default=hex(AM_IMAGE_MAGIC_NONSECURE),
                        type=lambda x: x.lower(),
#                        type = str.lower,
                        choices = [
                                hex(AM_IMAGE_MAGIC_MAIN),
                                hex(AM_IMAGE_MAGIC_CHILD),
                                hex(AM_IMAGE_MAGIC_CUSTPATCH),
                                hex(AM_IMAGE_MAGIC_NONSECURE),
                                hex(AM_IMAGE_MAGIC_INFO0)
                                ],
                        help = 'bin2blob: Magic Num ('
                                + str(hex(AM_IMAGE_MAGIC_MAIN)) + ': Main, '
                                + str(hex(AM_IMAGE_MAGIC_CHILD)) + ': Child, '
                                + str(hex(AM_IMAGE_MAGIC_CUSTPATCH)) + ': CustOTA, '
                                + str(hex(AM_IMAGE_MAGIC_NONSECURE)) + ': NonSecure, '
                                + str(hex(AM_IMAGE_MAGIC_INFO0)) + ': Info0) '
                                '- default[Main]'
                                )

    parser.add_argument('-o', dest = 'output', default='wuimage',
                    help = 'all: Output filename (without the extension) [also used for intermediate filenames]')

    parser.add_argument('-ota', dest = 'otadesc', type=auto_int, default=0xFE000,
                        help = 'upload: OTA Descriptor Page address (hex) - (Default is 0xFE000 - at the end of main flash) - enter 0xFFFFFFFF to instruct SBL to skip OTA')
    
    parser.add_argument('--options', dest = 'options', type=auto_int, default=0x1,
                        help = 'blob2wired: Options (16b hex value) - bit0 instructs to perform OTA of the image after wired download (set to 0 if only downloading & skipping OTA flow)')

    parser.add_argument('-p', dest = 'protection', type=auto_int, default=0, choices = [0x0, 0x1, 0x2, 0x3],
                        help = 'bin2blob: protection info 2 bit C W')

    parser.add_argument('-port', dest = 'port', help = 'upload: Serial COMx Port')

    parser.add_argument('-r', dest = 'reset', default=1, type=auto_int, choices = [0,1,2],
                        help = 'upload: Should it send reset command after image download? (0 = no reset, 1 = POI, 2 = POR) (default is 1)')

    parser.add_argument('--raw', dest='raw', default='',
                        help = 'upload: Binary file for raw message')

    parser.add_argument('--split', dest='split', type=auto_int, default=hex(MAX_DOWNLOAD_SIZE),
                        help='blob2wired, upload: Specify the max block size if the image will be downloaded in pieces')

    parser.add_argument('--version', dest = 'version', type=auto_int, default=0,
                        help = 'bin2blob: version (15 bit)')

    parser.add_argument("-v", "--verbose", default=0, help="All: Enable verbose output",
                        action="store_true")


    args = parser.parse_args()
    args.magic_num = int(args.magic_num, 16)


    return args



#******************************************************************************
#
# Main function.
#
#******************************************************************************

# example calling:
# python artemis_bin_to_board.py --bin application.bin --load-address-blob 0x20000 --magic-num 0xCB -o application --version 0x0 --load-address-wired 0xC000 -i 6 --options 0x1 -b 921600 -port COM4 -r 1 -v

def main():
    # Read the arguments.
    args = parse_arguments()
    am_set_print_level(args.loglevel)

    global blob2wiredfile

    bin2blob_process(args.loadaddress_blob, args.appFile, args.magic_num, args.crcI, args.crcB, args.authI, args.authB, args.protection, args.authkey, args.output, args.kek, args.version, args.erasePrev, args.child0, args.child1, args.authalgo, args.encalgo)
    blob2wired_process( blob2wiredfile, args.imagetype, args.loadaddress_image, args.authalgo, args.encalgo, args.authkey, args.kek, args.options, args.split, args.output)

    # todo: link the bin2blob step with the blob2wired step by input/output files


    #Create print function for verbose output if caller deems it: https://stackoverflow.com/questions/5980042/how-to-implement-the-verbose-or-v-option-into-a-script
    if args.verbose:
        def verboseprint(*args):
            # Print each argument separately so caller doesn't need to
            # stuff everything to be printed into a single string
            for arg in args:
                print(arg, end=''),
            print()
    else:   
        verboseprint = lambda *a: None      # do-nothing function

    upload(args, verboseprint)

    if(args.clean == 1):
        print('Cleaning up intermediate files') # todo: why isnt this showing w/ -clean option?


if __name__ == '__main__':
    main()
