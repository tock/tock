#Inventor: Hoseok Lee

#Issue List
#Issue 1 (Solved!): Sometimes the lenght of data_in is increased by 1byte(adding \n)
#It causes if else statement comparison, when checking command strings from OTA_app
#Originally the length of data_in have to be same as the one of self.command
#So, it maybe reqire to slice to the length of data_in with the length of self.command
#if there is a problem, release below #data_in = data_in[:self.rsp_size]
#If you don't slice data_in, helpful debug!() message will be presented from tock board

#Issue 2: 
#1) Reason: self.u32Crc ^= 0xffffffff => numpy.int64 (Auto casting!) => 8byte length!
#Total data packet length = 521 bytes!(It's supposed to be 517 byte! => 4byte over!)
#When receiving 517 byte, callback function is fired
#And the remaining 4 bytes are sent. Since the the amount of data do not reach to 517 bytes
#Callback function was not fired! When sending next data packet (for CRC check),
#The callback function will be triggered in the situation that the reamaining 4 bytes + 513 bytes (CRC data packet) 
#4 bytes(CRC data packet) are alos left like chain reaction.
#Because of that, Command decoder and CRC is not machted!
#2) Solution: numpy value need to be converted to int(4byte)
#3) Todo: Need to test lots of apps to verify this conversion!     

import sys
import serial
import time
import io
import numpy as np
from enum import Enum
from tqdm import tqdm


crc32_posix_lookup_table = np.array([
  0x00000000, 0x04c11db7, 0x09823b6e, 0x0d4326d9, 0x130476dc, 0x17c56b6b, 0x1a864db2, 0x1e475005,
  0x2608edb8, 0x22c9f00f, 0x2f8ad6d6, 0x2b4bcb61, 0x350c9b64, 0x31cd86d3, 0x3c8ea00a, 0x384fbdbd,
  0x4c11db70, 0x48d0c6c7, 0x4593e01e, 0x4152fda9, 0x5f15adac, 0x5bd4b01b, 0x569796c2, 0x52568b75,
  0x6a1936c8, 0x6ed82b7f, 0x639b0da6, 0x675a1011, 0x791d4014, 0x7ddc5da3, 0x709f7b7a, 0x745e66cd,
  0x9823b6e0, 0x9ce2ab57, 0x91a18d8e, 0x95609039, 0x8b27c03c, 0x8fe6dd8b, 0x82a5fb52, 0x8664e6e5,
  0xbe2b5b58, 0xbaea46ef, 0xb7a96036, 0xb3687d81, 0xad2f2d84, 0xa9ee3033, 0xa4ad16ea, 0xa06c0b5d,
  0xd4326d90, 0xd0f37027, 0xddb056fe, 0xd9714b49, 0xc7361b4c, 0xc3f706fb, 0xceb42022, 0xca753d95,
  0xf23a8028, 0xf6fb9d9f, 0xfbb8bb46, 0xff79a6f1, 0xe13ef6f4, 0xe5ffeb43, 0xe8bccd9a, 0xec7dd02d,
  0x34867077, 0x30476dc0, 0x3d044b19, 0x39c556ae, 0x278206ab, 0x23431b1c, 0x2e003dc5, 0x2ac12072,
  0x128e9dcf, 0x164f8078, 0x1b0ca6a1, 0x1fcdbb16, 0x018aeb13, 0x054bf6a4, 0x0808d07d, 0x0cc9cdca,
  0x7897ab07, 0x7c56b6b0, 0x71159069, 0x75d48dde, 0x6b93dddb, 0x6f52c06c, 0x6211e6b5, 0x66d0fb02,
  0x5e9f46bf, 0x5a5e5b08, 0x571d7dd1, 0x53dc6066, 0x4d9b3063, 0x495a2dd4, 0x44190b0d, 0x40d816ba,
  0xaca5c697, 0xa864db20, 0xa527fdf9, 0xa1e6e04e, 0xbfa1b04b, 0xbb60adfc, 0xb6238b25, 0xb2e29692,
  0x8aad2b2f, 0x8e6c3698, 0x832f1041, 0x87ee0df6, 0x99a95df3, 0x9d684044, 0x902b669d, 0x94ea7b2a,
  0xe0b41de7, 0xe4750050, 0xe9362689, 0xedf73b3e, 0xf3b06b3b, 0xf771768c, 0xfa325055, 0xfef34de2,
  0xc6bcf05f, 0xc27dede8, 0xcf3ecb31, 0xcbffd686, 0xd5b88683, 0xd1799b34, 0xdc3abded, 0xd8fba05a,
  0x690ce0ee, 0x6dcdfd59, 0x608edb80, 0x644fc637, 0x7a089632, 0x7ec98b85, 0x738aad5c, 0x774bb0eb,
  0x4f040d56, 0x4bc510e1, 0x46863638, 0x42472b8f, 0x5c007b8a, 0x58c1663d, 0x558240e4, 0x51435d53,
  0x251d3b9e, 0x21dc2629, 0x2c9f00f0, 0x285e1d47, 0x36194d42, 0x32d850f5, 0x3f9b762c, 0x3b5a6b9b,
  0x0315d626, 0x07d4cb91, 0x0a97ed48, 0x0e56f0ff, 0x1011a0fa, 0x14d0bd4d, 0x19939b94, 0x1d528623,
  0xf12f560e, 0xf5ee4bb9, 0xf8ad6d60, 0xfc6c70d7, 0xe22b20d2, 0xe6ea3d65, 0xeba91bbc, 0xef68060b,
  0xd727bbb6, 0xd3e6a601, 0xdea580d8, 0xda649d6f, 0xc423cd6a, 0xc0e2d0dd, 0xcda1f604, 0xc960ebb3,
  0xbd3e8d7e, 0xb9ff90c9, 0xb4bcb610, 0xb07daba7, 0xae3afba2, 0xaafbe615, 0xa7b8c0cc, 0xa379dd7b,
  0x9b3660c6, 0x9ff77d71, 0x92b45ba8, 0x9675461f, 0x8832161a, 0x8cf30bad, 0x81b02d74, 0x857130c3,
  0x5d8a9099, 0x594b8d2e, 0x5408abf7, 0x50c9b640, 0x4e8ee645, 0x4a4ffbf2, 0x470cdd2b, 0x43cdc09c,
  0x7b827d21, 0x7f436096, 0x7200464f, 0x76c15bf8, 0x68860bfd, 0x6c47164a, 0x61043093, 0x65c52d24,
  0x119b4be9, 0x155a565e, 0x18197087, 0x1cd86d30, 0x029f3d35, 0x065e2082, 0x0b1d065b, 0x0fdc1bec,
  0x3793a651, 0x3352bbe6, 0x3e119d3f, 0x3ad08088, 0x2497d08d, 0x2056cd3a, 0x2d15ebe3, 0x29d4f654,
  0xc5a92679, 0xc1683bce, 0xcc2b1d17, 0xc8ea00a0, 0xd6ad50a5, 0xd26c4d12, 0xdf2f6bcb, 0xdbee767c,
  0xe3a1cbc1, 0xe760d676, 0xea23f0af, 0xeee2ed18, 0xf0a5bd1d, 0xf464a0aa, 0xf9278673, 0xfde69bc4,
  0x89b8fd09, 0x8d79e0be, 0x803ac667, 0x84fbdbd0, 0x9abc8bd5, 0x9e7d9662, 0x933eb0bb, 0x97ffad0c,
  0xafb010b1, 0xab710d06, 0xa6322bdf, 0xa2f33668, 0xbcb4666d, 0xb8757bda, 0xb5365d03, 0xb1f740b4], dtype = np.uint32)

    
class enState(Enum):
    enInit = 0
    enInitFail = 1
    enWrite = 2
    enWriteFail = 3
    enWriteFailInvalidTBF = 4
    enwritePadBound = 5
    enWritePadBoundFail = 6
    enWritePadApps = 7
    enWritePadAppsFail = 8
    enCrc = 9
    enCrcFail = 10
    enAppload = 11
    enAppLoadFail = 12
    enAppErase = 13
    enAppEraseFail = 14
    enAppEraseSuccess = 15
    enSuccess = 16
    enFail = 17
    enDebug = 18

class cls_ota_serial:
    
    OPTION_BYTE_DUMMY = 0xFFFFFFFF
    PADDING_BYTE = 0x1
    INVALID_BYTE = 0xFF
    
    #Commands from this tool to OTA app
    COMMAND_FIND_STADDR = 0x5A
    COMMAND_WRITE_BINARY_DATA = 0x5B
    COMMAND_WRITE_PADDING_BOUNDARY = 0x5C
    COMMAND_WRITE_PADDING_APPS = 0x5D
    COMMAND_SEND_CRC = 0x5E
    COMMAND_APP_LOAD = 0x5F
    COMMAND_APP_ERASE = 0x60
    COMMAND_DEBUG = 0x61
    

    #Response from the OTA app. The length of response have to be 20 bytes!
    #And this size is alos have to be consistent with the size of response data from OTA app
    RESPONSE_FIND_STADDR_OK             = "find staddr ok     \n"
    RESPONSE_FIND_STADDR_FAIL           = "find staddr fail   \n"
    RESPONSE_WRITE_BINARY_OK            = "write binary ok    \n"
    RESPONSE_WRITE_BINARY_FAIL          = "write binary fail  \n"
    RESPONSE_WRITE_PADDING_BUNDRY_OK    = "write pad 01 ok    \n"
    RESPONSE_WRITE_PADDING_BUNDRY_FAIL  = "write pad 01 fail  \n"
    RESPONSE_CRC_CONSISTENCY_OK         = "checksum ok        \n"
    RESPONSE_CRC_CONSISTENCY_FAIL       = "checksum fail      \n"
    RESPONSE_APP_LOAD_OK                = "app load ok        \n"
    RESPONSE_APP_LOAD_FAIL              = "app load fail      \n"
    RESPONSE_ERASE_OK                   = "erase ok           \n"
    RESPONSE_ERASE_FAIL                 = "erase fail         \n"
    RESPONSE_WRITE_PADDING_APPS_OK      = "write pad app ok   \n"
    RESPONSE_WRITE_PADDING_APPS_FAIL    = "write pad app fail \n"
    RESPONSE_INVALID_TBF_HEADER         = "invalid tbf header \n"

    def __init__(self, binary_size):
        #use 512 byte pages to simplify the implementations and reduce uncertainty.
        self.page_size = 512 
        
        #application size
        self.app_size = 0
        
        #page counter variable to control loop, when sending binary data
        self.page_num = 0
        
        #response size
        self.rsp_size = 20
        
        #calculate crc32-posix
        self.u32Crc = np.uint32(0)
        
        #save the binary code size of an app which have to be transferred to OTA app 
        self.binary_size = binary_size
        
        #initialize state variable
        self.state = enState.enInit
    
    def fn_ota_serial_config_set(self, sp):
        # Open the actual serial port

        sp.port = '/dev/ttyS4'
        sp.baudrate = 115200
        sp.parity=serial.PARITY_NONE
        sp.stopbits=1
        sp.xonxoff=0
        sp.rtscts=0
        sp.timeout=60
        # Try to set initial conditions, but not all platforms support them.
        # https://github.com/pyserial/pyserial/issues/124#issuecomment-227235402
        
        sp.dtr = 0
        sp.rts = 0
        
        for i in range(0, 15):
            try:
                sp.open()
                break
            except:
                print("retrying open serial port for UART")
                
        time.sleep(0.05)
        
        #Flush buffers
        sp.flushInput()
        sp.flushOutput() 
        
    
    def fn_crc32_posix(self, banary_part):
        for banary in banary_part:
            self.u32Crc = (self.u32Crc << np.uint32(8)) ^ crc32_posix_lookup_table[((self.u32Crc >> np.uint32(24)) ^ np.uint32(banary)) & 0x000000ff]
            
    def fn_ota_serial_state_machine(self, binary_all, sp):
        #Organize data packet according to state
        #We use 517 bytes data packet size
        #1 byte (command) + 4 bytes (Optional) + 512 byte (binary data)
        if self.state == enState.enInit:
            #1 byte command
            data_packet = self.COMMAND_FIND_STADDR.to_bytes(1, 'big')
            
            #4 bytes application size (send by little endian. Need to convert, when receive data at OTA app)
            self.app_size = binary_all[4:8]
            data_packet += self.app_size
            
            #convert to bytearray
            data_packet = bytearray(data_packet)
            
            #512 bytes dummy 
            for i in range(self.page_size):
                data_packet.append(self.INVALID_BYTE)
            
            #write data to OTA via UART
            sp.write(data_packet)
                
            #read data from OTA via UART
            data_in = sp.readline().decode("utf-8")
            #Issue 1
            #data_in = data_in[:self.rsp_size]
            #print(data_in)
            
            if data_in == self.RESPONSE_FIND_STADDR_OK:
                #print("Success: Init Pass!")
                self.state = enState.enWrite 
                
            elif data_in == self.RESPONSE_FIND_STADDR_FAIL:
                self.state = enState.enInitFail  
                
            else:
                self.state = enState.enFail
                print("Message from tock => " + data_in)
                                   
        elif self.state == enState.enWrite:
            #1 byte command
            data_packet = self.COMMAND_WRITE_BINARY_DATA.to_bytes(1, 'big')
            
            #4 byte optional byte(page counter)
            data_packet += self.page_num.to_bytes(4, 'big')
            
            #append 512 bytes binary data
            binary_part = binary_all[(self.page_num * self.page_size) : self.page_size * (1 + self.page_num)]
            data_packet += binary_part
            
            #calculate crc32 value
            self.fn_crc32_posix(binary_part)
            
            #convert to bytearray
            data_packet = bytearray(data_packet)
            
            #write data to OTA via UART
            sp.write(data_packet)
                      
            #read data from OTA via UART             
            data_in = sp.readline().decode("utf-8")
            #Issue 1
            #data_in = data_in[:self.rsp_size]
            
            if data_in == self.RESPONSE_WRITE_BINARY_OK:
                self.page_num += 1
                page_cnt_max = self.binary_size / self.page_size #4
                
                if self.page_num < page_cnt_max:
                    self.state = enState.enWrite
                    #print("Success: Write!")
                else:
                    self.state = enState.enwritePadBound
                    #print("Success: Write Complete!")                           
            
            elif data_in == self.RESPONSE_INVALID_TBF_HEADER:
                self.state = enState.enWriteFailInvalidTBF
                
            elif data_in == self.RESPONSE_WRITE_BINARY_FAIL:
                self.state = enState.enWriteFail
                
            else:
                self.state = enState.enFail 
                print("Message from tock => " + data_in)                
        
        elif self.state == enState.enwritePadBound:
            #1 byte command
            data_packet = self.COMMAND_WRITE_PADDING_BOUNDARY.to_bytes(1, 'big')
            
            #4 bytes optional byte
            #page counter is already increased at the above write bianry data sequence
            data_packet += self.page_num.to_bytes(4, 'big')

            #convert to bytearray
            data_packet = bytearray(data_packet)
            
            #512 bytes in order for tockloader recognize the boundray of an app 
            for i in range(self.page_size):
                data_packet.append(self.PADDING_BYTE)
      
            #write data to OTA via UART
            sp.write(data_packet)
                
            #read data from OTA via UART
            data_in = sp.readline().decode("utf-8")
            #Issue 1
            #data_in = data_in[:self.rsp_size]
            #print(data_in)
            
            if data_in == self.RESPONSE_WRITE_PADDING_BUNDRY_OK:
                    self.state = enState.enCrc
                    #print("Success: Write Padding!")    
                    
            elif data_in == self.RESPONSE_WRITE_PADDING_BUNDRY_FAIL:
                self.state = enState.enWritePadBoundFail
                
            else:
                self.state = enState.enFail 
                print("Message from tock => " + data_in) 
                
        elif self.state == enState.enCrc:
            #1 byte command
            data_packet = self.COMMAND_SEND_CRC.to_bytes(1, 'big')          
            
            #Issue2      
            #finalize crc32-posix
            self.u32Crc ^= 0xffffffff
            Crc32_int = int(self.u32Crc)
            
            #4 bytes optional byte(crc32 result)
            data_packet += Crc32_int.to_bytes(4, 'big')           
            
            #For Erase Test
            #data_packet += self.DUMMY.to_bytes(4, 'big')
            
            #convert to bytearray 
            data_packet = bytearray(data_packet)
            
            #append 512 bytes dummy
            for i in range(self.page_size):
                data_packet.append(self.INVALID_BYTE)
            
            #write data to OTA via UART
            sp.write(data_packet)
                         
            #read data from OTA via UART
            data_in = sp.readline().decode("utf-8")
            #Issue 1
            #data_in = data_in[:self.rsp_size]
            #print(data_in)
            
            if data_in == self.RESPONSE_CRC_CONSISTENCY_OK:
                self.state = enState.enAppload
                #print("Success: CRC Consistency!")
                
            elif data_in == self.RESPONSE_CRC_CONSISTENCY_FAIL:
                self.state = enState.enCrcFail
                
            else:
                self.state = enState.enFail 
                print("Message from tock => " + data_in)
                    
        elif self.state == enState.enAppload:
            #1 byte command
            data_packet = self.COMMAND_APP_LOAD.to_bytes(1, 'big')
            
            #4 bytes optional byte(page counter)
            data_packet += self.OPTION_BYTE_DUMMY.to_bytes(4, 'big')
            
            #convert to bytearray 
            data_packet = bytearray(data_packet)
            
            #append 512 bytes dummy
            for i in range(self.page_size):
                data_packet.append(self.INVALID_BYTE)   
            
            #write data to OTA via UART
            sp.write(data_packet)

            #read data from OTA via UART
            data_in = sp.readline().decode("utf-8")
            #Issue 1
            #data_in = data_in[:self.rsp_size]
            #print(data_in)
            
            if data_in == self.RESPONSE_APP_LOAD_OK:
                #self.state = enState.enSuccess
                self.state = enState.enWritePadApps
                
            elif data_in == self.RESPONSE_APP_LOAD_FAIL:
                self.state = enState.enAppLoadFail
                
            else:
                self.state = enState.enAppLoadFail
                print("Message from tock => " + data_in)

        
        elif self.state == enState.enAppErase:
            #1 byte command
            data_packet = self.COMMAND_APP_ERASE.to_bytes(1, 'big')
            
            #4 bytes optional byte(page counter)
            data_packet += self.OPTION_BYTE_DUMMY.to_bytes(4, 'big')
            
            #convert to bytearray 
            data_packet = bytearray(data_packet)
            
            #append 512 bytes dummy
            for i in range(self.page_size):
                data_packet.append(self.INVALID_BYTE)
            
            sp.write(data_packet)

            #read data from OTA via UART
            data_in = sp.readline().decode("utf-8")
            #Issue 1
            #data_in = data_in[:self.rsp_size]
            #print(data_in)
            
            if data_in == self.RESPONSE_ERASE_OK:
                self.state = enState.enAppEraseSuccess 
                
            elif data_in == self.RESPONSE_ERASE_FAIL:
                self.state = enState.enAppEraseFail  
                
            else:
                self.state = enState.enFail 
                print("Message from tock => " + data_in)
    
        elif self.state == enState.enWritePadApps:
            #1 byte command
            data_packet = self.COMMAND_WRITE_PADDING_APPS.to_bytes(1, 'big')
            
            #4 bytes dummy optional byte
            data_packet += self.OPTION_BYTE_DUMMY.to_bytes(4, 'big')
            
            #convert to bytearray 
            data_packet = bytearray(data_packet)
            
            #512 bytes in order for tockloader recognize the boundray of an app 
            for i in range(self.page_size):
                data_packet.append(self.INVALID_BYTE)
      
            #write data to OTA via UART
            sp.write(data_packet)
                
            #read data from OTA via UART
            data_in = sp.readline().decode("utf-8")
            #Issue 1
            #data_in = data_in[:self.rsp_size]
            #print(data_in)
            
            if data_in == self.RESPONSE_WRITE_PADDING_APPS_OK:
                    self.state = enState.enSuccess
                    #self.state = enState.enDebug
                    #print("Success: Write Padding Apps!")
                    
                    #If you want to see some debug data, you can use enDebug state
                    #I will print out a couple of debug data after loading a new app successfully
                    
                    
            elif data_in == self.RESPONSE_WRITE_PADDING_APPS_FAIL:
                self.state = enState.enWritePadAppsFail
                
            else:
                self.state = enState.enFail 
                print("Message from tock => " + data_in)   
                
        elif self.state == enState.enDebug:   
            #1 byte command
            data_packet = self.COMMAND_DEBUG.to_bytes(1, 'big')
            
            #4 bytes optional byte(page counter)
            data_packet += self.OPTION_BYTE_DUMMY.to_bytes(4, 'big')
            
            #convert to bytearray 
            data_packet = bytearray(data_packet)
            
            #append 512 bytes dummy
            for i in range(self.page_size):
                data_packet.append(self.INVALID_BYTE)
            
            sp.write(data_packet)

            #read data from OTA via UART
            data_in = sp.readline().decode("utf-8")
            print("\n======Debug Data =========")
            print(data_in)
            self.state = enState.enSuccess 
            
def main(file_name):
    
    file = open(file_name, mode='rb')
    binary_all = file.read()
    binary_size = len(binary_all)
    sp = serial.Serial()
    
    ob_ota = cls_ota_serial(binary_size)
    ob_ota.fn_ota_serial_config_set(sp)
   
    
    pbar = tqdm(desc='OTA Update', total = (binary_size / ob_ota.page_size))
    
    while True:
        ob_ota.fn_ota_serial_state_machine(binary_all, sp)
        
        if ob_ota.state == enState.enInitFail:
            print("OTA Failure: Application cannot be loaded more than 4!")
            print("OTA Failure: Flash region is not enough to load the new application!")
            break

        elif ob_ota.state == enState.enWriteFail:
            print("OTA Failure: Write TBF Binary!")
            break        
        
        elif ob_ota.state == enState.enWriteFailInvalidTBF:
            print("OTA Failure: Invalid TBF!")
            break 
        
        elif ob_ota.state == enState.enWritePadBoundFail:
            print("OTA Failure: Padding 01 Write!")
            break
            
        elif ob_ota.state == enState.enCrcFail:
            ob_ota.state = enState.enAppErase
            print("OTA Failure: CRC consistency! We erase the loaded app..")
            print("Please wait.. It takes time..")
            #No break. we continue to go to the next state (erase the loaded app)

        elif ob_ota.state == enState.enAppLoadFail:
            #ob_ota.state = enState.enAppErase
            ob_ota.state = enState.enSuccess
            print("OTA Failure: App load Error caused by MPU alighment! We erase the loaded app.. It will take maximum 1 min..")
            #No break. we continue to go to the next state (erase the loaded app)
                    
        elif ob_ota.state == enState.enAppEraseSuccess:
            print("Success: Erase The Loaded App!")
            break
            
        elif ob_ota.state == enState.enAppEraseFail:
            print("OTA Failure: App Erase ")
            break
        
        elif ob_ota.state == enState.enWritePadAppsFail:
            print("OTA Failure: Padding Apps ")
            break
            
        elif ob_ota.state == enState.enSuccess:
            print("OTA Success: Dynamic App Load!")
            break
        
        elif ob_ota.state == enState.enFail:
            print("OTA Failure: Internal Error! You may disable process_printer and _process_console.start() at main.rs!")
            break
        
        pbar.update(1) 
        
    #End of update
    pbar.close()
    sp.close()
    
if __name__ == "__main__":
    #Todo: Check whether or not the input file is valid tbf file 
    
    try:
        file_name = sys.argv[1]
    except:
        print("Please input file name!")
        sys.exit(0)
    
    main(file_name)
